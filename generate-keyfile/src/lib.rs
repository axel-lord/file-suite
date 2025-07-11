#![doc = include_str!("../README.md")]

use ::std::{num::NonZero, os::fd::AsRawFd, path::PathBuf, ptr::null_mut};

use ::clap::Parser;
use ::rustix::{
    fs::{
        AtFlags, CWD, FlockOperation, Mode, OFlags, RenameFlags, flock, ftruncate, linkat, open,
        renameat_with, unlink,
    },
    mm::{MapFlags, ProtFlags, mmap, munmap},
};

/// Print error and exit application.
macro_rules! error {
    ($msg:literal) => {{
        $crate::error!($msg,)
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        eprintln!($fmt, $($arg)*);
        ::std::process::exit(1)
    }};
}
use error;

/// Create a file filled with random bytes.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Size of keyfile.
    #[arg(long, short, default_value = "64")]
    size: NonZero<u64>,
    /// Overwrite keyfile if it exists.
    #[arg(long, short)]
    force: bool,
    /// Where to write keyfile to.
    output: PathBuf,
}

impl ::file_suite_common::Run for Cli {
    type Error = ::std::convert::Infallible;

    fn run(self) -> Result<(), Self::Error> {
        let Cli {
            size,
            force,
            output,
        } = self;

        let output_path = std::path::absolute(&output).unwrap_or_else(|err| {
            error!(
                "could not make path '{path}' absolute, {err}",
                path = output.display(),
            )
        });
        let len = usize::try_from(size.get())
            .unwrap_or_else(|err| error!("could not convert {size} from u64 to usize, {err}"));

        if !force
            && output_path.try_exists().unwrap_or_else(|err| {
                error!(
                    "unable to verify if path '{path}' exists or not, {err}",
                    path = output.display()
                )
            })
        {
            error!(
                "output path '{path}' already exists, use --force flag to overwrite",
                path = output.display()
            );
        }

        let parent_path = output_path.parent().unwrap_or_else(|| {
            error!(
                "output path '{path}' should have a parent directory",
                path = output.display()
            )
        });

        let file =
            open(parent_path, OFlags::RDWR | OFlags::TMPFILE, Mode::RUSR).unwrap_or_else(|err| {
                error!(
                    "could not create temporary file in directory '{path}', {err}",
                    path = parent_path.display()
                )
            });

        flock(&file, FlockOperation::NonBlockingLockExclusive).unwrap_or_else(|err| {
            error!(
                "could not aquire exclusive lock for temp file in '{path}', {err}",
                path = parent_path.display()
            )
        });
        ftruncate(&file, size.get()).unwrap_or_else(|err| {
            error!(
                "could not truncate temp file in '{path}', {err}",
                path = parent_path.display()
            )
        });

        // SAFETY: No writer should exists for the file since it is unlinked and locked.
        let map = unsafe {
            mmap(
                null_mut(),
                len,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::POPULATE | MapFlags::SHARED,
                &file,
                0,
            )
        }
        .unwrap_or_else(|err| {
            error!(
                "could not memory temp file in '{path}', {err}",
                path = parent_path.display()
            )
        });

        // SAFETY: The memory mapped file should have the specified length and no other writers.
        ::getrandom::fill(unsafe { std::slice::from_raw_parts_mut(map.cast::<u8>(), len) })
            .unwrap_or_else(|err| {
                error!("failed to generate random bytes when filling temp file, {err}")
            });

        // SAFETY: The same variable for length was used when creating the map.
        unsafe { munmap(map, len) }
            .unwrap_or_else(|err| error!("failed to unmap temp file from memory, {err}"));

        let path = format!("/proc/self/fd/{}", file.as_raw_fd());

        linkat(CWD, &path, CWD, &output_path, AtFlags::SYMLINK_FOLLOW).unwrap_or_else(|err| {
        let out_path = output_path.display();
        with_fmt(
            format_args!("could not link '{path}' to '{out_path}', {err}"),
            |err| {
                if !force {
                    error!("{err}")
                }

                let temp_path = format!(
                    "{parent}/gk.{rng:0>32x}.gk",
                    parent = parent_path.display(),
                    rng = {
                        let mut buf = [0u8; 16];
                        ::getrandom::fill(&mut buf).unwrap_or_else(|err| {
                            error!("{err}\nfailed to generate random bytes, {err}")
                        });
                        u128::from_ne_bytes(buf)
                    },
                );

                linkat(CWD, &path, CWD, &temp_path, AtFlags::SYMLINK_FOLLOW).unwrap_or_else(
                    |link_err| {
                        error!("{err}\nfailed to link '{path}' to '{temp_path}', {link_err}")
                    },
                );

                // Unlink is always performed.
                let res_rename =
                    renameat_with(CWD, &temp_path, CWD, &output_path, RenameFlags::EXCHANGE);
                let res_unlink = unlink(&temp_path);

                match (res_rename, res_unlink) {
                    (Err(err_rename), Err(err_unlink)) => {
                        error!(
                            "{err}\n{err_rename}\n{err_unlink}",
                            err_rename = format_args!(
                                "could not exchange '{temp_path}' with {out_path}, {err_rename}",
                            ),
                            err_unlink =
                                format_args!("could not unlink '{temp_path}', {err_unlink}"),
                        )
                    }
                    (Err(err_rename), _) => {
                        error!(
                            "{err}\n{err_rename}",
                            err_rename = format_args!(
                                "could not exchange '{temp_path}' with {out_path}, {err_rename}",
                            ),
                        )
                    }
                    (_, Err(err_unlink)) => {
                        error!(
                            "{err}\n{err_unlink}",
                            err_unlink =
                                format_args!("could not unlink '{temp_path}', {err_unlink}")
                        )
                    }
                    _ => (),
                }
            },
        )
    });
        flock(&file, FlockOperation::NonBlockingUnlock).unwrap_or_else(|err| {
            error!(
                "failed to unlock fd {fd}, '{path}', {err}",
                fd = file.as_raw_fd(),
                path = output.display()
            )
        });
        Ok(())
    }
}

/// Run a closure giving it some formatted arguments.
fn with_fmt<T>(
    fmt: ::core::fmt::Arguments<'_>,
    f: impl for<'a> FnOnce(::core::fmt::Arguments<'a>) -> T,
) -> T {
    f(fmt)
}
