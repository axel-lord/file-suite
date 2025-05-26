//! Proc macro utilities.

use ::std::str::FromStr;

use ::proc_macro2::{Span, TokenStream};
use ::quote::quote_spanned;
use ::syn::parse::{End, Parse, ParseStream};

pub(crate) use self::kw_kind::kw_kind;

pub use self::lookahead::TokenLookahead;

mod kw_kind;
mod lookahead;
mod to_tokens_macros;

pub mod delimited;
pub mod fold_tokens;
pub mod lookahead_parse;
pub mod neverlike;
pub mod parse_wrap;
pub mod spanned_int;
pub mod tcmp;

pub mod group_help {
    //! Helpers for group parsing.

    pub use self::{
        delimited::Delimited, delimited_option::DelimitedOption, empty_delimited::EmptyDelimited,
        optional_delimited::OptionalDelimited,
    };

    mod delimited;
    mod delimited_option;
    mod empty_delimited;
    mod optional_delimited;
}

/// Ensure a [ParseStream] has reached it's end.
///
/// # Errors
/// If the parse buffer is not empty.
pub fn ensure_empty(input: ParseStream) -> ::syn::Result<()> {
    let lookahead = input.lookahead1();
    lookahead
        .peek(End)
        .then_some(())
        .ok_or_else(|| lookahead.error())
}

/// Parse a string slice with given span for input.
///
/// # Errors
/// If the string slice cannot be parsed to the given type.
pub fn spanned_parse_str<T>(span: Span, input: &str) -> ::syn::Result<T>
where
    T: Parse,
{
    let tokens = TokenStream::from_str(input).map_err(|err| ::syn::Error::new(span, err))?;
    let tokens = quote_spanned! {span=> #tokens};

    ::syn::parse2(tokens)
}
