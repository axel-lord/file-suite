//! Proc macros used by file-suite.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::parse::Parser;

use crate::array_expr::{ArrayExprPaste, Node, storage::Storage};

mod run;

mod error;

pub mod array_expr;

pub mod util;

pub use error::Error;

/// Crate result type.
pub type Result<T = ()> = ::std::result::Result<T, Error>;

/// Find array expression in input tokens and compute them, replacing them with their result.
///
/// # Errors
/// If the expression cannot be parsed.
/// Or if it cannot be computed.
pub fn array_expr_paste(input: TokenStream) -> ::syn::Result<TokenStream> {
    ::fold_tokens::fold_tokens(
        &mut ArrayExprPaste {
            storage: &mut Storage::initial(),
        },
        input.into(),
    )
}

/// Compute array expression expressed as macro input.
///
/// # Errors
/// If the expression cannot be parsed.
/// Or if it cannot be computed.
pub fn array_expr(input: TokenStream) -> ::syn::Result<TokenStream> {
    let mut tokens = TokenStream::default();
    let mut storage = Storage::initial();
    for node in Node::parse_multiple.parse2(input)? {
        for value in storage
            .with_local_layer(|storage| node.to_array_expr().compute_with_storage(storage))?
        {
            value.try_to_typed()?.to_tokens(&mut tokens);
        }
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
