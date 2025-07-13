//! Database actions.

use ::std::{fmt::Debug, marker::PhantomData};

use ::rusqlite::{CachedStatement, Connection, Statement, named_params};
use ::smallvec::SmallVec;

use crate::{
    action::{
        param::{InsertParams, LookupParams},
        result::LookupResult,
    },
    macros::action,
};

/// Trait for types which may perform an action.
pub trait Perform {
    /// Error type returned on failure.
    type Err;

    /// Parameters which should be passed as input.
    type Param<'t>;

    /// Returned value on success.
    type Output;

    /// Get sql expression to create statement using.
    fn sql() -> &'static str;

    /// Perform action.
    fn perform(
        stmt: &mut Statement<'_>,
        params: Self::Param<'_>,
    ) -> Result<Self::Output, Self::Err>;

    /// Perform an action once, creating the statement on every call.
    fn perform_once<E>(connection: &Connection, params: Self::Param<'_>) -> Result<Self::Output, E>
    where
        E: From<Self::Err> + From<::rusqlite::Error>,
    {
        let mut stmt = connection.prepare(Self::sql())?;
        Ok(Self::perform(&mut stmt, params)?)
    }
}

/// A database action which may be performed.
pub struct Action<'conn, T> {
    stmt: CachedStatement<'conn>,
    _p: PhantomData<fn() -> T>,
}

impl<'conn, T> Debug for Action<'conn, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("stmt", &*self.stmt)
            .field("T", &format_args!("{}", ::std::any::type_name::<T>()))
            .finish()
    }
}

impl<'conn, T> Action<'conn, T>
where
    T: Perform,
{
    /// Construct a new action bound to a database connection.
    ///
    /// # Errors
    /// If the sql cannot be prepared.
    pub fn new(connection: &'conn Connection) -> Result<Self, ::rusqlite::Error> {
        Ok(Self {
            stmt: connection.prepare_cached(T::sql())?,
            _p: PhantomData,
        })
    }

    /// Perform the action using specified parameters.
    #[inline]
    pub fn perform(&mut self, params: T::Param<'_>) -> Result<T::Output, T::Err> {
        T::perform(&mut self.stmt, params)
    }
}

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
    CountInodes(stmt, _param: ()) -> Result<u64, i32> {
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
    PathByInode(stmt, param: i64) -> Result<SmallVec<[u8; 64]>, ::rusqlite::Error> {
        stmt.query_row((&param,), |row| {
            Ok(SmallVec::from_slice(row.get_ref("name")?.as_bytes()?))
        })
    }
}

action! {
    /// Lookup an entry by parent and name.
    [r"SELECT ino, name FROM files WHERE parent = ?1 AND folded = ?2"]
    for<'p> Lookup(stmt, param: LookupParams<'p>) -> Result<LookupResult, ::rusqlite::Error> {
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
    for<'b> Insert(stmt, params: InsertParams<'b>) -> Result<usize, ::rusqlite::Error> {
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
    IncrementRc(stmt, param: i64) -> Result<i64, ::rusqlite::Error> {
        stmt.query_row((&param,), |row| Ok(row.get_ref(0)?.as_i64()?))
    }
}

action! {
    /// Insert a directory into opendir table.
    [r"INSERT INTO opendir (ino) VALUES (?1) RETURNING fh"]
    InsertIntoOpendir(stmt, param: i64) -> Result<i64, ::rusqlite::Error> {
        stmt.query_row((&param,), |row| Ok(row.get_ref(0)?.as_i64()?))
    }
}
