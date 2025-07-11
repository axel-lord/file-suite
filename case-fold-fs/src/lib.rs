//! A case folded bind mount.

use ::std::{
    any::type_name,
    ffi::OsStr,
    fmt::Display,
    os::{
        fd::{AsFd, BorrowedFd, OwnedFd},
        unix::ffi::OsStrExt,
    },
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};

pub mod action;

use ::clap::Parser;
use ::color_eyre::eyre::eyre;
use ::derive_more::IsVariant;
use ::fuser::{FileAttr, FileType};
use ::rusqlite::{
    Connection, DatabaseName, Transaction, fallible_streaming_iterator::FallibleStreamingIterator,
    types::FromSqlError,
};
use ::rustix::fs::{AtFlags, Dir, Mode, OFlags, StatxFlags, makedev, statx};
use ::signal_hook::{
    consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use ::smallvec::SmallVec;

use crate::action::{
    Action,
    param::{InsertParams, LookupParams},
    result::LookupResult,
};

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

/// Get file attributes.
fn get_attr(fd: BorrowedFd<'_>, path: &Path, ino: i64) -> Result<FileAttr, i32> {
    let statx = statx(fd, path, AtFlags::EMPTY_PATH, StatxFlags::BASIC_STATS).map_err(|err| {
        ::log::error!("could not stat {path:?}\n{err}");
        err.raw_os_error()
    })?;

    let file_type = ::rustix::fs::FileType::from_raw_mode(statx.stx_mode.into());
    let kind = if file_type.is_file() {
        FileType::RegularFile
    } else if file_type.is_dir() {
        FileType::Directory
    } else if file_type.is_symlink() {
        FileType::Symlink
    } else if file_type.is_fifo() {
        FileType::NamedPipe
    } else if file_type.is_char_device() {
        FileType::CharDevice
    } else if file_type.is_block_device() {
        FileType::BlockDevice
    } else if file_type.is_socket() {
        FileType::Socket
    } else {
        ::log::error!("unkown file type {file_type:?} of {path:?}");
        return Err(::libc::EIO);
    };

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

/// Mount a directory as case folded.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Source directory.
    source: PathBuf,
    /// Mount point, will be same as source if not given.
    mountpoint: Option<PathBuf>,

    /// Dump internal database to specified file.
    #[arg(long)]
    dump: Option<PathBuf>,

    /// Do not destroy database contents on deletion.
    #[arg(long)]
    leak: bool,
}

impl ::file_suite_common::Run for Cli {
    type Error = ::color_eyre::Report;

    fn run(self) -> Result<(), Self::Error> {
        let Self {
            source,
            mountpoint,
            dump,
            leak,
        } = self;
        let signal_kinds = &[SIGHUP, SIGTERM, SIGINT, SIGQUIT];
        let mut signals = Signals::new(signal_kinds).map_err(|err| eyre!(err))?;
        let signals_handle = signals.handle();

        let mut signals = signals.forever();

        let root_dir = ::rustix::fs::open(
            &source,
            OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::RDONLY,
            Mode::empty(),
        )
        .map_err(|err| eyre!(err))?;

        ::rayon::scope(|r| -> ::color_eyre::Result<()> {
            let connection = ::rusqlite::Connection::open_in_memory().map_err(|err| eyre!(err))?;
            let mut session = ::fuser::Session::new(
                Fs::new(root_dir.as_fd(), &connection, r)?.leak(leak),
                mountpoint.unwrap_or_else(|| source.clone()),
                &[],
            )
            .map_err(|err| eyre!(err))?;
            let mut unmounter = session.unmount_callable();

            thread::scope(|s| -> ::color_eyre::Result<()> {
                thread::Builder::new()
                    .name("case-fold-fs-signal-handler".into())
                    .spawn_scoped(s, || {
                        for signal in &mut signals {
                            ::log::info!("received signal {signal:?}");
                            break;
                        }

                        if let Err(err) = unmounter.unmount() {
                            ::log::error!("failed when unmounting, {err}");
                        }

                        for signal in &mut signals {
                            ::log::error!("received signal {signal:?}, terminating immediatly");
                            ::std::process::exit(1);
                        }
                    })
                    .map_err(|err| eyre!(err))?;

                let result = session.run();

                signals_handle.close();
                result.map_err(|err| eyre!(err))
            })?;

            let mut stmt = connection.prepare(r#"SELECT ino, parent, name, folded FROM files"#)?;
            let mut query = stmt.query([])?;

            while let Some(row) = query.next()? {
                let ino = row.get_ref(0)?.as_i64()?;
                let parent = row.get_ref(1)?.as_i64()?;
                let name = OsStr::from_bytes(row.get_ref(2)?.as_bytes()?);
                let folded = OsStr::from_bytes(row.get_ref(3)?.as_bytes()?);

                ::log::info!(
                    "db row - ino: {ino}, parent: {parent}, name: {name}, folded: {folded}",
                    name = name.display(),
                    folded = folded.display()
                );
            }

            if let Some(dump) = dump {
                ::std::fs::write(dump, &*connection.serialize(DatabaseName::Main)?)?;
            }

            Ok(())
        })
    }
}

fn case_fold(bytes: &[u8]) -> SmallVec<[u8; 64]> {
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

#[derive(Debug)]
struct DbStmts<'conn> {
    lookup: Action<'conn, action::Lookup>,
    path_by_ino: Action<'conn, action::PathByInode>,
    count_ino: Action<'conn, action::CountInodes>,
}

impl<'conn> DbStmts<'conn> {
    pub fn new(connection: &'conn Connection) -> ::color_eyre::Result<Self> {
        Ok(Self {
            lookup: Action::new(connection)?,
            count_ino: Action::new(connection)?,
            path_by_ino: Action::new(connection)?,
        })
    }
}

/// Static resources that may be shared.
#[derive(Debug)]
struct Shared {
    root_dir: OwnedFd,
}

impl Shared {
    pub fn new(root_dir: BorrowedFd<'_>) -> ::color_eyre::Result<Self> {
        Ok(Self {
            root_dir: root_dir.try_clone_to_owned().map_err(|err| eyre!(err))?,
        })
    }
}

macro_rules! conv_or_reply {
    ($reply:ident, $value:expr, $ty:ty) => {
        match <$ty>::try_from($value) {
            Ok(val) => val,
            Err(err) => {
                ::log::error!(
                    "could not convert {} {} to type {}\n{err}",
                    stringify!($value),
                    $value,
                    stringify!($ty)
                );
                return $reply.error(::libc::EIO);
            }
        }
    };
}

/// Error type.
#[derive(Debug, ::thiserror::Error, IsVariant)]
enum Error {
    #[error("raw error {0}")]
    Raw(i32),
    #[error(transparent)]
    Errno(#[from] ::rustix::io::Errno),
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
    fn inspect_not_raw(self, f: impl for<'a> FnOnce(&'a Self)) -> Self {
        f(&self);
        self
    }

    /// Convert into a raw error, sqlite errors become [::libc::EIO].
    fn into_raw(self) -> i32 {
        match self {
            Error::Raw(raw) => raw,
            Error::Errno(errno) => errno.raw_os_error(),
            Error::Sqlite(_) => ::libc::EIO,
        }
    }

    /// Get an eio error
    fn eio() -> Self {
        Self::Raw(::libc::EIO)
    }
}

/// Filesystem.
#[derive(Debug)]
struct Fs<'con, 'scope> {
    connection: &'con Connection,
    stmt: DbStmts<'con>,
    root_dir: BorrowedFd<'con>,
    shared: Arc<Shared>,
    root_ino: i64,
    leak: bool,
    scope: &'con ::rayon::Scope<'scope>,
}

impl<'con, 'scope> Fs<'con, 'scope> {
    pub fn new(
        root_dir: BorrowedFd<'con>,
        connection: &'con Connection,
        scope: &'con ::rayon::Scope<'scope>,
    ) -> ::color_eyre::Result<Self> {
        connection.execute(
            r#"
                CREATE TABLE files (
                    ino INTEGER PRIMARY KEY,
                    parent INTEGER NOT NULL,
                    name BLOB NOT NULL,
                    folded BLOB NOT NULL,
                    UNIQUE (parent, folded),
                    FOREIGN KEY (parent)
                        REFERENCES files (ino)
                            ON DELETE CASCADE
                            ON UPDATE CASCADE
                )
            "#,
            [],
        )?;
        connection.execute(
            r#"
                CREATE TABLE opendir (
                    fh INTEGER PRIMARY KEY,
                    ino INTEGER NOT NULL,
                    FOREIGN KEY (ino)
                        REFERENCES files (ino)
                            ON DELETE CASCADE
                            ON UPDATE CASCADE
                )
            "#,
            [],
        )?;
        connection.execute(
            r#"
                CREATE TABLE readdir (
                    fh INTEGER NOT NULL,
                    ino INTEGER NOT NULL,
                    name BLOB NOT NULL,
                    UNIQUE (fh, ino)
                        ON CONFLICT REPLACE,
                    FOREIGN KEY (fh)
                        REFERENCES opendir (fh)
                            ON DELETE CASCADE
                            ON UPDATE CASCADE,
                    FOREIGN KEY (ino)
                        REFERENCES files (ino)
                            ON DELETE CASCADE
                            ON UPDATE CASCADE
                )
            "#,
            [],
        )?;
        connection.execute(
            r#"INSERT INTO files (ino, parent, name, folded) VALUES (0, 0, "-", "-")"#,
            [],
        )?;
        connection.execute(
            r#"INSERT INTO files (ino, parent, name, folded) VALUES (?1, 0, "", "")"#,
            (&::fuser::FUSE_ROOT_ID,),
        )?;

        Ok(Self {
            connection,
            shared: Arc::new(Shared::new(root_dir)?),
            root_ino: connection.last_insert_rowid(),
            stmt: DbStmts::new(connection)?,
            leak: false,
            root_dir,
            scope,
        })
    }

    fn leak(mut self, should: bool) -> Self {
        self.leak = should;
        self
    }

    fn spawn(&self, f: impl FnOnce() + Send + 'scope) {
        self.scope.spawn(move |_| f());
    }

    fn with_transaction<T, E, F>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(&'a Transaction<'a>) -> Result<T, E>,
        E: From<i32>,
    {
        self.connection
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

    /// Create an iterator that populates a parent directory with children and yields case folded
    /// file names of the children as it does so.
    fn populate_dir(
        &self,
        parent: i64,
        dir: &[u8],
        transaction: &Transaction<'_>,
    ) -> Result<(), i32> {
        let dir_path = Path::new(OsStr::from_bytes(dir));

        let mut insert = Action::<action::Insert>::new(&transaction).map_err(|err| {
            ::log::error!("could not create insert action\n{err}");
            ::libc::EIO
        })?;

        insert
            .perform(InsertParams {
                parent,
                path: b"",
                folded: b".",
            })
            .map_err(|err| {
                ::log::error!(
                    "could not insert child marker for {path:?}\n{err}",
                    path = Path::new(OsStr::from_bytes(dir))
                );
                ::libc::EIO
            })?;

        let entries = if parent == self.root_ino {
            ::rustix::io::fcntl_dupfd_cloexec(self.root_dir, 0).map_err(|err| {
                ::log::error!("could not open root dir\n{err}");
                err.raw_os_error()
            })
        } else {
            ::rustix::fs::openat(
                self.root_dir,
                dir_path,
                OFlags::DIRECTORY | OFlags::RDONLY | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|err| {
                ::log::error!("could not open {dir_path:?}\n{err}",);
                err.raw_os_error()
            })
        }
        .and_then(|fd| {
            Dir::new(fd).map_err(|err| {
                ::log::error!("could not read {dir_path:?} as a directory\n{err}");
                err.raw_os_error()
            })
        })?;

        for entry in entries {
            let entry = match entry {
                Err(err) => {
                    ::log::error!(
                        "could not get entry of {path:?}\n{err}",
                        path = Path::new(OsStr::from_bytes(dir))
                    );
                    continue;
                }
                Ok(entry) => entry,
            };
            let name = entry.file_name().to_bytes();
            if matches!(name, b"." | b"..") {
                continue;
            }

            let mut path = SmallVec::<[u8; 64]>::from_slice(dir);
            if !path.is_empty() {
                path.push(b'/');
            }
            path.extend_from_slice(name);

            let path = path;
            let folded = case_fold(name);

            insert
                .perform(InsertParams {
                    parent,
                    path: &path,
                    folded: &folded,
                })
                .map_err(|err| {
                    ::log::error!("could not insert {path:?} into database\n{err}");
                    ::libc::EIO
                })?;
        }
        Ok(())
    }

    #[inline(never)]
    fn lookup_path_io(&mut self, parent: i64, folded: &[u8]) -> Result<LookupResult, i32> {
        match self.lookup_path_db(parent, b".").inspect_err(|_| {
            ::log::error!("could not check for directory child marker");
        })? {
            Some(_) => return Err(::libc::ENOENT),
            None => {}
        }

        let dir = match self.stmt.path_by_ino.perform(parent) {
            Ok(dir) => dir,
            Err(err) => {
                ::log::error!("could not get parent directory {parent}\n{err}");
                return Err(::libc::EIO);
            }
        };

        self.with_transaction(|transaction| self.populate_dir(parent, &dir, transaction))?;

        self.lookup_path_db(parent, folded)
            .transpose()
            .ok_or(::libc::ENOENT)
            .flatten()
    }

    fn lookup_path_db(&mut self, parent: i64, folded: &[u8]) -> Result<Option<LookupResult>, i32> {
        match self.stmt.lookup.perform(LookupParams { parent, folded }) {
            Ok(result) => Ok(Some(result)),
            Err(::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => {
                ::log::error!(
                    "database error on lookup of {folded} for {parent}\n{err}",
                    folded = OsStr::from_bytes(folded).display()
                );
                Err(::libc::EIO)
            }
        }
    }

    fn has_child_marker(&mut self, parent: i64) -> Result<bool, i32> {
        self.lookup_path_db(parent, b".")
            .map(|result| result.is_some())
    }

    fn lookup_path(&mut self, name: &[u8], parent: i64) -> Result<LookupResult, i32> {
        if matches!(name, b"." | b"..") {
            return Err(::libc::ENOENT);
        }

        let folded = case_fold(name);

        match self.lookup_path_db(parent, &folded)? {
            Some(result) => Ok(result),
            None => self.lookup_path_io(parent, &folded),
        }
    }
}

impl ::fuser::Filesystem for Fs<'_, '_> {
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let LookupResult { ino, path } =
            match self.lookup_path(name.as_bytes(), parent.cast_signed()) {
                Ok(value) => value,
                Err(err) => return reply.error(err),
            };

        let shared = self.shared.clone();
        self.spawn(move || {
            let path = Path::new(OsStr::from_bytes(&path));

            let attr = match get_attr(shared.root_dir.as_fd(), path, ino) {
                Ok(attr) => attr,
                Err(err) => return reply.error(err),
            };

            reply.entry(&Duration::MAX, &attr, 0);
        });
    }

    fn getattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: Option<u64>,
        reply: fuser::ReplyAttr,
    ) {
        let path = match self.stmt.path_by_ino.perform(ino.cast_signed()) {
            Ok(path) => path,
            Err(err) => {
                ::log::error!("could not get ino {ino} from database\n{err}");
                return reply.error(::libc::EIO);
            }
        };

        let shared = self.shared.clone();
        self.spawn(move || {
            let path = Path::new(OsStr::from_bytes(&path));

            let attr = match get_attr(shared.root_dir.as_fd(), path, ino.cast_signed()) {
                Ok(attr) => attr,
                Err(err) => return reply.error(err),
            };

            reply.attr(&Duration::MAX, &attr);
        });
    }

    fn opendir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _flags: i32,
        reply: fuser::ReplyOpen,
    ) {
        if let Err(err) = self
            .connection
            .execute(r"INSERT INTO opendir (ino) VALUES (?1) ", (&ino,))
        {
            ::log::error!("could not add opendir entry to databas for {ino}\n{err}");
            return reply.error(::libc::EIO);
        };
        let fh = self.connection.last_insert_rowid();
        reply.opened(fh.cast_unsigned(), 0);
    }

    fn releasedir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        if self.leak {
            return reply.ok();
        }
        let result = self.with_transaction::<_, Error, _>(|transaction| {
            transaction.execute(r"DELETE FROM opendir WHERE fh = ?1", (&fh.cast_signed(),))?;
            transaction.execute(r"DELETE FROM readdir WHERE fh = ?1", (&fh.cast_signed(),))?;
            Ok(())
        });
        if let Err(err) = result {
            reply.error(
                err.inspect_not_raw(|err| {
                    ::log::error!("could not close directory {ino}/{fh} ino/fh\n{err}")
                })
                .into_raw(),
            );
        } else {
            reply.ok();
        }
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        if offset == 0 {
            let should_populate = self
                .has_child_marker(ino.cast_signed())
                .map_err(Error::Raw)
                .and_then(|value| {
                    Ok((!value)
                        .then(|| self.stmt.path_by_ino.perform(ino.cast_signed()))
                        .transpose()?)
                });

            let should_populate = match should_populate {
                Ok(value) => value,
                Err(err) => {
                    return reply.error(
                        err.inspect_not_raw(|err| {
                            ::log::error!(
                                "could not check if dir population should be performed\n{err}"
                            )
                        })
                        .into_raw(),
                    );
                }
            };

            let result = self.with_transaction::<_, Error, _>(|transaction| {
                if let Some(dir) = should_populate {
                    self.populate_dir(ino.cast_signed(), &dir, transaction)?;
                }
                transaction.execute(
                    r#"
                        INSERT INTO readdir (ino, fh, name)
                        SELECT ino, ?1, name
                            FROM files
                            WHERE parent = ?2 AND folded != ?3;
                    "#,
                    (&fh.cast_signed(), &ino.cast_signed(), b"."),
                )?;

                Ok(())
            });

            if let Err(err) = result {
                ::log::error!("could not create readdir entries for {ino}\n{err}");
                return reply.error(::libc::EIO);
            }
        }

        let mut stmt = match self.connection.prepare(
            r#"
                SELECT ino, name
                    FROM readdir
                    WHERE fh = ?1 AND ino > ?2
                    ORDER BY ino
            "#,
        ) {
            Ok(stmt) => stmt,
            Err(err) => {
                ::log::error!("could not prepare readdir statement for {ino}\n{err}");
                return reply.error(::libc::EIO);
            }
        };

        let mut query = match stmt.query((&fh, &offset)) {
            Ok(query) => query,
            Err(err) => {
                ::log::error!("could not get query for readdir statement for {ino}\n{err}");
                return reply.error(::libc::EIO);
            }
        };

        while let Some(row) = match query.next() {
            Ok(row) => row,
            Err(err) => {
                ::log::error!("could not read row of {ino}\n{err}");
                return reply.error(::libc::EIO);
            }
        } {
            let ino = match row.get::<_, i64>("ino") {
                Ok(ino) => ino,
                Err(err) => {
                    ::log::error!("\n{err}");
                    return reply.error(::libc::EIO);
                }
            };

            let name = match row
                .get_ref("name")
                .and_then(|r| Ok(r.as_bytes()?))
                .map(OsStr::from_bytes)
                .map(Path::new)
            {
                Ok(name) => name,
                Err(err) => {
                    ::log::error!("\n{err}");
                    return reply.error(::libc::EIO);
                }
            };

            let name = match name.file_name() {
                Some(name) => name,
                None => {
                    ::log::error!("could not get file name for {name:?}");
                    return reply.error(::libc::EIO);
                }
            };

            if reply.add(ino.cast_unsigned(), ino, FileType::RegularFile, name) {
                break;
            }
        }

        reply.ok();
    }

    fn statfs(&mut self, _req: &fuser::Request<'_>, _ino: u64, reply: fuser::ReplyStatfs) {
        let files = match self.stmt.count_ino.perform(()) {
            Ok(count) => count,
            Err(err) => return reply.error(err),
        };

        let shared = self.shared.clone();
        self.spawn(move || {
            let statfs = match ::rustix::fs::fstatfs(shared.root_dir.as_fd()) {
                Ok(statfs) => statfs,
                Err(err) => {
                    return reply.error(err.raw_os_error());
                }
            };

            let ffree = u64::MAX - files;
            let bsize = conv_or_reply!(reply, statfs.f_bsize, u32);
            let namelen = conv_or_reply!(reply, statfs.f_namelen, u32);
            let frsize = conv_or_reply!(reply, statfs.f_frsize, u32);

            reply.statfs(
                statfs.f_blocks,
                statfs.f_bfree,
                statfs.f_bavail,
                files,
                ffree,
                bsize,
                namelen,
                frsize,
            );
        });
    }
}
