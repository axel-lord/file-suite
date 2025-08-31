//! Utilities for lookahead parsing.

use ::syn::{
    parse::{Lookahead1, Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
};

/// Trait for types which should be parsed based on the next token in a lookahead.
pub trait Lookahead {
    /// Check if the next token indicates trait implementor should be parsed.
    fn lookahead_peek(lookahead: &Lookahead1) -> bool;

    /// Check if the next token in input indicates trait implementor should be parsed.
    #[inline]
    fn input_peek(input: ParseStream) -> bool {
        Self::lookahead_peek(&input.lookahead1())
    }

    /// Parse the type T if [Lookahead::lookahead_peek] returns true.
    ///
    /// # Errors
    /// If [Lookahead::lookahead_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> ::syn::Result<Option<Self>>
    where
        Self: Parse,
    {
        if Self::lookahead_peek(lookahead) {
            input.parse().map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<T, P> Lookahead for Punctuated<T, P>
where
    T: Lookahead,
{
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        T::lookahead_peek(lookahead)
    }
}

/// Extension trait for [ParseBuffer] using [Lookahead].
pub trait ParseBufferExt {
    /// Parse the type T if [Lookahead::lookahead_peek] returns true.
    ///
    /// # Errors
    /// If [Lookahead::lookahead_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn lookahead_parse<T>(&self, lookahead: &Lookahead1) -> ::syn::Result<Option<T>>
    where
        T: Lookahead + Parse;

    /// Parse the type T if [Lookahead::input_peek] returns true.
    /// # Errors
    /// If [Lookahead::input_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn optional_parse<T>(&self) -> ::syn::Result<Option<T>>
    where
        T: Lookahead + Parse;

    /// Parse the type T if [Lookahead::lookahead_peek] returns true.
    ///
    /// Then replace the [Lookahead1], if parse was successfull.
    ///
    /// # Errors
    /// If [Lookahead::lookahead_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn forward_parse<'s, T>(&'s self, lookahead: &mut Lookahead1<'s>) -> ::syn::Result<Option<T>>
    where
        T: Lookahead + Parse;

    /// Use [Lookahead::lookahead_peek] to decide if a [Punctuated] should be parsed
    /// and parse it if so.
    ///
    /// # Errors
    /// If [Lookahead::lookahead_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn lookahead_parse_terminated<T, P>(
        &self,
        lookahead: &Lookahead1,
    ) -> ::syn::Result<Option<Punctuated<T, P>>>
    where
        T: Lookahead + Parse,
        P: Parse;

    /// Use [Lookahead::lookahead_peek] to decide if a [Punctuated] should be parsed
    /// and parse it if so using the parsing function for values.
    ///
    /// # Errors
    /// If [Lookahead::lookahead_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn lookahead_parse_terminated_with<'a, T, P>(
        &'a self,
        lookahead: &Lookahead1,
        parser: fn(ParseStream<'a>) -> ::syn::Result<T>,
    ) -> ::syn::Result<Option<Punctuated<T, P>>>
    where
        T: Lookahead,
        P: Parse;
}

impl ParseBufferExt for ParseBuffer<'_> {
    #[inline]
    fn lookahead_parse<T>(&self, lookahead: &Lookahead1) -> syn::Result<Option<T>>
    where
        T: Lookahead + Parse,
    {
        T::lookahead_parse(self, lookahead)
    }

    #[inline]
    fn optional_parse<T>(&self) -> syn::Result<Option<T>>
    where
        T: Lookahead + Parse,
    {
        self.lookahead_parse(&self.lookahead1())
    }

    #[inline]
    fn forward_parse<'s, T>(&'s self, lookahead: &mut Lookahead1<'s>) -> syn::Result<Option<T>>
    where
        T: Lookahead + Parse,
    {
        match self.lookahead_parse(lookahead) {
            result @ (Err(..) | Ok(None)) => result,
            result @ Ok(Some(..)) => {
                *lookahead = self.lookahead1();
                result
            }
        }
    }

    fn lookahead_parse_terminated<T, P>(
        &self,
        lookahead: &Lookahead1,
    ) -> syn::Result<Option<Punctuated<T, P>>>
    where
        T: Lookahead + Parse,
        P: Parse,
    {
        if T::lookahead_peek(lookahead) {
            Punctuated::parse_terminated(self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn lookahead_parse_terminated_with<'a, T, P>(
        &'a self,
        lookahead: &Lookahead1,
        parser: fn(ParseStream<'a>) -> syn::Result<T>,
    ) -> syn::Result<Option<Punctuated<T, P>>>
    where
        T: Lookahead,
        P: Parse,
    {
        if T::lookahead_peek(lookahead) {
            Punctuated::parse_terminated_with(self, parser).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// Crate keywords implementing [Lookahead]
#[macro_export]
macro_rules! lookahead_keywords {
    ($(#[ $($attr:tt)* ])* $vis:vis $mod:ident { $($kw:ident),* $(,)? }) => {
        $(#[$($attr)*])*
        mod $mod {
        $(
        $crate::__private::custom_keyword!($kw);
        )*
        }
        const _: () = {$(
        impl $crate::Lookahead for $mod::$kw {
            fn lookahead_peek(lookahead: &$crate::__private::Lookahead1) -> bool {
                if lookahead.peek($mod::$kw) {
                    return true;
                }
                false
            }

            fn input_peek(input: $crate::__private::ParseStream) -> bool {
                if input.peek($mod::$kw) {
                    return true;
                }
                false
            }

            fn lookahead_parse(
                input: $crate::__private::ParseStream,
                lookahead: &$crate::__private::Lookahead1
            ) -> $crate::__private::syn::Result<Option<Self>> {
                if lookahead.peek($mod::$kw) {
                    return input.parse().map(Some);
                }
                Ok(None)
            }
        }
        )*};
    };
}

/// Implement [Lookahead] and [Parse] for an enum with variants implementing [Lookahead] and [Parse].
#[macro_export]
macro_rules! lookahead_parse_enum {
    ($name:path {$($variant_name:ident: $variant_ty:ty),* $(,)?}) => {
        impl $crate::__private::Parse for $name {
            fn parse(input: $crate::__private::ParseStream) -> $crate::__private::syn::Result<Self> {
                let lookahead = input.lookahead1();
                if let Some(value) = $crate::Lookahead::lookahead_parse(input, &lookahead)? {
                    Ok(value)
                } else {
                    Err(lookahead.error())
                }
            }
        }

        impl $crate::Lookahead for $name {
            fn lookahead_peek(lookahead: &$crate::__private::Lookahead1) -> bool {
                $(
                if <$variant_ty as $crate::Lookahead>::lookahead_peek(lookahead) {
                    return true;
                }
                )*
                false
            }

            fn input_peek(input: $crate::__private::ParseStream) -> bool {
                $(
                if <$variant_ty as $crate::Lookahead>::input_peek(input) {
                    return true;
                }
                )*
                false
            }

            fn lookahead_parse(
                input: $crate::__private::ParseStream,
                lookahead: &$crate::__private::Lookahead1
            ) -> $crate::__private::syn::Result<Option<Self>> {
                $(
                if let Some(value) = <$variant_ty as $crate::Lookahead>::lookahead_parse(input, lookahead)? {
                    return Ok(Some(Self::$variant_name(value)));
                }
                )*
                Ok(None)
            }
        }
    };
}

#[doc(hidden)]
mod impl_for_peek {

    /// Implement lookahead for types implementing peek
    macro_rules! impl_for_peek {
        ($($ident:ident),*) => {$(
            impl Lookahead for $ident {
                fn lookahead_peek(lookahead: &Lookahead1) -> bool {
                    lookahead.peek($ident)
                }

                fn input_peek(input: ParseStream) -> bool {
                    input.peek($ident)
                }

                fn lookahead_parse(
                    input: ParseStream,
                    lookahead: &Lookahead1,
                ) -> ::syn::Result<Option<Self>>
                where
                    Self: Parse,
                {
                    if lookahead.peek($ident) {
                        input.parse().map(Some)
                    } else {
                        Ok(None)
                    }
                }
            }
        )*};
    }

    use ::syn::{
        Ident, LitBool, LitChar, LitInt, LitStr,
        ext::IdentExt,
        parse::ParseBuffer,
        token::{Colon, Comma, Dot, Eq},
    };
    use syn::parse::{Lookahead1, Parse, ParseStream};

    use crate::Lookahead;

    impl_for_peek!(LitBool, LitChar, LitInt, LitStr, Colon, Comma, Dot, Eq);

    impl Lookahead for Ident {
        fn lookahead_peek(lookahead: &Lookahead1) -> bool {
            lookahead.peek(Ident)
        }

        fn input_peek(input: syn::parse::ParseStream) -> bool {
            input.peek(Ident)
        }

        fn lookahead_parse(input: &ParseBuffer, lookahead: &Lookahead1) -> syn::Result<Option<Self>>
        where
            Self: syn::parse::Parse,
        {
            if lookahead.peek(Ident) {
                input.call(Ident::parse_any).map(Some)
            } else {
                Ok(None)
            }
        }
    }
}
