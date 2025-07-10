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
use ::fuser::{FileAttr, FileType};
use ::rusqlite::{Connection, named_params};
use ::rustix::fs::{AtFlags, Dir, Mode, OFlags, StatxFlags, makedev, statx};
use ::signal_hook::{
    consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use ::smallvec::SmallVec;

use crate::action::{
    Action,
    param::{InsertParams, LookupParams},
    result::{DirectoryResult, LookupResult},
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
fn get_attr(fd: BorrowedFd<'_>, path: &Path) -> Result<FileAttr, i32> {
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
        ino: statx.stx_ino,
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
}

impl ::file_suite_common::Run for Cli {
    type Error = ::color_eyre::Report;

    fn run(self) -> Result<(), Self::Error> {
        let Self { source, mountpoint } = self;
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
                Fs::new(root_dir.as_fd(), &connection, r)?,
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
    directory: Action<'conn, action::Directory>,
    path_by_ino: Action<'conn, action::PathByInode>,
    count_ino: Action<'conn, action::CountInodes>,
}

impl<'conn> DbStmts<'conn> {
    pub fn new(connection: &'conn Connection) -> ::color_eyre::Result<Self> {
        Ok(Self {
            lookup: Action::new(connection)?,
            count_ino: Action::new(connection)?,
            path_by_ino: Action::new(connection)?,
            directory: Action::new(connection)?,
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

/// Filesystem.
#[derive(Debug)]
struct Fs<'con, 'scope> {
    connection: &'con Connection,
    stmt: DbStmts<'con>,
    root_dir: BorrowedFd<'con>,
    shared: Arc<Shared>,
    root_ino: i64,
    scope: &'con ::rayon::Scope<'scope>,
}

impl<'con, 'scope> Fs<'con, 'scope> {
    pub fn new(
        root_dir: BorrowedFd<'con>,
        connection: &'con Connection,
        scope: &'con ::rayon::Scope<'scope>,
    ) -> ::color_eyre::Result<Self> {
        let create_table = r#"
            CREATE TABLE files (
                ino INTEGER PRIMARY KEY,
                parent INTEGER NOT NULL,
                name BLOB NOT NULL,
                folded BLOB NOT NULL,
                has_children INTEGER,
                UNIQUE (parent, folded),
                FOREIGN KEY (parent)
                    REFERENCES files (ino)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            )
        "#;
        connection.execute(create_table, [])?;
        connection.execute(
            r#"INSERT INTO files (ino, parent, name, folded) VALUES (0, 0, "-", "-")"#,
            [],
        )?;
        connection.execute(
            r#"INSERT INTO files (ino, parent, name, folded) VALUES (:ino, 0, "", "")"#,
            named_params! {":ino": ::fuser::FUSE_ROOT_ID},
        )?;
        Ok(Self {
            connection,
            shared: Arc::new(Shared::new(root_dir)?),
            root_ino: connection.last_insert_rowid(),
            stmt: DbStmts::new(connection)?,
            root_dir,
            scope,
        })
    }

    fn spawn(&self, f: impl FnOnce() + Send + 'scope) {
        self.scope.spawn(move |_| f());
    }

    #[inline(never)]
    fn lookup_path_io(
        &mut self,
        name: &[u8],
        parent: i64,
        folded_name: &[u8],
    ) -> Result<LookupResult, i32> {
        let dir = match self.stmt.directory.perform(parent) {
            Ok(dir) => dir,
            Err(err) => {
                ::log::error!("could not get parent directory {parent}\n{err}");
                return Err(::libc::EIO);
            }
        };

        if dir.has_children.is_some() {
            return Err(::libc::ENOENT);
        }

        let dir_path = Path::new(OsStr::from_bytes(&dir.path));
        let dir_entries = if parent == self.root_ino {
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
                ::log::error!(
                    "could not open {dir_path:?}\n{err}\n({parent}, {name:?})",
                    name = OsStr::from_bytes(name)
                );
                err.raw_os_error()
            })
        }
        .and_then(|fd| {
            Dir::new(fd).map_err(|err| {
                ::log::error!("could not read {dir_path:?} as a directory\n{err}");
                err.raw_os_error()
            })
        })?;

        let transaction = self.connection.unchecked_transaction().map_err(|err| {
            ::log::error!("could not start transaction\n{err}");
            ::libc::EIO
        })?;

        let mut insert = Action::<action::Insert>::new(&transaction).map_err(|err| {
            ::log::error!("could not create insert action\n{err}");
            ::libc::EIO
        })?;

        let mut has_children = false;
        let mut result = Err(::libc::ENOENT);
        for entry in dir_entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    ::log::error!("could not get entry of {dir_path:?}\n{err}");
                    continue;
                }
            };
            let name = entry.file_name().to_bytes();

            if matches!(name, b"." | b"..") {
                continue;
            }

            has_children = true;

            let mut path = dir.path.clone();
            path.push(b'/');
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
            if folded.as_slice() == folded_name {
                let ino = self.connection.last_insert_rowid();
                result = Ok(LookupResult { ino, path });
            }
        }

        let mut set_has_children =
            Action::<action::SetHasChildren>::new(&transaction).map_err(|err| {
                ::log::error!("could not create set_has_children action\n{err}");
                ::libc::EIO
            })?;

        set_has_children
            .perform((parent, has_children))
            .map_err(|err| {
                ::log::error!("could not set has_children of {dir_path:?}\n{err}");
                ::libc::EIO
            })?;

        drop(insert);
        drop(set_has_children);

        transaction.commit().map_err(|err| {
            ::log::error!("could not commit transaction\n{err}");
            ::libc::EIO
        })?;

        result
    }

    /// Create an iterator that populates a parent directory with children and yields case folded
    /// file names of the children as it does so.
    fn populate_dir(
        parent: i64,
        root_ino: i64,
        root_dir: BorrowedFd<'_>,
        dir: DirectoryResult,
        connection: &Connection,
    ) -> Result<impl Iterator<Item = Result<SmallVec<[u8; 64]>, i32>>, i32> {
        let dir_path = Path::new(OsStr::from_bytes(&dir.path));

        let mut insert = Action::<action::Insert>::new(connection).map_err(|err| {
            ::log::error!("could not create insert action\n{err}");
            ::libc::EIO
        })?;

        let mut set_has_children =
            Action::<action::SetHasChildren>::new(connection).map_err(|err| {
                ::log::error!("could not create set_has_children action\n{err}");
                ::libc::EIO
            })?;

        let entries = if parent == root_ino {
            ::rustix::io::fcntl_dupfd_cloexec(root_dir, 0).map_err(|err| {
                ::log::error!("could not open root dir\n{err}");
                err.raw_os_error()
            })
        } else {
            ::rustix::fs::openat(
                root_dir,
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

        let mut has_children = false;
        Ok(entries.filter_map(move |entry| {
            let entry = entry
                .map_err(|err| {
                    ::log::error!(
                        "could not get entry of {path:?}\n{err}",
                        path = Path::new(OsStr::from_bytes(&dir.path))
                    )
                })
                .ok()?;
            let name = entry.file_name().to_bytes();

            if matches!(name, b"." | b"..") {
                return None;
            }

            if !has_children {
                if let Err(err) = set_has_children.perform((parent, true)) {
                    ::log::error!(
                        "could not set has_children of {path:?}\n{err}",
                        path = Path::new(OsStr::from_bytes(&dir.path))
                    );
                    return Some(Err(::libc::EIO));
                }
                has_children = true;
            }

            let mut path = dir.path.clone();
            path.push(b'/');
            path.extend_from_slice(name);

            let path = path;
            let folded = case_fold(name);

            if let Err(err) = insert.perform(InsertParams {
                parent,
                path: &path,
                folded: &folded,
            }) {
                ::log::error!("could not insert {path:?} into database\n{err}");
                return Some(Err(::libc::EIO));
            }

            Some(Ok(folded))
        }))
    }

    fn lookup_path(&mut self, name: &[u8], parent: i64) -> Result<LookupResult, i32> {
        let folded = case_fold(name);

        let err = match self.stmt.lookup.perform(LookupParams {
            parent,
            folded: &folded,
        }) {
            Ok(val) => return Ok(val),
            Err(err) => err,
        };

        if !matches!(err, ::rusqlite::Error::QueryReturnedNoRows) {
            ::log::error!("database error unrelated to not finding anything\n{err}");
            return Err(::libc::EIO);
        }

        self.lookup_path_io(name, parent, &folded)
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

            let attr = match get_attr(shared.root_dir.as_fd(), path) {
                Ok(attr) => attr,
                Err(err) => return reply.error(err),
            };

            reply.entry(
                &Duration::MAX,
                &FileAttr {
                    ino: ino.cast_unsigned(),
                    ..attr
                },
                0,
            );
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

            let mut attr = match get_attr(shared.root_dir.as_fd(), path) {
                Ok(attr) => attr,
                Err(err) => return reply.error(err),
            };
            attr.ino = ino;

            reply.attr(&Duration::MAX, &attr);
        });
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: fuser::ReplyDirectory,
    ) {
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
