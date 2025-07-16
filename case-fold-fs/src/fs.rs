//! [Fs] impl

use ::std::{
    ffi::OsStr,
    ops::Not,
    os::{
        fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd},
        unix::ffi::OsStrExt,
    },
    path::Path,
    time::{Duration, SystemTime},
};

use ::dashmap::DashMap;
use ::fuser::{FileAttr, FileType};
use ::rusqlite::{Connection, Transaction};
use ::rustc_hash::FxBuildHasher;
use ::rustix::{
    fs::{AtFlags, Dir, Mode, OFlags, RenameFlags},
    process::{getgid, getuid},
};
use ::smallvec::SmallVec;

use crate::{
    DbgFn, Error,
    action::{
        self, CountInodes, DeleteFromOpendir, EntryExists, ForgetInode, IncrementRc, Insert,
        InsertIntoOpendir, InsertIntoReaddir, IsEmpty, Lookup, PathByInode, SelectReaddir,
        TypeByInode, result::LookupResult,
    },
    case_fold, get_attr, log_conv,
    macros::{conv_or_reply, db_stmts},
    path_from_bytes,
};

db_stmts! {
    DbStmts<'conn> {
        lookup: Lookup<'conn>,
        path_by_ino: PathByInode<'conn>,
        count_ino: CountInodes<'conn>,
        increment_rc: IncrementRc<'conn>,
        insert_into_opendir: InsertIntoOpendir<'conn>,
        select_readdir: SelectReaddir<'conn>,
        forget_ino: ForgetInode<'conn>,
        ty_by_ino: TypeByInode<'conn>,
        is_empty: IsEmpty<'conn>,
        entry_exists: EntryExists<'conn>,
    }
}

/// Filesystem.
///
/// References with `'conn` lifetimes may only be
/// used in the main thread (by [::fuser::Filesystem] implementation)
/// whilst the `'scope` lifetime allows for sync resources to
/// be passed while using `spawn`.
#[derive(Debug)]
pub struct Fs<'conn, 'scope> {
    connection: &'conn Connection,
    stmt: DbStmts<'conn>,
    root_dir: BorrowedFd<'scope>,
    root_ino: i64,
    leak: bool,
    scope: &'conn ::rayon::Scope<'scope>,
    file_descriptors: &'scope DashMap<u64, OwnedFd, FxBuildHasher>,
    fh_counter: u64,
}

