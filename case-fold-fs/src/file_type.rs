//! [::rustix::fs::FileType] wrapper dealing with conversion to [::fuser::FileType].

use ::derive_more::{Deref, From, Into};
use ::rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
};
use ::tap::{Conv, Pipe};

/// Wrapper around file types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref)]
pub struct FileType {
    inner: ::rustix::fs::FileType,
}

impl FileType {
    /// Convert to an [::fuser::FileType].
    pub const fn to_fuser(self) -> Option<::fuser::FileType> {
        Some(match self.inner {
            ::rustix::fs::FileType::RegularFile => ::fuser::FileType::RegularFile,
            ::rustix::fs::FileType::Directory => ::fuser::FileType::Directory,
            ::rustix::fs::FileType::Symlink => ::fuser::FileType::Symlink,
            ::rustix::fs::FileType::Fifo => ::fuser::FileType::NamedPipe,
            ::rustix::fs::FileType::Socket => ::fuser::FileType::Socket,
            ::rustix::fs::FileType::CharacterDevice => ::fuser::FileType::CharDevice,
            ::rustix::fs::FileType::BlockDevice => ::fuser::FileType::BlockDevice,
            ::rustix::fs::FileType::Unknown => return None,
        })
    }

    /// Crate a file type for regular files.
    pub const fn regular_file() -> Self {
        Self {
            inner: ::rustix::fs::FileType::RegularFile,
        }
    }

    /// Crate a file type for directories.
    pub const fn directory() -> Self {
        Self {
            inner: ::rustix::fs::FileType::Directory,
        }
    }

    /// Crate a file type for symlinks.
    pub const fn symlink() -> Self {
        Self {
            inner: ::rustix::fs::FileType::Symlink,
        }
    }

    /// Crate a file type for fifos.
    pub const fn fifo() -> Self {
        Self {
            inner: ::rustix::fs::FileType::Fifo,
        }
    }

    /// Crate a file type for sockets.
    pub const fn socket() -> Self {
        Self {
            inner: ::rustix::fs::FileType::Socket,
        }
    }

    /// Crate a file type for char devices.
    pub const fn character_device() -> Self {
        Self {
            inner: ::rustix::fs::FileType::CharacterDevice,
        }
    }

    /// Crate a file type for block devices.
    pub const fn block_device() -> Self {
        Self {
            inner: ::rustix::fs::FileType::BlockDevice,
        }
    }
}

impl From<::fuser::FileType> for FileType {
    fn from(value: ::fuser::FileType) -> Self {
        let inner = match value {
            ::fuser::FileType::NamedPipe => ::rustix::fs::FileType::Fifo,
            ::fuser::FileType::CharDevice => ::rustix::fs::FileType::CharacterDevice,
            ::fuser::FileType::BlockDevice => ::rustix::fs::FileType::BlockDevice,
            ::fuser::FileType::Directory => ::rustix::fs::FileType::Directory,
            ::fuser::FileType::RegularFile => ::rustix::fs::FileType::RegularFile,
            ::fuser::FileType::Symlink => ::rustix::fs::FileType::Symlink,
            ::fuser::FileType::Socket => ::rustix::fs::FileType::Socket,
        };
        Self { inner }
    }
}

impl ToSql for FileType {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, ::rusqlite::Error> {
        self.inner
            .as_raw_mode()
            .conv::<u64>()
            .cast_signed()
            .pipe(Value::Integer)
            .pipe(ToSqlOutput::Owned)
            .pipe(Ok)
    }
}

impl FromSql for FileType {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        let value = value.as_i64()?;
        value
            .cast_unsigned()
            .try_into()
            .map_err(|_| FromSqlError::OutOfRange(value))
            .map(::rustix::fs::FileType::from_raw_mode)
            .map(Self::from)
    }
}
