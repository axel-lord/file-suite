//! Proc macro utilities.

mod to_tokens_macros;

pub mod __private;

pub mod from_arg;
pub mod kw_kind;
pub mod lookahead;
pub mod macro_delim;
pub mod neverlike;
pub mod spanned_int;
pub mod to_arg;
pub mod tokens_rc;

pub use crate::{
    from_arg::{ArgTy, FromArg},
    lookahead::Lookahead,
    to_arg::ToArg,
    tokens_rc::TokensRc,
};

/// Parsable version of [End][::syn::parse::End].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct End;

impl Lookahead for End {
    fn lookahead_peek(lookahead: &syn::parse::Lookahead1) -> bool {
        lookahead.peek(::syn::parse::End)
    }

    fn input_peek(input: syn::parse::ParseStream) -> bool {
        input.peek(::syn::parse::End)
    }

    fn lookahead_parse(
        _input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>>
    where
        Self: ::syn::parse::Parse,
    {
        if lookahead.peek(::syn::parse::End) {
            Ok(Some(Self))
        } else {
            Ok(None)
        }
    }
}

impl ::syn::parse::Parse for End {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        match lookahead::ParseBufferExt::lookahead_parse(input, &lookahead)? {
            Some(end) => Ok(end),
            None => Err(lookahead.error()),
        }
    }
}

impl ::quote::ToTokens for End {
    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {}
}

/// Ensure a [ParseStream][::syn::parse::ParseStream] has reached it's end.
///
/// # Errors
/// If the parse buffer is not empty.
pub fn ensure_empty(input: ::syn::parse::ParseStream) -> ::syn::Result<()> {
    let lookahead = input.lookahead1();
    lookahead
        .peek(::syn::parse::End)
        .then_some(())
        .ok_or_else(|| lookahead.error())
}

/// Parse a string slice with given span for input.
///
/// # Errors
/// If the string slice cannot be parsed to the given type.
pub fn spanned_parse_str<T>(span: ::proc_macro2::Span, input: &str) -> ::syn::Result<T>
where
    T: ::syn::parse::Parse,
{
    let tokens = <::proc_macro2::TokenStream as ::std::str::FromStr>::from_str(input)
        .map_err(|err| ::syn::Error::new(span, err))?;
    let tokens = ::quote::quote_spanned! {span=> #tokens};

    ::syn::parse2(tokens)
}
