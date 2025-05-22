//! Proc macros used by file-suite.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::parse::{Parse, Parser};

use crate::{
    array_expr::{ArrayExprPaste, Node},
    util::fold_tokens::fold_token_stream,
};

mod run;

pub mod array_expr;

pub mod util;

/// Find array expression in input tokens and compute them, replacing them with their result.
///
/// # Errors
/// If the expression cannot be parsed.
/// Or if it cannot be computed.
pub fn array_expr_paste(input: TokenStream) -> ::syn::Result<TokenStream> {
    fold_token_stream(&mut ArrayExprPaste, input)
}

/// Compute array expression expressed as macro input.
///
/// # Errors
/// If the expression cannot be parsed.
/// Or if it cannot be computed.
pub fn array_expr(input: TokenStream) -> ::syn::Result<TokenStream> {
    let mut tokens = TokenStream::default();
    for value in Node::parse.parse2(input)?.to_array_expr().compute()? {
        value.try_to_typed()?.to_tokens(&mut tokens);
    }
    Ok(tokens)
}

/// Derive Run for an enum with only single field or empty variants.
///
/// # Errors
/// If given invalid input.
pub fn derive_run(input: TokenStream) -> ::syn::Result<TokenStream> {
    run::derive_run(::syn::parse2(input)?)
}
