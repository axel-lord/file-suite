//! Utilities for parsing using a [Lookahead1].

use ::file_suite_proc_lib::{Lookahead, lookahead::ParseBufferExt};
use ::syn::{
    parse::{End, Lookahead1, Parse, ParseStream},
    punctuated::Punctuated,
};

/// Use a [LookaheadParse] impl to parse T if possible.
///
/// # Errors
/// If a valid value peeked by lookahead cannot be parsed.
#[inline]
pub fn lookahead_parse<T>(input: ParseStream, lookahead: &Lookahead1) -> ::syn::Result<Option<T>>
where
    T: LookaheadParse,
{
    T::lookahead_parse(input, lookahead)
}

/// Use a [LookaheadParse] impl to parse an optional T if match, else parse nothing.
///
/// # Errors
/// If a valid value peeked by lookahead cannot be parsed.
#[inline]
pub fn optional_parse<T>(input: ParseStream) -> ::syn::Result<Option<T>>
where
    T: LookaheadParse,
{
    input.optional_parse()
}

/// Use a [LookaheadParse] impl to parse an optional T if match, else parse nothing.
/// On parse the lookahead is replaced.
///
/// # Errors
/// If a valid value peeked by lookahead cannot be parsed.
#[inline]
pub fn optional_lookahead_parse<'a, T>(
    input: ParseStream<'a>,
    lookahead: &mut Lookahead1<'a>,
) -> ::syn::Result<Option<T>>
where
    T: LookaheadParse,
{
    T::optional_lookahead_parse(input, lookahead)
}

/// Use a [LookaheadParse] impl to parse a list of T punctuated by P if possible.
/// With optional trailing punctuation.
///
/// # Errors
/// If a valid value peeked by lookahead cannot be parsed.
pub fn lookahead_parse_terminated<T: LookaheadParse, P: LookaheadParse>(
    input: ParseStream,
    lookahead: &Lookahead1,
) -> ::syn::Result<Option<Punctuated<T, P>>> {
    let Some(first) = lookahead_parse(input, lookahead)? else {
        return Ok(None);
    };

    let mut punctuated = Punctuated::new();
    punctuated.push_value(first);

    let lookahead = input.lookahead1();
    if lookahead.peek(End) {
        return Ok(Some(punctuated));
    }

    let punct = lookahead_parse(input, &lookahead)?.ok_or_else(|| lookahead.error())?;
    punctuated.push_punct(punct);

    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(End) {
            break;
        }

        let value = lookahead_parse(input, &lookahead)?.ok_or_else(|| lookahead.error())?;
        punctuated.push_value(value);

        let lookahead = input.lookahead1();
        if lookahead.peek(End) {
            break;
        }

        let punct = lookahead_parse(input, &lookahead)?.ok_or_else(|| lookahead.error())?;
        punctuated.push_punct(punct);
    }

    Ok(Some(punctuated))
}

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
