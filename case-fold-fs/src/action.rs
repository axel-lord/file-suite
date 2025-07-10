//! Database actions.

use ::std::marker::PhantomData;

use ::rusqlite::{Connection, Statement, named_params};
use ::smallvec::SmallVec;

use crate::action::{
    param::{InsertParams, LookupParams},
    result::{DirectoryResult, LookupResult},
};

/// Trait for types which may perform an action.
pub trait Perform {
    /// Error type returned on failure.
    type Err;

    /// Parameters which should be passed as input.
    type Param<'t>;

    /// Returned value on success.
    type Output;

    /// Sql expression of operation.
    const SQL: &str;

    /// Perform action.
    fn perform(
        stmt: &mut Statement<'_>,
        params: Self::Param<'_>,
    ) -> Result<Self::Output, Self::Err>;
}

/// A database action which may be performed.
#[derive(Debug)]
pub struct Action<'conn, T> {
    stmt: Statement<'conn>,
    _p: PhantomData<fn() -> T>,
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
            stmt: connection.prepare(T::SQL)?,
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

    /// A directory row.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DirectoryResult {
        /// If it is known if this parent has any children and if so
        /// what is known.
        ///
        /// # Returns
        /// `Some(true)` if the parent has children.
        ///
        /// `Some(false)` if the parent has no children.
        ///
        /// `None` if it is unknown.
        pub has_children: Option<bool>,

        /// Relative path from root to parent.
        pub path: SmallVec<[u8; 64]>,
    }

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
    /// Get directory info of inode.
    [r"SELECT name, has_children FROM files WHERE ino = :ino"]
    Directory(stmt, param: i64) -> Result<DirectoryResult, ::rusqlite::Error> {
        stmt.query_row(named_params! {":ino": param}, |row| {
            Ok(DirectoryResult {
                path: row.get_ref("name")?.as_bytes().map(SmallVec::from_slice)?,
                has_children: row
                    .get_ref("has_children")?
                    .as_i64_or_null()?
                    .map(|val| val != 0),
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
    [r"INSERT INTO files (parent, name, folded) VALUES (:parent, :name, :folded)"]
    for<'b> Insert(stmt, params: InsertParams<'b>) -> Result<usize, ::rusqlite::Error> {
        let InsertParams {
            parent,
            path,
            folded,
        } = params;
        stmt.execute(named_params! {":parent": parent, ":name": path, ":folded": folded})
    }
}

action! {
    /// Set the value of has_children for a row.
    [r#"UPDATE files SET has_children = :has_children WHERE ino = :ino"#]
    SetHasChildren(stmt, params: (i64, bool)) -> Result<(), ::rusqlite::Error> {
        stmt.execute(named_params! {":has_children": i64::from(params.1), ":ino": params.0}).map(|_| ())
    }
}

macro_rules! action {
    (
        #[doc = $doc:expr]
        [$sql:expr]
        $nm:ident(
            $stmt_ident:ident,
            $params_ident:ident: $param:ty
        ) -> Result<$output:ty, $err:ty> $stmt:stmt
    ) => {
        $crate::action::action! {
            #[doc = $doc]
            [$sql]
            for<'_t> $nm($stmt_ident, $params_ident: $param) -> Result<$output, $err> $stmt
        }
    };
    (
        #[doc = $doc:expr]
        [$sql:expr]
        for<$lt:lifetime>
        $nm:ident(
            $stmt_ident:ident,
            $params_ident:ident: $param:ty
        ) -> Result<$output:ty, $err:ty> $stmt:stmt
    ) => {
        #[doc = $doc]
        #[derive(Debug)]
        pub enum $nm {}

        impl $crate::action::Perform for $nm {
            type Err = $err;
            type Param<$lt> = $param;
            type Output = $output;

            const SQL: &'static str = $sql;

            fn perform(
                $stmt_ident: &mut Statement<'_>,
                $params_ident: Self::Param<'_>,
            ) -> Result<Self::Output, Self::Err> {
                $stmt
            }
        }
    };
}
pub(crate) use action;
