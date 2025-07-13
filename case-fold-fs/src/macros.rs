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
        $nm:ident(
            $stmt_ident:ident,
            $params_ident:ident: $param:ty
        ) -> Result<$output:ty, $err:ty> $stmt:stmt
    } => {
        $crate::action::action! {
            #[doc = $doc]
            [$sql]
            for<'_t> $nm($stmt_ident, $params_ident: $param) -> Result<$output, $err> $stmt
        }
    };
    {
        #[doc = $doc:expr]
        [$sql:expr]
        for<$lt:lifetime>
        $nm:ident(
            $stmt_ident:ident,
            $params_ident:ident: $param:ty
        ) -> Result<$output:ty, $err:ty> $stmt:stmt
    } => {
        #[doc = $doc]
        #[derive(Debug)]
        pub enum $nm {}

        impl $crate::action::Perform for $nm {
            type Err = $err;
            type Param<$lt> = $param;
            type Output = $output;

            fn sql() -> &'static str {
                $sql
            }

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
    {$vis:vis $ident:ident { $( $field:ident: $perform:ty ),+ $(,)? }} => {
        #[derive(Debug)]
        $vis struct $ident<'conn>{
            $( pub $field: $crate::action::Action<'conn, $perform>,)*
        }

        impl<'conn> $ident<'conn> {
            #[doc = "Create a new instance from a connection."]
            pub fn new(connection: &'conn ::rusqlite::Connection) -> ::color_eyre::Result<Self> {
                Ok(Self {
                    $( $field: $crate::action::Action::new(connection)? ,)*
                })
            }
        }
    };
}
pub(crate) use db_stmts;
