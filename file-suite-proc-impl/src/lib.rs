//! Proc macros used by file-suite.

use ::proc_macro2::TokenStream;
use ::syn::parse::Parser;

mod kebab;

mod util;

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

/// Derive Run for an enum with only single field or empty variants.
///
/// # Errors
/// If given invalid input.
pub fn derive_run(input: TokenStream) -> ::syn::Result<TokenStream> {
    run::derive_run(::syn::parse2(input)?)
}

mod run;
