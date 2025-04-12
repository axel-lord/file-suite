use std::{
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

    /// Do not print anything.
    #[arg(long, short)]
    quiet: bool,
}

fn path_is_valid(path: &Path) -> bool {
    let Some(name) = path.file_name() else {
        return true;
    };
    name.to_str().is_some()
}

fn main() -> ExitCode {
    let Cli {
        path,
        recursive,
        print0,
        quiet,
    } = Cli::parse();

    let mut path_list = Vec::new();

    for path in path {
        if recursive && path.is_dir() {
            let mut path_stack = vec![path];
            while let Some(path) = path_stack.pop() {
                if let Ok(dir) = std::fs::read_dir(&path) {
                    for entry in dir {
                        let Ok(entry) = entry else {
                            continue;
                        };

                        let path = entry.path();

                        if path.is_dir() {
                            path_stack.push(path)
                        } else {
                            path_list.push(path)
                        }
                    }
                }
                path_list.push(path);
            }
        } else {
            path_list.push(path);
        }
    }

    let mut invalid_paths = path_list
        .into_par_iter()
        .filter_map(|path| {
            if path_is_valid(&path) {
                None
            } else {
                Some(path.to_string_lossy().into_owned())
            }
        })
        .collect::<Vec<_>>();
    invalid_paths.sort();

    let code = if invalid_paths.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    };
    let term = if print0 { '\0' } else { '\n' };
    let mut stdout = stdout().lock();
    if !quiet {
        for path in invalid_paths {
            write!(stdout, "{path}{term}").expect("writing to stdout should give success")
        }
    }
    code
}
