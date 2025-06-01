//! Proc macro utilities.

pub mod __private;

pub mod from_arg;
pub mod kw_kind;
pub mod lookahead;
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
