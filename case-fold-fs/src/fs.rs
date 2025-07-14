//! [Fs] impl

use ::std::{
    ffi::OsStr,
    os::{fd::BorrowedFd, unix::ffi::OsStrExt},
    path::Path,
    sync::mpsc,
    time::Duration,
};

use ::fuser::FileType;
use ::rusqlite::{Connection, Transaction};
use ::rustix::fs::{Dir, Mode, OFlags};
use ::smallvec::SmallVec;

use crate::{
    Correction, Error,
    action::{
        self, DeleteFromOpendirReaddir, InsertToReaddir,
        param::{InsertParams, LookupParams},
        result::LookupResult,
    },
    case_fold, get_attr,
    macros::{conv_or_reply, db_stmts},
};

db_stmts! {
    DbStmts<'conn> {
        lookup: action::Lookup<'conn>,
        path_by_ino: action::PathByInode<'conn>,
        count_ino: action::CountInodes<'conn>,
        increment_rc: action::IncrementRc<'conn>,
        insert_into_opendir: action::InsertIntoOpendir<'conn>,
        select_readdir: action::SelectReaddir<'conn>,
        forget_ino: action::ForgetInode<'conn>,
        ty_by_ino: action::TypeByInode<'conn>,
        path_ty_by_ino: action::PathTypeByInode<'conn>,
    }
}

/// Filesystem.
#[derive(Debug)]
pub struct Fs<'conn, 'scope> {
    connection: &'conn Connection,
    stmt: DbStmts<'conn>,
    root_dir: BorrowedFd<'scope>,
    root_ino: i64,
    leak: bool,
    correction: &'scope mpsc::Sender<Correction>,
    scope: &'conn ::rayon::Scope<'scope>,
}

impl<'conn, 'scope> Fs<'conn, 'scope> {
    /// Create a new instance.
    pub fn new(
        root_dir: BorrowedFd<'scope>,
        connection: &'conn Connection,
        scope: &'conn ::rayon::Scope<'scope>,
        correction: &'scope mpsc::Sender<Correction>,
    ) -> ::color_eyre::Result<Self> {
        connection.execute_batch(include_str!("./db_setup.sql"))?;

        Ok(Self {
            connection,
            root_ino: connection.last_insert_rowid(),
            stmt: DbStmts::new(connection)?,
            leak: false,
            correction,
            root_dir,
            scope,
        })
    }

