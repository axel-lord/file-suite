//! Proc macros used by file-suite.

use ::proc_macro2::TokenStream;

mod run;

/// Derive Run for an enum with only single field or empty variants.
///
/// # Errors
/// If given invalid input.
pub fn derive_run(input: TokenStream) -> ::syn::Result<TokenStream> {
    run::derive_run(::syn::parse2(input)?)
}
