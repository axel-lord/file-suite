//! Proc macro export.

use ::proc_macro::TokenStream;

/// Find array expressions in input token and replace them with computed result.
#[proc_macro]
pub fn array_expr_paste(input: TokenStream) -> TokenStream {
    ::array_expr::array_expr_paste(input.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

/// Calculate an array expression on input.
#[proc_macro]
pub fn array_expr(input: TokenStream) -> TokenStream {
    ::array_expr::array_expr(input.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[expect(missing_docs)]
#[proc_macro_derive(Run, attributes(run))]
pub fn derive_run(input: TokenStream) -> TokenStream {
    ::run_derive::derive_run(input.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
