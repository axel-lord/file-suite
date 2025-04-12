//! Crate to recursively rename directories to lowercase.

use ::std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    os::{
        fd::{AsRawFd, OwnedFd, RawFd},
        unix::ffi::OsStrExt,
    },
    path::{Path, PathBuf},
    rc::Rc,
};

use ::clap::{Parser, ValueEnum};
use ::log::LevelFilter;
use ::nix::{
    fcntl::{AtFlags, OFlag, OpenHow, RenameFlags, ResolveFlag},
    sys::stat::Mode,
};
use ::tokio::task::{JoinSet, LocalSet};
use ::tokio_stream::StreamExt;
use clap::builder::PossibleValue;

/// Display a linked-list style path.
#[derive(Clone, Debug)]
struct DisplayPath {
    /// Prior segments if any.
    prior: Option<Rc<DisplayPath>>,

    /// Current segment.
    current: Rc<Path>,
}

impl DisplayPath {
    /// Create a new instance from a path-like.
    fn new(path: impl AsRef<Path>) -> Self {
        Self {
            prior: None,
            current: Rc::from(path.as_ref()),
        }
    }

    /// Create a new instance from a path-like and a prior [DisplayPath].
    fn with_prior(prior: Rc<Self>, path: impl AsRef<Path>) -> Self {
        Self {
            prior: Some(prior),
            current: Rc::from(path.as_ref()),
        }
    }
}

impl Display for DisplayPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack = Vec::from([Rc::new(self.clone())]);

        while let Some(value) = stack.last().unwrap().prior.as_ref() {
            stack.push(value.clone())
        }

        let path = stack
            .into_iter()
            .rev()
            .map(|value| Rc::clone(&value.current))
            .collect::<PathBuf>();

        Display::fmt(&path.display(), f)
    }
}

#[derive(Debug, Clone)]
/// [log::LevelFilter] wrapper for cli.
pub struct LogLevel(pub LevelFilter);

impl ValueEnum for LogLevel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self(LevelFilter::Off),
            Self(LevelFilter::Error),
            Self(LevelFilter::Warn),
            Self(LevelFilter::Info),
            Self(LevelFilter::Debug),
            Self(LevelFilter::Trace),
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.0.as_str()))
    }
}

/// More descriptive error for failing to canonicalize a path.
#[derive(Debug, thiserror::Error)]
#[error("failed to canonicalize path '{}', {err}", .path.display())]
struct CanonicalizationError {
    /// Path that was attempted to be canonicalized.
    path: PathBuf,

    /// Error returned by canonicalization.
    #[source]
    err: ::std::io::Error,
}

/// More descriptive open error.
#[derive(Debug, thiserror::Error)]
#[error("failed to open '{}' as a directory, '{err}'", .path.display())]
struct DirOpenError {
    /// Path that was attempted to be opened.
    path: PathBuf,

    /// Source.
    #[source]
    err: ::nix::errno::Errno,
}

