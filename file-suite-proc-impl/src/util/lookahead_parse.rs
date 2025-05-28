//! Utilities for parsing using a [Lookahead1].

use ::syn::{
    parse::{End, Lookahead1, ParseStream},
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
    T::optional_parse(input)
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
    Self: Sized,
{
    /// Parse an instance if lookahead peek matches.
    ///
    /// # Errors
    /// If a valid value peeked by lookahead cannot be parsed.
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> ::syn::Result<Option<Self>>;

    /// Parse an instance using [LookaheadParse::lookahead_parse] implementation.
    ///
    /// # Errors
    /// If an expected value cannot be parsed.
    /// Or if no expected value was encountered.
    #[inline]
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        if let Some(value) = Self::lookahead_parse(input, &lookahead)? {
            Ok(value)
        } else {
            Err(lookahead.error())
        }
    }

    /// Parse an instance if lookahead peek matches, else parse nothing.
    ///
    /// # Errors
    /// If a valid value peeked by lookahead cannot be parsed.
    #[inline]
    fn optional_parse(input: ParseStream) -> ::syn::Result<Option<Self>> {
        Self::lookahead_parse(input, &input.lookahead1())
    }

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
        #[doc(hidden)]
        mod kw {$(
            ::syn::custom_keyword!($kw);

            impl $crate::util::lookahead_parse::LookaheadParse for $kw {
                fn lookahead_parse(
                    input: ::syn::parse::ParseStream,
                    lookahead: &::syn::parse::Lookahead1
                ) -> ::syn::Result<Option<Self>> {
                    if lookahead.peek($kw) {
                        Ok(Some(::syn::parse::ParseBuffer::parse::<$kw>(input)?))
                    } else {
                        Ok(None)
                    }
                }
            }
        )*}
    };
}

/// Implement [LookaheadParse] for a struct with a leading value implementing [LookaheadParse].
#[macro_export]
macro_rules! lookahead_parse_struct {
    ($name:ident {
        $lnm:ident: $lty:ty
    $(
        , $([$attr:ident])? $fnm:ident: $fty:ty
    )* $(,)?
    }) => {
        impl $crate::util::lookahead_parse::LookaheadParse for $name {
            fn lookahead_parse(
                input: ::syn::parse::ParseStream,
                lookahead: &::syn::parse::Lookahead1,
            ) -> ::syn::Result<Option<Self>> {
                if let Some($lnm) = <$lty>::lookahead_parse(input, lookahead)? {
                    $(
                    let $fnm = $crate::lookahead_parse_struct!(@arm input, $fty $(, $attr)*);
                    )*
                    Ok(Some(Self { $lnm $(, $fnm)* }))
                } else {
                    Ok(None)
                }
            }
        }
    };
    (@arm $input:expr, $ty:ty) => {{ <$ty>::parse($input)? }};
    (@arm $input:expr, $ty:ty, optional) => {{ $crate::util::lookahead_parse::LookaheadParse::optional_parse($input)? }};
}

/// Create an enum of [LookaheadParse] types, that itself implements [LookaheadParse].
#[macro_export]
macro_rules! lookahead_parse_enum {
    (
        $name:ident { $(
            $fnm:ident($fty:ty)
        ),+ $(,)?}) => {

        impl $crate::util::lookahead_parse::LookaheadParse for $name {
            fn lookahead_parse(
                input: ::syn::parse::ParseStream,
                lookahead: &::syn::parse::Lookahead1
            ) -> ::syn::Result<Option<Self>> {
                $( if let Some(value) = <$fty>::lookahead_parse(input, lookahead)? {
                    Ok(Some(Self::$fnm(value)))
                } else )* {
                    Ok(None)
                }
            }
        }
    };
}
mod peek_impl {
    //! Implementation for types implementing peek.
    use ::syn::{
        Ident, LitBool, LitChar, LitInt, LitStr,
        ext::IdentExt,
        token::{Colon, Comma, Dot, Eq},
    };

    peek_impl!(LitStr LitInt LitBool LitChar Comma Dot Eq Colon);

    impl LookaheadParse for Ident {
        fn lookahead_parse(
            input: syn::parse::ParseStream,
            lookahead: &syn::parse::Lookahead1,
        ) -> syn::Result<Option<Self>> {
            if lookahead.peek(Ident) {
                Ok(Some(input.call(Ident::parse_any)?))
            } else {
                Ok(None)
            }
        }
    }

    /// Implement for peek implementor.
    macro_rules! peek_impl {
        ($($ident:ident)*) => {$(
            impl $crate::util::lookahead_parse::LookaheadParse for $ident {
                fn lookahead_parse(
                    input: ::syn::parse::ParseStream,
                    lookahead: &::syn::parse::Lookahead1,
                ) -> ::syn::Result<Option<Self>> {
                    if lookahead.peek($ident) {
                        Ok(Some(input.parse()?))
                    } else {
                        Ok(None)
                    }
                }
            }
        )*};
    }
    use peek_impl;

    use crate::util::lookahead_parse::LookaheadParse;
}
