//! Proc macro utilities.

use ::syn::parse::{End, ParseStream};

pub(crate) use self::{
    delimited::{MacroDelimExt, macro_delimited},
    kw_kind::kw_kind,
};

pub use self::lookahead::TokenLookahead;

mod lookahead;

mod kw_kind;

mod delimited;

mod to_tokens_enum;

pub mod lookahead_parse;

pub mod fold_tokens;

pub mod tcmp;

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
