//! Combine A nested folder structure by hardlinkning the files into a given directory.

use ::std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    hash::RandomState,
    path::{Path, PathBuf},
    sync::Arc,
};

use ::clap::Parser;
use ::color_eyre::{Section, eyre::eyre};
use ::itertools::Itertools;
use ::rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use ::walkdir::WalkDir;

/// Application to compile nested directory contents into a single directory.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
struct Cli {
    /// Where to compile files to.
    outdir: PathBuf,
    /// What directory/ies to compile. If left out current directory is used.
    indir: Vec<PathBuf>,
    /// Use provided value to separate path components.
    #[arg(long, short, default_value = "_")]
    sep: OsString,
    /// Use symlinks instead of hardlinks.
    #[arg(long)]
    symlink: bool,
}

impl ::file_suite_common::Run for Cli {
    type Err = ::color_eyre::Report;

    fn run(self) -> color_eyre::Result<()> {
        let Self {
            outdir,
            indir,
            sep,
            symlink,
        } = self;

        let indir = if indir.is_empty() {
            Vec::from([::std::env::current_dir()
                .map_err(|err| eyre!(err).suggestion("how was the executable called?"))?])
        } else {
            indir
        };

        let inset = HashSet::<_, RandomState>::from_iter(indir.into_iter().filter_map(|path| {
            path.canonicalize()
                .inspect_err(|err| {
                    ::log::warn!(
                        "could not canonicalize '{path}, {err}'",
                        path = path.display()
                    );
                })
                .ok()
        }));

        let file_paths = inset
            .par_iter()
            .flat_map_iter(|dir| {
                let dir = Arc::<Path>::from(dir.as_path());
                WalkDir::new(&dir)
                    .into_iter()
                    .filter_map(|entry| {
                        entry
                            .inspect_err(|err| ::log::warn!("could not get dir entry, {err}"))
                            .ok()
                            .and_then(|entry| {
                                entry
                                    .metadata()
                                    .ok()?
                                    .is_file()
                                    .then_some(entry.into_path())
                            })
                    })
                    .map(move |e| (Arc::clone(&dir), e))
            })
            .collect::<Vec<_>>();

        fs::create_dir_all(&outdir).map_err(|err| {
            eyre!(err).note(format!(
                "are there sufficient permissions to create '{outdir}' if it is missing?",
                outdir = outdir.display()
            ))
        })?;

        file_paths.into_par_iter().for_each(|(root, entry)| {
            let root = AsRef::<Path>::as_ref(&root);
            let entry = entry.as_path();

            let Ok(relative) = entry.strip_prefix(root).inspect_err(|err| {
                ::log::warn!(
                    "could not stip prefix '{root}' from '{entry}', {err}",
                    root = root.display(),
                    entry = entry.display()
                )
            }) else {
                return;
            };

            let new_name = Itertools::intersperse(
                relative.components().filter_map(|comp| match comp {
                    ::std::path::Component::Normal(os_str) => Some(os_str),
                    _ => None,
                }),
                sep.as_os_str(),
            )
            .fold(OsString::new(), |mut s, v| {
                s.push(v);
                s
            });

            let new_path = outdir.join(new_name);

            if symlink {
                if let Err(err) = ::symlink::symlink_file(entry, &new_path) {
                    ::log::warn!(
                        "could not create symlink '{new_path}' -> '{entry}', {err}",
                        entry = entry.display(),
                        new_path = new_path.display()
                    );
                }
            } else if let Err(err) = fs::hard_link(entry, &new_path) {
                ::log::warn!(
                    "could not create hardlink '{new_path}' -> '{entry}', {err}",
                    new_path = new_path.display(),
                    entry = entry.display()
                );
            }
        });

        Ok(())
    }
}

/// Application entry.
///
/// # Errors
/// If a fatal error occurs or the panic handler cannot be installed.
fn main() -> ::color_eyre::Result<()> {
    ::file_suite_common::start::<Cli>(&["compile_nested"])
}
