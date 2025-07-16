//! Database actions.

use ::std::{ffi::OsStr, fmt::Debug, os::unix::ffi::OsStrExt, path::Path};

use ::rusqlite::named_params;
use ::smallvec::SmallVec;
use ::tap::Pipe;

use crate::{action::result::LookupResult, macros::action};

/// Trait to provide alternative debug implementations.
pub trait DbgProxy
where
    Self: Debug + Sized + Copy,
{
    /// For macro generated debugging use provided debug implementation instead.
    fn dbg_proxy(self) -> impl Debug {
        self
    }
}

impl DbgProxy for u64 {}
impl DbgProxy for i64 {}
impl DbgProxy for &Path {}
impl DbgProxy for crate::FileType {}
impl DbgProxy for &[u8] {
    fn dbg_proxy(self) -> impl Debug {
        OsStr::from_bytes(self)
    }
}

pub mod result {
    //! Types of values returned by actions.

    /// Result of a lookup.
    #[derive(Debug)]
    pub struct LookupResult {
        /// Inode of child.
        pub ino: i64,
        /// Relative path to child.
        pub path: crate::Buf,
        /// Type of child.
        pub ty: crate::FileType,
    }
}

action! {
    /// Count amount of inodes in database.
    [r"SELECT COUNT(ino) FROM files"]
    CountInodes(stmt) -> Result<u64, i32> {
        stmt.query_row([], |row| Ok(row.get_ref(0)?.as_i64()?))
            .map_err(|err| {
                ::log::error!("could not cound inodes\n{err}");
                ::libc::EIO
            })
            .and_then(|ino| {
                u64::try_from(ino).map_err(|err| {
                    ::log::error!("inode count returned a negative number {ino}\n{err}");
                    ::libc::EIO
                })
            })
    }
}

action! {
    /// Get relative path of inode.
    [r"SELECT name FROM files WHERE ino = ?1"]
    PathByInode(stmt, ino: i64) -> Result<crate::Buf, ::rusqlite::Error> {
        stmt.query_row((&ino,), |row| {
            Ok(crate::Buf::from_slice(row.get_ref("name")?.as_bytes()?))
        })
    }
}

action! {
    /// Get inode type.
    [r"SELECT type FROM files WHERE ino = ?1"]
    TypeByInode(stmt, ino: i64) -> Result<crate::FileType, ::rusqlite::Error> {
        stmt.query_row((&ino,), |row| row.get(0))
    }
}

action! {
    /// Get path and type by inode.
    [r"SELECT name, type FROM files WHERE ino = ?1"]
    PathTypeByInode(stmt, ino: i64) -> Result<(crate::Buf, crate::FileType), ::rusqlite::Error> {
        stmt.query_row((&ino,), |row| Ok((row.get_ref(0)?.as_bytes()?.pipe(SmallVec::from_slice), row.get(1)?)))
    }
}

action! {
    /// Lookup an entry by parent and name.
    [r"SELECT ino, name, type FROM files WHERE parent = ?1 AND folded = ?2 LIMIT 1"]
    Lookup(stmt, parent: i64, folded: &[u8]) -> Result<Option<LookupResult>, ::rusqlite::Error> {
        stmt.query_map((&parent, folded), |row| {
            Ok(LookupResult {
                ino: row.get_ref(0)?.as_i64()?,
                path: row.get_ref(1)?.as_bytes().map(SmallVec::from_slice)?,
                ty: row.get(2)?,
            })
        })?.next().transpose()
    }
}

action! {
    /// Insert a row into the database.
    [r"INSERT INTO files (parent, name, folded, type) VALUES (:parent, :name, :folded, :type) RETURNING ino"]
    Insert(stmt, parent: i64, ty: crate::FileType, path: &Path, folded: &[u8]) -> Result<i64, ::rusqlite::Error> {
        let path = path.as_os_str().as_bytes();
        stmt.query_row(named_params! {":parent": parent, ":name": path, ":folded": folded, ":type": ty}, |row| Ok(row.get(0)?))
    }
}

action! {
    /// Increase rc of an inode
    [r"UPDATE files SET rc = rc + 1 WHERE ino = ?1"]
    IncrementRc(stmt, ino: i64) -> Result<(), ::rusqlite::Error> {
        stmt.execute ((&ino,))?;
        Ok(())
    }
}