/// Errors that may occur during initial file rename.
#[derive(Debug, thiserror::Error)]
enum InitialFileError {
    /// Error returned when trying to canonicalize a path.
    #[error(transparent)]
    Canonicalization(#[from] CanonicalizationError),

    /// Error returned when trying to open a directory.
    #[error(transparent)]
    DirOpen(#[from] DirOpenError),
}

/// Get name and parent of a path.
/// If needed path will be canonicalized, possibly returning a canonicalization error.
/// Value will be none if the path has no name such as "/".
///
fn get_name_parent(path: &Path) -> Result<Option<(OsString, PathBuf)>, CanonicalizationError> {
    // Path will always have a parent if a file name exist however we still fall back to
    // canonicalization in case this is wrong.
    if let (Some(name), Some(parent)) = (path.file_name(), path.parent()) {
        Ok(Some((name.to_os_string(), parent.to_path_buf())))
    } else {
        let path = path.canonicalize().map_err(|err| CanonicalizationError {
            path: path.to_path_buf(),
            err,
        })?;

        if let (Some(name), Some(parent)) = (path.file_name(), path.parent()) {
            Ok(Some((name.to_os_string(), parent.to_path_buf())))
        } else {
            Ok(None)
        }
    }
}

/// Recursively rename files to lowercase.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Log file renames and no-ops as INFO.
    #[arg(short, long)]
    verbose: bool,

    /// Max log level to display.
    ///
    /// If not specified will be INFO in release builds and TRACE in debug builds.
    #[arg(short, long)]
    log_level: Option<LogLevel>,

    /// File/Directory to start at.
    #[arg(required = true)]
    file: Vec<PathBuf>,
}

impl Cli {
    /// Run cli.
    ///
    /// # Panics
    /// If tokio runtime cannot be built.
    ///
    /// # Errors
    pub fn run(self) {
        let Self {
            verbose,
            log_level,
            file,
        } = self;

        env_logger::builder()
            .filter_module(
                "recursive_lowercase",
                log_level.map_or_else(
                    || {
                        if cfg!(debug_assertions) {
                            LevelFilter::Trace
                        } else {
                            LevelFilter::Info
                        }
                    },
                    |level| level.0,
                ),
            )
            .parse_default_env()
            .init();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let local_set = LocalSet::new();

        local_set.block_on(&runtime, async move {
            let mut set = JoinSet::<Result<(), InitialFileError>>::new();
            for file in file {
                set.spawn_local(async move {
                    if let Some((name, parent)) = get_name_parent(&file)? {
                        let dir = async_fs_utils::open_at(
                            None,
                            &parent,
                            Mode::empty(),
                            OFlag::O_DIRECTORY,
                        )
                        .await
                        .map_err(|err| DirOpenError {
                            path: parent.to_path_buf(),
                            err,
                        })?;
                        to_lower_recursive(
                            dir.as_raw_fd(),
                            name,
                            DisplayPath::new(parent).into(),
                            verbose,
                        )
                        .await;
                    } else {
                        let dir =
                            async_fs_utils::open_at(None, &file, Mode::empty(), OFlag::O_DIRECTORY)
                                .await
                                .map_err(|err| DirOpenError {
                                    path: file.to_path_buf(),
                                    err,
                                })?;

                        dir_contents_to_lower(dir, DisplayPath::new(file).into(), verbose).await;
                    };
                    Ok(())
                });
            }
            while let Some(res) = set.join_next().await {
                let Ok(res) = res.inspect_err(|err| log::error!("could not join task, {err}"))
                else {
                    continue;
                };
                if let Err(err) = res {
                    log::error!("{err}");
                }
            }
        });
    }
}

/// Turn a directory or file lower case.
async fn to_lower_recursive(
    dir_fd: RawFd,
    name: OsString,
    disp_path: Rc<DisplayPath>,
    verbose: bool,
) {
    let lower = insensitive_buf::to_lower(name.as_bytes());

    let new_disp;
    let new_name = if name.as_bytes() == lower {
        new_disp = Rc::new(DisplayPath::with_prior(disp_path, &name));
        if verbose {
            log::info!("skipping {}", new_disp);
        }
        name.as_os_str()
    } else {
        let lower = OsStr::from_bytes(&lower);
        let old_disp = Rc::new(DisplayPath::with_prior(disp_path.clone(), &name));
        match async_fs_utils::rename_at_2(
            Some(dir_fd),
            &name,
            Some(dir_fd),
            lower,
            RenameFlags::RENAME_NOREPLACE,
        )
        .await
        {
            Err(err) => {
                log::warn!(
                    "could not rename '{}' -> '{}', {err}",
                    old_disp,
                    DisplayPath::with_prior(disp_path, lower),
                );
                new_disp = old_disp;
                name.as_os_str()
            }
            Ok(()) => {
                new_disp = Rc::new(DisplayPath::with_prior(disp_path.clone(), lower));
                if verbose {
                    log::info!("renamed '{}' -> '{}'", old_disp, new_disp);
                }
                lower
            }
        }
    };

    let Ok(stat) =
        async_fs_utils::file_stat_at(Some(dir_fd), &new_name, AtFlags::AT_SYMLINK_NOFOLLOW)
            .await
            .inspect_err(|err| log::warn!("could not stat '{}', {err}", &new_disp))
    else {
        return;
    };

    // Explicitly disallow links.
    if stat.st_mode & libc::S_IFLNK == libc::S_IFLNK {
        return;
    }

    // Only continue if directory.
    if stat.st_mode & libc::S_IFDIR != libc::S_IFDIR {
        return;
    }

    let Ok(dir) = async_fs_utils::open_at_2(
        dir_fd,
        &new_name,
        OpenHow::new()
            .flags(OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW)
            .resolve(ResolveFlag::RESOLVE_NO_XDEV | ResolveFlag::RESOLVE_NO_SYMLINKS),
    )
    .await
    .inspect_err(|err| log::warn!("could not open '{}', {err}", &new_disp)) else {
        return;
    };

    dir_contents_to_lower(dir, new_disp, verbose).await;
}

/// Turn directory contents lowercase, dir_fd will have ownership taken.
async fn dir_contents_to_lower(dir_fd: OwnedFd, disp_path: Rc<DisplayPath>, verbose: bool) {
    let (dir, dir_fd) = match async_fs_utils::clone_fd(dir_fd).await {
        (dir, Ok(dir_fd)) => (dir, dir_fd),
        (dir, Err(err)) => {
            log::error!("failed to clone file descriptor {dir:?}, {err}");
            return;
        }
    };
    let Ok(dir) = async_fs_utils::open_dir(dir)
        .await
        .inspect_err(|err| log::warn!("failed to open directory '{}', {err}", disp_path))
    else {
        return;
    };

    let mut dir = async_fs_utils::read_dir(dir);
    let mut set = JoinSet::new();
    while let Some(res) = dir.next().await {
        let Ok(entry) =
            res.inspect_err(|err| log::warn!("could not open entry in'{}', {err}", disp_path))
        else {
            continue;
        };
        let name = entry.file_name().to_bytes();

        if name == b"." || name == b".." {
            continue;
        }

        let name = OsStr::from_bytes(name).to_os_string();
        set.spawn_local(to_lower_recursive(
            dir_fd.as_raw_fd(),
            name,
            Rc::clone(&disp_path),
            verbose,
        ));
    }
    while let Some(res) = set.join_next().await {
        if let Err(err) = res {
            log::error!("could not join task, {err}");
        }
    }
}
