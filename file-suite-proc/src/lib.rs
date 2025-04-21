//! Proc macro export.

use ::proc_macro::TokenStream;

/// Convert between case, and string literal/ident.
/// Also handles concatenation respecting given case.
///
/// # Errors
/// If given invalid input, be it illegal literals or the wrong pattern.
#[proc_macro]
pub fn kebab(input: TokenStream) -> TokenStream {
    ::file_suite_proc_impl::kebab(input.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

/// Find kebab expressions in input tokens and replace them with kebabed result.
///
/// # Errors
/// If the kebab expressions are malformed.
#[proc_macro]
pub fn kebab_paste(input: TokenStream) -> TokenStream {
    ::file_suite_proc_impl::kebab_paste(input.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
