//! Utilities for parsing using a [Lookahead1].

use ::syn::parse::{Lookahead1, ParseStream};

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

    /// Parse an instance if using [LookaheadParse::lookahead_parse] implementation.
    ///
    /// # Errors
    /// If an expected value cannot be parsed.
    /// Or if no expected value was encountered.
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        if let Some(value) = Self::lookahead_parse(input, &lookahead)? {
            Ok(value)
        } else {
            Err(lookahead.error())
        }
    }
}

/// Create an enum of [LookaheadParse] types, that itsels implements [LookaheadParse].
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
    use ::syn::{Ident, LitBool, LitChar, LitInt, LitStr, ext::IdentExt};

    peek_impl!(LitStr LitInt LitBool LitChar);

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