impl<'conn, 'scope> Fs<'conn, 'scope> {
    /// Create a new instance.
    pub fn new(
        root_dir: BorrowedFd<'scope>,
        connection: &'conn Connection,
        scope: &'conn ::rayon::Scope<'scope>,
        file_descriptors: &'scope DashMap<u64, OwnedFd, FxBuildHasher>,
    ) -> ::color_eyre::Result<Self> {
        connection.execute_batch(include_str!("./db_setup.sql"))?;

        Ok(Self {
            connection,
            root_ino: connection.last_insert_rowid(),
            stmt: DbStmts::new(connection)?,
            leak: false,
            fh_counter: 0,
            root_dir,
            file_descriptors,
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

    fn new_fh(&mut self) -> u64 {
        self.fh_counter += 1;
        self.fh_counter
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

        let mut insert = Insert::new_or_errno(transaction)?;

        insert.perform_or_errno(
            parent,
            crate::FileType::character_device(),
            Path::new(""),
            b".",
        )?;

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

            let path = crate::path_from_bytes(&path);
            let folded = case_fold(name);
            let ty = entry.file_type().into();

            if let Err(err) = insert.perform(parent, ty, path, &folded) {
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

        let (dir, ty) = action::PathTypeByInode::new(self.connection)
            .and_then(|mut action| action.perform(parent))
            .map_err(|err| {
                ::log::error!("could not get parent directory {parent}\n{err}");
                ::libc::EIO
            })?;

        if !ty.is_dir() {
            return Err(::libc::ENOTDIR);
        }

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
        self.stmt.lookup.perform_or_errno(parent, folded)
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

    fn increment_rc(&mut self, ino: i64) -> Result<(), i32> {
        self.stmt.increment_rc.perform(ino).map_err(|err| {
            ::log::error!("could not increase rc for {ino}\n{err}");
            ::libc::EIO
        })?;
        Ok(())
    }

    fn path_by_ino(&mut self, ino: i64) -> Result<crate::Buf, i32> {
        self.stmt.path_by_ino.perform(ino).map_err(|err| {
            ::log::error!("could not get path by ino {ino}\n{err}");
            ::libc::EIO
        })
    }

    fn lookup(&mut self, parent: i64, name: &[u8]) -> Result<FileAttr, i32> {
        let LookupResult { ino, path, ty: _ } = self.lookup_path(name, parent)?;
        let attr = get_attr(self.root_dir, path_from_bytes(&path), ino)?;
        self.increment_rc(ino)?;
        Ok(attr)
    }

    fn unlink(&mut self, parent: i64, name: &[u8]) -> Result<(), i32> {
        let LookupResult { ino, path, ty } = self.lookup_path(name, parent)?;

        if ty.is_dir() {
            return Err(::libc::EISDIR);
        }

        let random = ::rand::random::<u64>();
        let new_path = {
            use ::std::io::Write;
            let mut path = crate::Buf::new();
            write!(path, ".rm_{ino:X}_{random:X}").map_err(|err| {
                ::log::error!("write to smallvec failed\n{err}");
                ::libc::EIO
            })?;
            path
        };

        self.with_transaction(|transaction| {
            let path = path_from_bytes(&path);

            transaction
                .execute(
                    r#"
                    UPDATE files
                    SET 
                        name = ?2,
                        parent = 0
                    WHERE ino = ?1
                    "#,
                    (&ino, new_path.as_slice()),
                )
                .map_err(|err| {
                    ::log::error!("could not update row for unlink of {path:?}\n{err}");
                    ::libc::EIO
                })?;

            let new_path = path_from_bytes(&new_path);

            ::rustix::fs::renameat_with(
                self.root_dir,
                path,
                self.root_dir,
                new_path,
                RenameFlags::NOREPLACE,
            )
            .map_err(|err| {
                ::log::error!("could not rename {path:?} to {new_path:?}\n{err}");
                err.raw_os_error()
            })
        })
    }

    fn rmdir(&mut self, parent: i64, name: &[u8]) -> Result<(), i32> {
        let LookupResult { ino, path, ty } = self.lookup_path(name, parent)?;

        if !ty.is_dir() {
            return Err(::libc::ENOTDIR);
        }

        self.ensure_populated(ino)?;

        let is_empty = self.stmt.is_empty.perform(ino).map_err(|err| {
            ::log::error!("could not check if a directory was empty\n{err}");
            ::libc::EIO
        })?;

        if !is_empty {
            return Err(::libc::ENOTEMPTY);
        }

        let random = ::rand::random::<u64>();
        let new_path = {
            use ::std::io::Write;
            let mut path = crate::Buf::new();
            write!(path, ".rm_{ino:X}_{random:X}").map_err(|err| {
                ::log::error!("write to smallvec failed\n{err}");
                ::libc::EIO
            })?;
            path
        };

        self.with_transaction(|transaction| {
            let path = path_from_bytes(&path);

            transaction
                .execute(
                    r#"
                    UPDATE files
                    SET 
                        name = ?2,
                        parent = 0
                    WHERE ino = ?1
                    "#,
                    (&ino, new_path.as_slice()),
                )
                .map_err(|err| {
                    ::log::error!("could not update row for rmdir of {path:?}\n{err}");
                    ::libc::EIO
                })?;

            let new_path = path_from_bytes(&new_path);

            ::rustix::fs::renameat_with(
                self.root_dir,
                path,
                self.root_dir,
                new_path,
                RenameFlags::NOREPLACE,
            )
            .map_err(|err| {
                ::log::error!("could not rename {path:?} to {new_path:?}\n{err}");
                err.raw_os_error()
            })
        })
    }

    fn mknod(
        &mut self,
        parent: i64,
        name: &[u8],
        ty: crate::FileType,
        mode: Mode,
        rdev: u32,
    ) -> Result<FileAttr, i32> {
        self.ensure_populated(parent)?;

        let folded = case_fold(name);

        if self
            .stmt
            .entry_exists
            .perform(parent, &folded)
            .map_err(|err| {
                ::log::error!(
                    "could not check for existance of entry, parent {parent}, name {name}\n{err}",
                    name = OsStr::from_bytes(name).display()
                );
                ::libc::EIO
            })?
        {
            return Err(::libc::EEXIST);
        }

        let mut path = self.path_by_ino(parent)?;
        if !path.is_empty() {
            path.extend_from_slice(b"/");
        }
        path.extend_from_slice(name);

        self.with_transaction(|transaction| {
            let path = crate::path_from_bytes(&path);
            let ino =
                Insert::new_or_errno(transaction)?.perform_or_errno(parent, ty, &path, &folded)?;

            ::rustix::fs::mknodat(self.root_dir, path, *ty, mode, rdev.into()).map_err(|err| {
                ::log::error!("could not call mknod for {path:?}\n{err}");
                err.raw_os_error()
            })?;

            Ok(FileAttr {
                ino: ino.cast_unsigned(),
                size: 0,
                blocks: 0,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::UNIX_EPOCH,
                kind: ty
                    .to_fuser()
                    .unwrap_or_else(|| ::fuser::FileType::RegularFile),
                perm: mode.bits() as u16,
                nlink: 1,
                uid: getuid().as_raw(),
                gid: getgid().as_raw(),
                rdev,
                blksize: 0,
                flags: 0,
            })
        })
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
        match self.lookup(parent.cast_signed(), name.as_bytes()) {
            Ok(attr) => reply.entry(&Duration::MAX, &attr, 0),
            Err(err) => reply.error(err),
        }
    }

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        ::log::info!("forget - {ino}");
        if let Err(err) = self.stmt.forget_ino.perform(ino.cast_signed(), nlookup) {
            ::log::error!(
                "could not forget ino {ino} nlookup {nlookup}\n{err}\n{:#?}",
                DbgFn(|f| f
                    .debug_struct("parameters")
                    .field("ino", &ino)
                    .field("nlookup", &nlookup)
                    .finish())
            );
        }
    }

    fn mknod(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: fuser::ReplyEntry,
    ) {
        match self.mknod(
            parent.cast_signed(),
            name.as_bytes(),
            crate::FileType::from(::rustix::fs::FileType::from_raw_mode(mode)),
            Mode::from_raw_mode(mode) & Mode::not(Mode::from_raw_mode(umask)),
            rdev,
        ) {
            Ok(attr) => reply.entry(&Duration::MAX, &attr, 0),
            Err(err) => reply.error(err),
        }
    }

    fn unlink(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        match self.unlink(parent.cast_signed(), name.as_bytes()) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err),
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

    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        match self.rmdir(parent.cast_signed(), name.as_bytes()) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err),
        }
    }

    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        let ino = ino.cast_signed();
        let path = match self.path_by_ino(ino) {
            Ok(path) => path,
            Err(err) => return reply.error(err),
        };

        let fh = self.new_fh();
        let root_dir = self.root_dir;
        let file_descriptors = self.file_descriptors;
        self.spawn(move || {
            let path = path_from_bytes(&path);
            let flags = OFlags::from_bits_truncate(flags.cast_unsigned());
            let fd = match ::rustix::fs::openat(root_dir, path, flags, Mode::empty()) {
                Ok(fd) => fd,
                Err(err) => {
                    ::log::error!("could not open file, ino {ino}, path {path:?}\n{err}");
                    return reply.error(err.raw_os_error());
                }
            };
            let entry = file_descriptors.entry(fh);
            match entry {
                ::dashmap::Entry::Occupied(_) => {
                    ::log::error!("fh {fh} already in map");
                    reply.error(::libc::EIO);
                }
                ::dashmap::Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(fd);
                    reply.opened(fh, flags.bits());
                }
            };
        });
    }

    fn write(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        let data = Vec::from(data);
        let file_descriptors = self.file_descriptors;
        self.spawn(move || {
            let result = file_descriptors
                .get(&fh)
                .ok_or_else(|| {
                    ::log::error!("could not get fh {fh} from map");
                    ::libc::EIO
                })
                .and_then(|fd| {
                    let offset = log_conv::<_, usize>(offset)?;
                    let mut start_at = 0usize;

                    loop {
                        if start_at >= data.len() {
                            break;
                        }
                        match ::rustix::io::pwrite(
                            fd.as_fd(),
                            &data[start_at..],
                            log_conv(start_at + offset)?,
                        ) {
                            Ok(n) => {
                                start_at += n;
                            }
                            Err(err) => {
                                ::log::error!("while writing to {fd}\n{err}", fd = fd.as_raw_fd());
                                return Err(err.raw_os_error());
                            }
                        }
                    }

                    log_conv(start_at)
                });

            match result {
                Ok(value) => reply.written(value),
                Err(err) => reply.error(err),
            }
        });
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        let file_descriptors = self.file_descriptors;
        self.spawn(move || {
            let result = file_descriptors
                .get(&fh)
                .ok_or_else(|| {
                    ::log::error!("could not get fh {fh} from map");
                    ::libc::EIO
                })
                .and_then(|fd| {
                    let offset = log_conv::<_, usize>(offset)?;
                    let size = log_conv(size)?;
                    let mut buf = vec![0u8; size];
                    let mut start_at = 0usize;

                    loop {
                        match ::rustix::io::pread(
                            fd.as_fd(),
                            &mut buf[start_at..],
                            log_conv(offset + start_at)?,
                        ) {
                            Ok(0) => break,
                            Ok(n) => {
                                start_at += n;
                            }
                            Err(err) => {
                                ::log::error!(
                                    "while readding {raw_fd}\n{err}",
                                    raw_fd = fd.as_raw_fd()
                                );
                                return Err(err.raw_os_error());
                            }
                        }
                    }

                    Ok((buf, start_at))
                });

            match result {
                Ok((value, end)) => reply.data(&value[..end]),
                Err(err) => reply.error(err),
            }
        });
    }

    fn release(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        let file_descriptors = self.file_descriptors;
        self.spawn(move || match file_descriptors.remove(&fh) {
            Some(_) => reply.ok(),
            None => {
                ::log::error!("could not get fh {fh} from map");
                reply.error(::libc::EIO);
            }
        });
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
            DeleteFromOpendir::new(transaction)?.perform(fh.cast_signed())?;
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

                InsertIntoReaddir::new(transaction)?
                    .perform(fh.cast_signed(), ino.cast_signed())?;

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

    fn destroy(&mut self) {
        if self.leak {
            return;
        }
        ::log::info!("lowering rc");
        if let Err(err) = self.connection.execute(
            r#"
            UPDATE files
            SET rc = 0
            WHERE ino > 1
            "#,
            [],
        ) {
            ::log::error!("could not reset rc columns in database\n{err}");
        };
        let mut stmt = self
            .connection
            .prepare(r#"SELECT name, type FROM paths_to_delete"#)
            .map_err(|err| ::log::error!("could not prepare cleanup statement\n{err}"))
            .ok();
        if let Some(mut query) = stmt.as_mut().and_then(|stmt| {
            stmt.query([])
                .map_err(|err| ::log::error!("could not query paths to be deleted\n{err}"))
                .ok()
        }) {
            while let Some(row) = query
                .next()
                .map_err(|err| ::log::error!("failed to query row for path deletion\n{err}"))
                .ok()
                .flatten()
            {
                let Ok(path) = row
                    .get_ref(0)
                    .and_then(|r| Ok(r.as_bytes()?))
                    .map_err(|err| ::log::error!("could not get path for row\n{err}"))
                    .map(crate::path_from_bytes)
                else {
                    continue;
                };
                let Ok(ty) = row.get::<_, crate::FileType>(1).map_err(|err| {
                    ::log::error!("could not get file type for row {path:?}\n{err}")
                }) else {
                    continue;
                };

                match ::rustix::fs::unlinkat(
                    self.root_dir,
                    path,
                    if ty.is_dir() {
                        AtFlags::REMOVEDIR
                    } else {
                        AtFlags::empty()
                    },
                ) {
                    Ok(_) => ::log::info!("unlinked {path:?}"),
                    Err(err) => ::log::error!("could not unlink {path:?}\n{err}"),
                }
            }
        };
    }
}
