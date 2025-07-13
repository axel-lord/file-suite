//! Database actions.

use ::rusqlite::named_params;
use ::smallvec::SmallVec;

use crate::{
    action::{
        param::{InsertParams, LookupParams},
        result::LookupResult,
    },
    macros::action,
};

pub mod result {
    //! Types of values returned by actions.

    use ::smallvec::SmallVec;

    /// Result of a lookup.
    #[derive(Debug)]
    pub struct LookupResult {
        /// Inode of child.
        pub ino: i64,
        /// Relative path to child.
        pub path: SmallVec<[u8; 64]>,
    }
}

pub mod param {
    //! Types used as parameters by actions.

    /// Parameters for lookup.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LookupParams<'b> {
        /// Parent inode.
        pub parent: i64,
        /// Folded name of value to find
        pub folded: &'b [u8],
    }

    /// Parameters for insert.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InsertParams<'b> {
        /// Parent of row
        pub parent: i64,
        /// Type for row
        pub ty: crate::FileType,
        /// Relative path of row.
        pub path: &'b [u8],
        /// Case folded name of row.
        pub folded: &'b [u8],
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
    PathByInode(stmt, ino: i64) -> Result<SmallVec<[u8; 64]>, ::rusqlite::Error> {
        stmt.query_row((&ino,), |row| {
            Ok(SmallVec::from_slice(row.get_ref("name")?.as_bytes()?))
        })
    }
}

action! {
    /// Lookup an entry by parent and name.
    [r"SELECT ino, name FROM files WHERE parent = ?1 AND folded = ?2"]
    Lookup(stmt, param: LookupParams<'_>) -> Result<LookupResult, ::rusqlite::Error> {
        stmt.query_row((&param.parent, param.folded), |row| {
            Ok(LookupResult {
                ino: row.get_ref("ino")?.as_i64()?,
                path: row.get_ref("name")?.as_bytes().map(SmallVec::from_slice)?,
            })
        })
    }
}

action! {
    /// Insert a row into the database.
    [r"INSERT INTO files (parent, name, folded, type) VALUES (:parent, :name, :folded, :type)"]
    Insert(stmt, params: InsertParams<'_>) -> Result<usize, ::rusqlite::Error> {
        let InsertParams {
            parent,
            path,
            folded,
            ty,
        } = params;
        stmt.execute(named_params! {":parent": parent, ":name": path, ":folded": folded, ":type": ty})
    }
}

action! {
    /// Increase rc of an inode
    [r"UPDATE files SET rc = rc + 1 WHERE ino = ?1 RETURNING rc"]
    IncrementRc(stmt, ino: i64) -> Result<i64, ::rusqlite::Error> {
        stmt.query_row((&ino,), |row| Ok(row.get_ref(0)?.as_i64()?))
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
    [r"DELETE FROM opendir WHERE fh = ?1", r"DELETE FROM readdir WHERE fh = ?1"]
    DeleteFromOpendirReaddir(stmts, fh: i64) -> Result<(), ::rusqlite::Error> {
        for stmt in stmts {
            stmt.execute((&fh,))?;
        }
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
    InsertToReaddir(stmt, fh: i64, parent: i64) -> Result<usize, ::rusqlite::Error> {
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
