//! Proc macro export.

use ::proc_macro::TokenStream;

/// Attribute to put on functions that should be ran in a blocking thread.
#[proc_macro_attribute]
pub fn in_blocking(attr: TokenStream, item: TokenStream) -> TokenStream {
    wrap_blocking_attr_impl::in_blocking(attr.into(), item.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
