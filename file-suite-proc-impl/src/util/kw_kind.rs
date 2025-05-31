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

        #[derive(Clone, Copy, Debug $($(, $wrap_derive)*)*)]
        $( #[$($wrap_attr)*] )*
        pub struct $wrap_ident {
            #[doc = "Keyword variant that was parsed."]
            pub kind: $kind_ident,
            #[doc = "Span of parsed keyword."]
            pub span: ::proc_macro2::Span,
        }


        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash $($(, $kind_derive)*)*)]
        $( #[$($kind_attr)*] )*
        pub enum $kind_ident {$(
            $(#[$($keyword_attr)*])*
            $keyword_ident,
        )*}

        const _: () = {

        $crate::lookahead_parse_keywords!($($keyword_ident),*);

        impl $crate::util::lookahead_parse::LookaheadParse for $wrap_ident {
            fn lookahead_parse(
                input: ::syn::parse::ParseStream,
                lookahead: &::syn::parse::Lookahead1
            ) -> ::syn::Result<Option<Self>> {
                let (kind, span) = $(if lookahead.peek(kw::$keyword_ident) {
                    ($kind_ident::$keyword_ident, <kw::$keyword_ident as ::syn::parse::Parse>::parse(input)?.span)
                } else)* {
                    return Ok(None);
                };

                Ok(Some(Self { kind, span }))
            }
        }

        impl ::syn::parse::Parse for $wrap_ident {
            #[inline]
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                <Self as $crate::util::lookahead_parse::LookaheadParse>::parse(input)
            }
        }

        impl ::quote::ToTokens for $wrap_ident {
            fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                match self.kind {$(
                    $kind_ident::$keyword_ident => kw::$keyword_ident(self.span).to_tokens(tokens),
                )*}
            }
        }

        impl ::core::ops::Deref for $wrap_ident {
            type Target = $kind_ident;

            fn deref(&self) -> &Self::Target {
                &self.kind
            }
        }

        impl ::core::ops::DerefMut for $wrap_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.kind
            }
        }

        impl ::core::convert::AsRef<$kind_ident> for $wrap_ident {
            fn as_ref(&self) -> &$kind_ident {
                self
            }
        }

        impl ::core::convert::AsMut<$kind_ident> for $wrap_ident {
            fn as_mut(&mut self) -> &mut $kind_ident {
                self
            }
        }

        impl ::core::convert::From<$wrap_ident> for $kind_ident {
            fn from(value: $wrap_ident) -> Self {
                value.kind
            }
        }

        impl ::file_suite_proc_lib::ToArg for $wrap_ident {
            type Arg = $kind_ident;

            fn to_arg(&self) -> Self::Arg { self.kind }
        }

        impl::file_suite_proc_lib::FromArg for $kind_ident {
            type ArgFactory = $wrap_ident;

            fn from_arg(kind: $kind_ident) -> Self {
                kind
            }
        }

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
