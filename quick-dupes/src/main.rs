use ::std::{
    borrow::Cow,
    collections::hash_map::Entry::{Occupied, Vacant},
    ffi::OsStr,
    fs,
    hash::Hash,
    io::{self, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering::SeqCst},
        Arc,
    },
    thread,
};

use ::clap::Parser;
use ::color_eyre::{eyre::eyre, Section};
use ::derive_more::Constructor;
use ::insensitive_buf::{Insensitive, InsensitiveBuf};
use ::rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use ::rustc_hash::FxHashMap;
use ::sha2::{
    digest::{generic_array::GenericArray, OutputSizeUser},
    Digest, Sha256,
};
use ::tinyvec::TinyVec;

mod cli;
mod error;
mod fmt_oneshot;
mod group_summary;
mod observer;

use crate::{
    cli::{Cli, Filter},
    error::log_if_err,
    group_summary::GroupSummary,
    observer::Observer,
};

type VecHashMap<K, V> = FxHashMap<K, TinyVec<[V; 3]>>;

fn fold_vec_hashmap<K: Hash + Eq, V: Default>(
    mut map: VecHashMap<K, V>,
    (key, value): (K, V),
) -> VecHashMap<K, V> {
    map.entry(key).or_default().push(value);
    map
}

fn reduce_vec_hashmap<K: Hash + Eq, V: Default>(
    a: VecHashMap<K, V>,
    b: VecHashMap<K, V>,
) -> VecHashMap<K, V> {
    let (mut a, b) = if b.len() < a.len() { (a, b) } else { (b, a) };
    for (key, value) in b {
        match a.entry(key) {
            Occupied(mut entry) => entry.get_mut().extend(value),
            Vacant(entry) => {
                entry.insert(value);
            }
        }
    }
    a
}

type HashArray = GenericArray<u8, <Sha256 as OutputSizeUser>::OutputSize>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Constructor)]
struct Key<'a> {
    name: Cow<'a, Insensitive>,
    size: u64,
    hash: HashArray,
}

#[derive(Debug)]
struct Shared {
    status: AtomicUsize,
    total_paths: AtomicUsize,
    filtered_paths: AtomicUsize,
    hashed_paths: AtomicUsize,
}

impl Shared {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            status: AtomicUsize::new(0),
            total_paths: AtomicUsize::new(0),
            filtered_paths: AtomicUsize::new(0),
            hashed_paths: AtomicUsize::new(0),
        })
    }
}

fn filter_entry<'a>(
    entry: ::walkdir::DirEntry,
    filter: &Filter,
) -> Option<(Key<'a>, Cow<'a, Path>)> {
    let path_bytes = entry.path().as_os_str().as_bytes();
    if filter.newlines.is_skip() && path_bytes.contains(&b'\n') {
        return None;
    }

    if let Some(regex) = &filter.regex {
        if !regex.is_match(path_bytes) {
            return None;
        }
    }

    let name = filter
        .name
        .is_no()
        .then_some(OsStr::new(""))
        .or_else(|| entry.path().file_name())?;

    let meta = log_if_err(::log::Level::Warn, || {
        entry.metadata().map_err(|err| {
            eyre!(
                "cannot get metadata of \"{path}\"",
                path = entry.path().display()
            )
            .error(err)
        })
    })?;
    if !meta.is_file() {
        return None;
    };
    let size = if filter.size.is_no() {
        0
    } else {
        let len = meta.len();
        if len == 0 {
            return None;
        }
        len
    };

    Some((
        Key::new(
            Cow::Owned::<Insensitive>(InsensitiveBuf::new(name.as_bytes())),
            size,
            Default::default(),
        ),
        Cow::Owned::<Path>(entry.into_path()),
    ))
}

