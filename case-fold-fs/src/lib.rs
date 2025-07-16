//! A case folded bind mount.

use ::std::{
    any::type_name,
    ffi::OsStr,
    fmt::{Debug, Display},
    os::{fd::BorrowedFd, unix::ffi::OsStrExt},
    path::Path,
    time::{Duration, SystemTime},
};

use ::derive_more::IsVariant;
use ::fuser::FileAttr;
use ::rusqlite::{Connection, Transaction, types::FromSqlError};
use ::rustix::fs::{AtFlags, Mode, StatxFlags, makedev, statx};
use ::smallvec::SmallVec;
use ::tap::{Conv, Pipe};

use crate::file_type::FileType;
pub use crate::{cli::Cli, fs::Fs};

pub mod action;
pub mod file_type;

mod cli;
mod fs;
mod macros;

/// Re-exported type for convenience.
pub type Buf = SmallVec<[u8; 64]>;

/// Run a function producing a result in a transaction. Commiting on success.
fn with_transaction<T, E, F>(connection: &Connection, f: F) -> Result<T, E>
where
    F: for<'a> FnOnce(&'a Transaction<'a>) -> Result<T, E>,
    E: From<i32>,
{
    connection
        .unchecked_transaction()
        .map_err(|err| {
            ::log::error!("could not start transaction\n{err}");
            E::from(::libc::EIO)
        })
        .and_then(|transaction| {
            let result = f(&transaction)?;
            transaction.commit().map_err(|err| {
                ::log::error!("could not commit transaction\n{err}");
                ::libc::EIO
            })?;
            Ok(result)
        })
}

/// Convert a value, logging and converting errors to eio.
fn log_conv<T, V>(value: T) -> Result<V, i32>
where
    V: TryFrom<T>,
    T: Copy + Display,
    V::Error: Display,
{
    V::try_from(value).map_err(|err| {
        ::log::error!(
            "could not convert value {value} of type {} to type {}\n{err}",
            type_name::<T>(),
            type_name::<V>()
        );
        ::libc::EIO
    })
}

/// Get a path from a byte slice.
fn path_from_bytes(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}

/// Get file attributes.
fn get_attr(fd: BorrowedFd<'_>, path: &Path, ino: i64) -> Result<FileAttr, i32> {
    let statx = statx(fd, path, AtFlags::EMPTY_PATH, StatxFlags::BASIC_STATS).map_err(|err| {
        ::log::error!("could not stat {path:?}\n{err}");
        err.raw_os_error()
    })?;

    let kind = statx
        .stx_mode
        .conv::<::rustix::fs::RawMode>()
        .pipe(::rustix::fs::FileType::from_raw_mode)
        .conv::<FileType>()
        .to_fuser()
        .unwrap_or_else(|| ::fuser::FileType::RegularFile);

    Ok(FileAttr {
        ino: ino.cast_unsigned(),
        size: statx.stx_size,
        blocks: statx.stx_blocks,
        atime: SystemTime::UNIX_EPOCH + Duration::from_secs(log_conv(statx.stx_atime.tv_sec)?),
        mtime: SystemTime::UNIX_EPOCH + Duration::from_secs(log_conv(statx.stx_mtime.tv_sec)?),
        ctime: SystemTime::UNIX_EPOCH + Duration::from_secs(log_conv(statx.stx_ctime.tv_sec)?),
        crtime: SystemTime::UNIX_EPOCH,
        kind,
        perm: log_conv(Mode::from_raw_mode(statx.stx_mode.into()).as_raw_mode())?,
        nlink: statx.stx_nlink,
        uid: statx.stx_uid,
        gid: statx.stx_gid,
        rdev: log_conv(makedev(statx.stx_rdev_major, statx.stx_rdev_minor))?,
        blksize: statx.stx_blksize,
        flags: 0,
    })
}

fn case_fold(bytes: &[u8]) -> Buf {
    let mut folded = SmallVec::new();

    let mut buf = [0u8; 4];
    for chunk in bytes.utf8_chunks() {
        for chr in chunk.valid().chars().flat_map(|c| c.to_uppercase()) {
            folded.extend_from_slice(chr.encode_utf8(&mut buf).as_bytes());
        }
        folded.extend_from_slice(chunk.invalid());
    }

    folded
}

/// Error type.
#[derive(Debug, ::thiserror::Error, IsVariant)]
pub enum Error {
    /// Raw os error.
    #[error("raw error {0}")]
    Raw(i32),
    /// Rustix errno.
    #[error(transparent)]
    Errno(#[from] ::rustix::io::Errno),
    /// Sqlite error.
    #[error(transparent)]
    Sqlite(#[from] ::rusqlite::Error),
}

impl From<i32> for Error {
    fn from(value: i32) -> Self {
        Self::Raw(value)
    }
}

impl From<FromSqlError> for Error {
    fn from(value: FromSqlError) -> Self {
        Self::Sqlite(value.into())
    }
}

impl Error {
    /// Run a function if the value is not a raw i32.
    pub fn inspect_not_raw(self, f: impl for<'a> FnOnce(&'a Self)) -> Self {
        f(&self);
        self
    }

    /// Convert into a raw error, sqlite errors become [::libc::EIO].
    pub fn into_raw(self) -> i32 {
        match self {
            Error::Raw(raw) => raw,
            Error::Errno(errno) => errno.raw_os_error(),
            Error::Sqlite(_) => ::libc::EIO,
        }
    }

    /// Get an eio error
    pub fn eio() -> Self {
        Self::Raw(::libc::EIO)
    }
}

/// Proxy for debug printing by closure
pub struct DbgFn<F>(pub F)
where
    F: Fn(&mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result;

impl<F> Debug for DbgFn<F>
where
    F: Fn(&mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self.0)(f)
    }
}
