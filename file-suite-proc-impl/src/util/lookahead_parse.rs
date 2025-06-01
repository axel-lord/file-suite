#![allow(deprecated)]

//! Utilities for parsing using a [Lookahead1].

use ::file_suite_proc_lib::Lookahead;
use ::syn::parse::{Lookahead1, Parse, ParseStream};

#[deprecated]
/// Trait for conditional parsing useing a [Lookahead1].
pub trait LookaheadParse
where
    Self: Sized + Parse + Lookahead,
{
    /// Parse an instance if lookahead peek matches and replace lookahead, else leave lookahead as-is.
    ///
    /// # Errors
    /// If a valid value peeked by lookahead cannot be parsed.
    fn optional_lookahead_parse<'a>(
        input: ParseStream<'a>,
        lookahead: &mut Lookahead1<'a>,
    ) -> ::syn::Result<Option<Self>> {
        if let Some(value) = Self::lookahead_parse(input, lookahead)? {
            *lookahead = input.lookahead1();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

/// Create keywords implementing [LookaheadParse].
#[macro_export]
macro_rules! lookahead_parse_keywords {
    ($($kw:ident),* $(,)?) => {
        ::file_suite_proc_lib::lookahead_keywords!(#[doc(hidden)] kw {$($kw),*});
        $(
        impl $crate::util::lookahead_parse::LookaheadParse for kw::$kw {}
        )*
    };
}

/// Create an enum of [LookaheadParse] types, that itself implements [LookaheadParse].
#[macro_export]
macro_rules! lookahead_parse_enum {
    (
        $name:ident { $(
            $fnm:ident($fty:ty)
        ),+ $(,)?}) => {

        ::file_suite_proc_lib::lookahead_parse_enum!($name { $($fnm: $fty),* });

        impl $crate::util::lookahead_parse::LookaheadParse for $name {}
    };
}
mod peek_impl {
    //! Implementation for types implementing peek.
    use ::syn::{
        Ident, LitBool, LitChar, LitInt, LitStr,
        token::{Colon, Comma, Dot, Eq},
    };

    peek_impl!(LitStr LitInt LitBool LitChar Comma Dot Eq Colon);

    impl LookaheadParse for Ident {}

    /// Implement for peek implementor.
    macro_rules! peek_impl {
        ($($ident:ident)*) => {$(
            impl $crate::util::lookahead_parse::LookaheadParse for $ident {
            }
        )*};
    }
    use peek_impl;

    use crate::util::lookahead_parse::LookaheadParse;
}
