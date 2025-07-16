#![allow(unused)]

macro_rules! replace_expr {
    ($_t:expr, $sub:expr) => {
        $sub
    };
}
pub(crate) use replace_expr;

macro_rules! conv_or_reply {
    ($reply:ident, $expr:expr, $ty:ty) => {
        match <$ty>::try_from($expr) {
            Ok(val) => val,
            Err(err) => {
                ::log::error!(
                    "could not convert {} {} to type {}\n{err}",
                    stringify!($expr),
                    $expr,
                    stringify!($ty)
                );
                return $reply.error(::libc::EIO);
            }
        }
    };
}
pub(crate) use conv_or_reply;

/// Create a [Perform][crate::action::Perform] implementor.
///
/// # Usage
/// ```
/// action! {
///     /// Get relative path of inode.
///     [r"SELECT name FROM files WHERE ino = ?1"]
///     PathByInode(stmt, param: i64) -> Result<SmallVec<[u8; 64]>, ::rusqlite::Error> {
///         stmt.query_row((&param,), |row| {
///             Ok(SmallVec::from_slice(row.get_ref("name")?.as_bytes()?))
///         })
///     }
/// }
/// ```
macro_rules! action {
    {
        #[doc = $doc:expr]
        [$sql:expr]
         $nm:ident $(<$lt:lifetime>)? (
            $stmt_ident:ident
            $(, $params_ident:ident: $param:ty )* $(,)?
        ) -> Result<$output:ty, $err:ty> $stmt:stmt
    } => {
        #[doc = $doc]
        pub struct $nm<'conn> {
            $stmt_ident: ::rusqlite::CachedStatement<'conn>,
        }

        impl<'conn> ::core::fmt::Debug for $nm<'conn> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!($ident))
                    .field(stringify!($stmt_ident), &*self.stmt)
                    .finish()
            }
        }

        impl<'conn> $nm<'conn> {
            /// Create a new instance bound to passed connection.
            pub fn new(connection: &'conn ::rusqlite::Connection) -> Result<Self, ::rusqlite::Error> {
                Ok(Self {
                    $stmt_ident: connection.prepare_cached($sql)?,
                })
            }

            /// Create a new instance bound to passed connection.
            pub fn new_or_errno(connection: &'conn ::rusqlite::Connection) -> Result<Self, i32> {
                Self::new(connection).map_err(|err| {
                    ::log::error!("could not crate {} action\n{err}", stringify!($nm));
                    ::libc::EIO
                })
            }

            /// Perform sql statement using given parameters.
            pub fn perform $( < $lt > )*(& $($lt)* mut self, $( $params_ident: $param ),*) -> Result<$output, $err>
            $( where 'conn: $lt )*
            {
                let Self {$stmt_ident} = self;
                $stmt
            }

            /// Perform sql statement using given parameters.
            pub fn perform_or_errno $( < $lt > )*(& $($lt)* mut self, $( $params_ident: $param ),*) -> Result<$output, i32>
            $( where 'conn: $lt )*
            {
                self.perform($($params_ident),*).map_err(|err| {
                    ::log::error!(
                        "could not perform {} action\n{:#?}\n{err}",
                        stringify!($nm),
                        $crate::DbgFn(|f| f
                            .debug_struct("parameters")
                            $( .field(stringify!($params_ident), &$params_ident) )*
                            .finish()),
                    );
                    ::libc::EIO
                })
            }
        }
    };
}
pub(crate) use action;

/// Create a collection of statements bound to a connection.
///
/// # Usage
/// ```
/// db_stmts! {
///     pub DbStmts {
///         lookup: action::Lookup,
///         count_ino: action::CountInodes,
///     }
/// }
/// ```
macro_rules! db_stmts {
    {$vis:vis $ident:ident<$lt:lifetime> { $( $field:ident: $perform:ty ),+ $(,)? }} => {
        #[derive(Debug)]
        $vis struct $ident<$lt>{
            $( pub $field: $perform,)*
        }

        impl<$lt> $ident<$lt> {
            #[doc = "Create a new instance from a connection."]
            pub fn new(connection: &$lt ::rusqlite::Connection) -> ::color_eyre::Result<Self> {
                Ok(Self {
                    $( $field: <$perform>::new(connection)? ,)*
                })
            }
        }
    };
}
pub(crate) use db_stmts;
