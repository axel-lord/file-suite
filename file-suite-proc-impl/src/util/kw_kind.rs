//! Macro to create an enum and a wrapper for parsing keywords.

/// Create an enum and a wrapper for parsing a set of keywords.
macro_rules! kw_kind {
    (
        $(#[ $($wrap_attr:tt)* ])*
        $wrap_ident:ident
        $(: $($wrap_derive:path),+ )?
        ;

        $(#[ $($kind_attr:tt)* ])*
        $kind_ident:ident
        $(: $($kind_derive:path),+ )?
        {$(
            $(#[$($keyword_attr:tt)*])*
            $keyword_ident:ident
        ),+ $(,)?}) => {

        ::file_suite_proc_lib::kw_kind!{
            $(#[ $($wrap_attr)* ])*
            $wrap_ident
            $(: $($wrap_derive),* )*
            ;

            $(#[ $($kind_attr)* ])*
            $kind_ident
            $(: $($kind_derive),* )*
            {$(
                $(#[$($keyword_attr)*])*
                $keyword_ident
            ),*}
        }

        const _: () = {
        impl $crate::util::lookahead_parse::LookaheadParse for $wrap_ident {}

        impl $crate::array_expr::from_values::FromValues for $kind_ident {
            fn from_values(values: &[$crate::array_expr::value::Value]) -> $crate::Result<Self> {
                match $crate::array_expr::from_values::str_from_values(values)? {
                    $(
                    stringify!($keyword_ident) => Ok(Self::$keyword_ident),
                    )*
                    value => Err(format!("{value} is not a variant of {}", stringify!($kind_ident)).into()),
                }
            }
        }

        };
    };
}

pub(crate) use kw_kind;
