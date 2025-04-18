//! Check if a path is valid utf-8.

use ::std::{
    io::{self, Write},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    process::{ExitCode, Termination},
};

use ::clap::Parser;
use ::file_suite_common::ExitCodeError;
use ::rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use ::walkdir::{DirEntry, WalkDir};

/// Check if a path is valid utf-8, and print it if not.
#[derive(Parser)]
#[command(author, version, long_about = None)]
struct Cli {
    /// Path to check.
    #[arg(required = true)]
    path: Vec<PathBuf>,

    /// Recurse into directories.
    #[arg(long, short)]
    recursive: bool,

    /// Terminate results by null character.
    ///
    /// Works but is useless if quiet is specified.
    #[arg(long, short = '0')]
    print0: bool,

    /// Include hidden files in recursive traversal.
    #[arg(long, short)]
    hidden: bool,

    /// Do not print anything.
    #[arg(long, short)]
    quiet: bool,
}

impl ::file_suite_common::Run for Cli {
    type Err = ::file_suite_common::ExitCodeError;

    fn run(self) -> ::core::result::Result<(), Self::Err> {
        let Self {
            path,
            recursive,
            print0,
            quiet,
            hidden,
        } = self;

        if !recursive {
            if quiet {
                for path in path {
                    if path.to_str().is_none() {
                        return Err(ExitCode::FAILURE.into());
                    }
                }
                return Ok(());
            }

            let mut invalid = path
                .into_iter()
                .filter(|path| path.to_str().is_some())
                .peekable();

            if invalid.peek().is_none() {
                return Ok(());
            }

            let mut stdout = io::stdout().lock();
            let term = if print0 { b"\0" } else { b"\n" };
            for invalid in invalid {
                stdout.write_all(invalid.as_os_str().as_bytes()).unwrap();
                stdout.write_all(term).unwrap();
            }

            return Err(ExitCode::FAILURE.into());
        }

        let is_hidden = |entry: &DirEntry| entry.file_name().as_bytes().starts_with(b".");

        let invalid = path
            .into_par_iter()
            .flat_map_iter(|path| {
                WalkDir::new(path)
                    .into_iter()
                    .filter_entry(|e| !hidden && !is_hidden(e))
            })
            .filter_map(|e| {
                let path = e.ok()?.into_path();

                if path.to_str().is_some() {
                    None
                } else {
                    Some(path)
                }
            });

        if quiet {
            if invalid.any(|_| true) {
                return Err(ExitCode::FAILURE.into());
            }
            return Ok(());
        }

        let mut invalid = invalid.collect::<Vec<_>>();

        if invalid.is_empty() {
            return Ok(());
        }

        invalid.par_sort();

        let mut stdout = io::stdout().lock();
        let term = if print0 { b"\0" } else { b"\n" };

        for invalid in invalid {
            stdout.write_all(invalid.as_os_str().as_bytes()).unwrap();
            stdout.write_all(term).unwrap();
        }

        Err(ExitCode::FAILURE.into())
    }
}

/// Application entrypoint.
///
/// # Errors
/// On failure.
fn main() -> ExitCode {
    let Err(err) = ::file_suite_common::start::<Cli>(&["path_is_utf8"]) else {
        return ExitCode::SUCCESS;
    };

    if let Some(ExitCodeError(exit_code)) = err.downcast_ref::<ExitCodeError>() {
        return *exit_code;
    }

    Termination::report(Err::<(), _>(err))
}