action! {
    /// Insert a directory into opendir table.
    [r"INSERT INTO opendir (ino) VALUES (?1) RETURNING fh"]
    InsertIntoOpendir(stmt, ino: i64) -> Result<i64, ::rusqlite::Error> {
        stmt.query_row((&ino,), |row| Ok(row.get_ref(0)?.as_i64()?))
    }
}

action! {
    /// Delete rows from opendir and readdir
    [r"DELETE FROM opendir WHERE fh = ?1"]
    DeleteFromOpendir(stmt, fh: i64) -> Result<(), ::rusqlite::Error> {
        stmt.execute((&fh,))?;
        Ok(())
    }
}

action! {
    /// Move rows of an open directory to readdir table.
    [
        r#"
            INSERT INTO readdir (ino, fh, name, type)
            SELECT ino, ?1, name, type
                FROM files
                WHERE parent = ?2 AND folded != ?3;
        "#
    ]
    InsertIntoReaddir(stmt, fh: i64, parent: i64) -> Result<usize, ::rusqlite::Error> {
        stmt.execute((&fh, &parent, b"."))
    }
}

action! {
    /// Select readdir rows by fh
    [
        r#"
            SELECT ino, name, type
                FROM readdir
                WHERE fh = ?1 AND ino > ?2
                ORDER BY ino
        "#
    ]
    SelectReaddir<'stmt>(stmt, fh: i64, offset: i64) -> Result<::rusqlite::Rows<'stmt>, ::rusqlite::Error> {
        stmt.query((&fh, &offset))
    }
}

action! {
    /// Forget an inode
    [r"UPDATE files SET rc = rc - ?2 WHERE ino = ?1"]
    ForgetInode(stmt, ino: i64, nlookup: u64) -> Result<(), ::rusqlite::Error> {
        stmt.execute((&ino, &nlookup))?;
        Ok(())
    }
}

action! {
    /// Check if a directory is empty.
    [r#"SELECT 1 FROM files WHERE parent = ?1 AND folded != ?2 LIMIT 1"#]
    IsEmpty(stmt, ino: i64) -> Result<bool, ::rusqlite::Error> {
        Ok(!stmt.exists((&ino, b"." ))?)
    }
}

action! {
    /// Check if an entry exists for parent, folded.
    [r"SELECT 1 FROM files WHERE parent = ?1 AND folded = ?2 LIMIT 1"]
    EntryExists(stmt, parent: i64, folded: &[u8]) -> Result<bool, ::rusqlite::Error> {
        stmt.exists((&parent, folded))
    }
}

action! {
    /// Select paths that are to be deleted.
    [r#"SELECT name, type FROM paths_to_delete"#]
    SelectToBeDeleted(stmt) -> Result<impl Iterator<Item = Result<(SmallVec<[u8; 64]>, crate::FileType), i32>>, ::rusqlite::Error> {
        Ok(stmt.query_map([], |row| Ok((
            row.get_ref(0)?
                .as_bytes()?
                .pipe(SmallVec::from_slice),
            row.get::<_, crate::FileType>(1)?
        )))?.map(|result| result.map_err(|err| {
            ::log::error!("could not get row from paths_to_delete\n{err}");
            ::libc::EIO
        })))
    }
}

action! {
    /// Lower reference counts of files to 0.
    [
        r#"
        UPDATE files
        SET rc = 0
        WHERE ino > 1
        "#
    ]
    ResetRc(stmt) -> Result<usize, ::rusqlite::Error> {
        stmt.execute([])
    }
}

action! {
    /// Unlink a file or directory
    [
        r#"
        UPDATE files
        SET 
            name = ?2,
            parent = 0
        WHERE ino = ?1
        "#
    ]
    UnlinkFile(stmt, ino: i64, temp_path: &[u8]) -> Result<(), ::rusqlite::Error> {
        stmt.execute((&ino, temp_path))?;
        Ok(())
    }
}

action! {
    /// Check if a directory has a child marker entry.
    [r#"SELECT 1 FROM files WHERE parent = ?1 AND folded = ?2 LIMIT 1"#]
    HasChildMarker(stmt, parent: i64) -> Result<bool, ::rusqlite::Error> {
        stmt.exists((&parent, b"."))
    }
}
