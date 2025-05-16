//! Proc macros used by file-suite.

use ::proc_macro2::TokenStream;
use ::syn::parse::Parser;

use crate::{array_expr::ArrayExprPaste, util::fold_tokens::fold_token_stream};

mod kebab;

mod array_expr;

mod run;

pub mod util;

pub mod value;

pub mod typed_value;

/// Convert between cases, for strings and identifiers.
///
/// # Errors
/// If given invalid input, be it illegal literals or the wrong pattern.
pub fn kebab(input: TokenStream) -> ::syn::Result<TokenStream> {
    kebab::parse_kebab.parse2(input)
}

/// Find kebab expressions in input tokens and replace them with kebabed result.
///
/// # Errors
/// If the kebab expressions are malformed.
pub fn kebab_paste(input: TokenStream) -> ::syn::Result<TokenStream> {
    kebab::kebab_paste(input)
}

/// Find array expression in input tokens and compute them, replacing them with their result.
///
/// # Errors
/// If the expression cannot be parsed.
/// Or if it cannot be computed.
pub fn array_expr_paste(input: TokenStream) -> ::syn::Result<TokenStream> {
    fold_token_stream(&mut ArrayExprPaste, input)
}

/// Derive Run for an enum with only single field or empty variants.
///
/// # Errors
/// If given invalid input.
pub fn derive_run(input: TokenStream) -> ::syn::Result<TokenStream> {
    run::derive_run(::syn::parse2(input)?)
}