    /// If should is true database rows are not deleted when not needed.
    /// file rows get 0 as parent.
    pub fn leak(mut self, should: bool) -> Self {
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

        let mut insert = action::Insert::new(&transaction).map_err(|err| {
            ::log::error!("could not create insert action\n{err}");
            ::libc::EIO
        })?;

        insert
            .perform(InsertParams {
                parent,
                path: b"",
                folded: b".",
                ty: crate::FileType::from(::rustix::fs::FileType::CharacterDevice),
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
            let ty = entry.file_type().into();

            if let Err(err) = insert.perform(InsertParams {
                parent,
                path: &path,
                folded: &folded,
                ty,
            }) {
                let path = Path::new(OsStr::from_bytes(&path));
                match err.sqlite_error_code() {
                    Some(::rusqlite::ErrorCode::ConstraintViolation) => {
                        ::log::warn!(
                            "skipping insert of {path:?} due to constraint violation\n{err}"
                        );
                    }
                    _ => {
                        ::log::error!("could not insert {path:?} into database\n{err}",);
                        return Err(::libc::EIO);
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_populated(&mut self, parent: i64) -> Result<(), i32> {
        if self.has_child_marker(parent)? {
            return Ok(());
        }

        let (dir, _ty) = self.stmt.path_ty_by_ino.perform(parent).map_err(|err| {
            ::log::error!("could not get parent directory {parent}\n{err}");
            ::libc::EIO
        })?;

        self.with_transaction(|transaction| self.populate_dir(parent, &dir, transaction))
    }

    #[inline(never)]
    fn lookup_path_io(&mut self, parent: i64, folded: &[u8]) -> Result<LookupResult, i32> {
        self.ensure_populated(parent)?;

        let lookup_result = self
            .lookup_path_db(parent, folded)
            .transpose()
            .ok_or(::libc::ENOENT)
            .flatten()?;

        Ok(lookup_result)
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

impl<'conn, 'scope> ::fuser::Filesystem for Fs<'conn, 'scope> {
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let LookupResult { ino, path, ty: _ } =
            match self.lookup_path(name.as_bytes(), parent.cast_signed()) {
                Ok(value) => value,
                Err(err) => return reply.error(err),
            };

        if let Err(err) = self.stmt.increment_rc.perform(ino) {
            ::log::error!("could not increase rc for {ino}\n{err}");
            return reply.error(::libc::EIO);
        };

        let attr = match get_attr(self.root_dir, Path::new(OsStr::from_bytes(&path)), ino) {
            Ok(attr) => attr,
            Err(err) => {
                Correction::Rc { ino }.send(self.correction);
                return reply.error(err);
            }
        };

        reply.entry(&Duration::MAX, &attr, 0);
    }

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        ::log::info!("forget - {ino}");
        if let Err(err) = self.stmt.forget_ino.perform(ino.cast_signed(), nlookup) {
            ::log::error!("could not forget ino {ino} nlookup {nlookup}\n{err}");
        }
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

        let fd = self.root_dir;
        self.spawn(move || {
            let path = Path::new(OsStr::from_bytes(&path));

            let attr = match get_attr(fd, path, ino.cast_signed()) {
                Ok(attr) => attr,
                Err(err) => return reply.error(err),
            };

            reply.attr(&Duration::MAX, &attr);
        });
    }

    fn open(&mut self, _req: &fuser::Request<'_>, _ino: u64, _flags: i32, reply: fuser::ReplyOpen) {
        reply.error(::libc::ENOSYS);
    }

    fn opendir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _flags: i32,
        reply: fuser::ReplyOpen,
    ) {
        let ino = ino.cast_signed();

        let ty = match self.stmt.ty_by_ino.perform(ino) {
            Ok(ty) => ty,
            Err(err) => {
                ::log::error!("could not get type of ino {ino}\n{err}");
                return reply.error(::libc::EIO);
            }
        };

        if !ty.is_dir() {
            return reply.error(::libc::ENOTDIR);
        }

        let fh = match self.stmt.insert_into_opendir.perform(ino) {
            Ok(fh) => fh,
            Err(err) => {
                ::log::error!("could not add opendir entry to databas for {ino}\n{err}");
                return reply.error(::libc::EIO);
            }
        };
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
            DeleteFromOpendirReaddir::new(transaction)?.perform(fh.cast_signed())?;
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

                InsertToReaddir::new(transaction)?.perform(fh.cast_signed(), ino.cast_signed())?;

                Ok(())
            });

            if let Err(err) = result {
                ::log::error!("could not create readdir entries for {ino}\n{err}");
                return reply.error(::libc::EIO);
            }
        }

        let mut query = match self.stmt.select_readdir.perform(fh.cast_signed(), offset) {
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
                    ::log::error!("could not get ino for readdir row\n{err}");
                    return reply.error(::libc::EIO);
                }
            };

            let ty = match row.get::<_, crate::FileType>("type") {
                Ok(ty) => ty,
                Err(err) => {
                    ::log::error!("could not get type for readdir row, ino {ino}\n{err}");
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
                    ::log::error!("could not get name for readdir row, ino {ino}\n{err}");
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

            if reply.add(
                ino.cast_unsigned(),
                ino,
                ty.to_fuser().unwrap_or_else(|| FileType::RegularFile),
                name,
            ) {
                break;
            }
        }

        reply.ok();
    }

    fn statfs(&mut self, _req: &fuser::Request<'_>, _ino: u64, reply: fuser::ReplyStatfs) {
        let files = match self.stmt.count_ino.perform() {
            Ok(count) => count,
            Err(err) => return reply.error(err),
        };

        let fd = self.root_dir;
        self.spawn(move || {
            let statfs = match ::rustix::fs::fstatfs(fd) {
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