fn main() -> ::color_eyre::Result<()> {
    ::color_eyre::install()?;
    let Cli {
        threads: _,
        path,
        print,
        null,
        canonicalize: _,
        filter,
        log,
    } = dbg!(Cli::parse().setup()?);

    // State shared across threads.
    let shared = Shared::new();

    // Thread observing state to give status updates.
    let observer = Observer::new(Arc::clone(&shared)).spawn()?;

    // Update state.
    let set_status = |val| {
        shared.status.store(val, SeqCst);
        observer.thread().unpark();
    };

    // Collect paths to work with.
    let path_list = path
        .iter()
        .flat_map(|path| {
            ::walkdir::WalkDir::new(path)
                .same_file_system(true)
                .follow_links(false)
                .min_depth(filter.min_depth)
                .max_depth(filter.max_depth)
                .into_iter()
                .filter_map(|entry| entry.ok())
        })
        .inspect(|_| {
            shared.total_paths.fetch_add(1, SeqCst);
        })
        .collect::<Vec<_>>();

    set_status(1);

    // First filter/collect pass of paths, using name and metadata.
    let groups = path_list
        .into_par_iter()
        .filter_map(|entry| {
            let entry = filter_entry(entry, &filter);

            if entry.is_some() {
                shared.filtered_paths.fetch_add(1, SeqCst);
            } else {
                shared.total_paths.fetch_sub(1, SeqCst);
            }

            entry
        })
        .fold(VecHashMap::default, fold_vec_hashmap)
        .reduce(VecHashMap::default, reduce_vec_hashmap)
        .into_iter()
        .filter(|(_, paths)| {
            let len = paths.len();
            if len <= 1 {
                shared.total_paths.fetch_sub(len, SeqCst);
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    // Optional second filter/collect pass, using hashes
    let flattened;
    let groups = if filter.hash.is_yes() {
        set_status(2);

        flattened = groups
            .iter()
            .flat_map(|(key, values)| ::std::iter::repeat(key).zip(values));

        flattened
            .par_bridge()
            .filter_map(|(Key { name, size, .. }, path)| {
                let hash = log_if_err(::log::Level::Error, || {
                    let mut hasher = Sha256::new();
                    let mut file =
                        io::BufReader::new(fs::File::open(path).map_err(|err| eyre!(err))?);

                    io::copy(&mut file, &mut hasher).map_err(|err| eyre!(err))?;

                    shared.hashed_paths.fetch_add(1, SeqCst);

                    Result::<_, ::color_eyre::Report>::Ok(hasher.finalize())
                })?;
                Some((
                    Key::new(Cow::Borrowed::<Insensitive>(name), *size, hash),
                    Cow::Borrowed::<Path>(path),
                ))
            })
            .fold(VecHashMap::default, fold_vec_hashmap)
            .reduce(VecHashMap::default, reduce_vec_hashmap)
            .into_iter()
            .filter(|(_, p)| {
                let len = p.len();
                if len <= 1 {
                    shared.total_paths.fetch_sub(len, SeqCst);
                    false
                } else {
                    true
                }
            })
            .collect::<Vec<_>>()
    } else {
        groups
    };

    set_status(3);

    thread::scope(|s| {
        let (mut print_thread, mut log_thread) = (None, None);

        if print {
            print_thread = Some(s.spawn(|| {
                let mut stdout = io::stdout().lock();
                let delim = if null { b'\0' } else { b'\n' };
                let mut write_group = |paths: &[Cow<'_, Path>]| -> Result<(), io::Error> {
                    for path in paths {
                        stdout.write_all(path.as_os_str().as_bytes())?;
                        stdout.write_all(&[delim])?;
                    }
                    stdout.write_all(&[delim])
                };

                for (_, paths) in &groups {
                    write_group(paths)
                        .map_err(|err| eyre!("failed to write group to stdout").error(err))?;
                }

                Result::<(), ::color_eyre::Report>::Ok(())
            }));
        }

        if log.log_groups {
            log_thread = Some(s.spawn(|| {
                for (i, (key, paths)) in groups.iter().enumerate() {
                    ::log::info!("\ngroup-{i} {:#}", GroupSummary::new(key, paths, &filter));
                }
            }));
        }

        if let Some(h) = print_thread {
            h.join().unwrap()?;
        }

        if let Some(h) = log_thread {
            h.join().unwrap();
        }

        Result::<(), ::color_eyre::Report>::Ok(())
    })?;

    let (size, total_size, total) = groups
        .iter()
        .map(|(Key { size, .. }, path)| (*size, *size * path.len() as u64, path.len()))
        .reduce(
            |(size_a, total_size_a, total_a), (size_b, total_size_b, total_b)| {
                (
                    size_a + size_b,
                    total_size_a + total_size_b,
                    total_a + total_b,
                )
            },
        )
        .unwrap_or((0, 0, 0));
    log::info!(
        "found {len} unique files ({size}), total {total} ({total_size}), diff {diff} ({size_diff})",
        len = groups.len(),
        size = ::bytesize::ByteSize(size),
        total_size = ::bytesize::ByteSize(total_size),
        diff = total - groups.len(),
        size_diff = ::bytesize::ByteSize(total_size - size),
    );

    observer.join().unwrap();

    Ok(())
}
