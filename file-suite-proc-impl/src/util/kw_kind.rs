//! Macro to create an enum and a wrapper for parsing keywords.

/// Create an enum and a wrapper for parsing a set of keywords.
macro_rules! kw_kind {
    (
        $(#[doc = $wr_doc:expr])?
        $([ $($wr_ty_attr:tt)* ])*
        $wr_nm:ident
        $(( $($wr_add_derive:ident)? ))?
        $(#[doc = $ki_doc:expr])?
        $([ $($ki_ty_attr:tt)* ])*
        $ki_nm:ident
        $(( $($ki_add_derive:ident)? ))?
        {$(
            $(#[doc = $va_doc:expr])?
            $([$($attr:tt)*])*
            $kw_nm:ident
        ),+ $(,)?}) => {

        $(#[doc = $wr_doc])*
        #[derive(Clone, Copy, Debug $($(, $wr_add_derive)*)*)]
        $( #[$($wr_ty_attr)*] )*
        pub struct $wr_nm {
            #[doc = "Keyword variant that was parsed."]
            pub kind: $ki_nm,
            #[doc = "Span of parsed keyword."]
            pub span: ::proc_macro2::Span,
        }


        $(#[doc = $ki_doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash $($(, $ki_add_derive)*)*)]
        $( #[$($ki_ty_attr)*] )*
        pub enum $ki_nm {$(
            $(#[doc = $va_doc])*
            $(#[$($attr)*])*
            $kw_nm,
        )*}

        const _: () = {

        #[doc = "Implementation keywords"]
        mod kw {$(
            ::syn::custom_keyword!($kw_nm);
        )*}

        impl $crate::util::lookahead_parse::LookaheadParse for $wr_nm {
            fn lookahead_parse(
                input: ::syn::parse::ParseStream,
                lookahead: &::syn::parse::Lookahead1
            ) -> ::syn::Result<Option<Self>> {
                let (kind, span) = $(if lookahead.peek(kw::$kw_nm) {
                    ($ki_nm::$kw_nm, <kw::$kw_nm as ::syn::parse::Parse>::parse(input)?.span)
                } else)* {
                    return Ok(None);
                };

                Ok(Some(Self { kind, span }))
            }
        }

        impl ::quote::ToTokens for $wr_nm {
            fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                match self.kind {$(
                    $ki_nm::$kw_nm => kw::$kw_nm(self.span).to_tokens(tokens),
                )*}
            }
        }

        impl ::core::ops::Deref for $wr_nm {
            type Target = $ki_nm;

            fn deref(&self) -> &Self::Target {
                &self.kind
            }
        }

        impl ::core::ops::DerefMut for $wr_nm {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.kind
            }
        }

        impl ::core::convert::AsRef<$ki_nm> for $wr_nm {
            fn as_ref(&self) -> &$ki_nm {
                self
            }
        }

        impl ::core::convert::AsMut<$ki_nm> for $wr_nm {
            fn as_mut(&mut self) -> &mut $ki_nm {
                self
            }
        }

        impl ::core::convert::From<$wr_nm> for $ki_nm {
            fn from(value: $wr_nm) -> Self {
                value.kind
            }
        }

        };
    };
}

pub(crate) use kw_kind;
