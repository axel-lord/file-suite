//! [OptionalDelimited] impl.

use ::file_suite_proc_lib::{Lookahead, ensure_empty, macro_delim::MacroDelimExt, macro_delimited};
use ::quote::ToTokens;
use ::syn::{MacroDelimiter, parse::Parse};

use crate::function::ToCallable;

/// A delimited group, {}, [], (), which may be empty, and may not exist. As such whilst it
/// implements [LookaheadParse], `Ok(None)` will never be returned by it.
#[derive(Debug, Clone)]
pub struct OptionalDelimited<T> {
    /// Group delimiter.
    pub delim: Option<MacroDelimiter>,
    /// Content of group, may be empty.
    pub inner: Option<T>,
}

impl<T> ToCallable for OptionalDelimited<T>
where
    Option<T>: ToCallable,
{
    type Call = <Option<T> as ToCallable>::Call;

    fn to_callable(&self) -> Self::Call {
        self.inner.to_callable()
    }
}

impl<T> ToTokens for OptionalDelimited<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { delim, inner } = self;
        if let Some(delim) = delim {
            delim.surround(tokens, |tokens| inner.to_tokens(tokens));
        }
    }
}

impl<T> Lookahead for OptionalDelimited<T> {
    fn lookahead_peek(_: &syn::parse::Lookahead1) -> bool {
        true
    }
}

impl<T> Parse for OptionalDelimited<T>
where
    T: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if MacroDelimiter::input_peek(input) {
            let content;
            let delim = macro_delimited!(content in input);

            let inner = if content.is_empty() {
                None
            } else {
                let inner = content.parse()?;
                ensure_empty(&content)?;
                Some(inner)
            };

            Ok(Self {
                delim: Some(delim),
                inner,
            })
        } else {
            Ok(Self {
                delim: None,
                inner: None,
            })
        }
    }
}
