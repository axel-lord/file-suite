//! [Delimited] impl.

use ::file_suite_proc_lib::ensure_empty;
use ::quote::ToTokens;
use ::syn::{MacroDelimiter, parse::Parse};

use crate::{
    array_expr::function::ToCallable,
    macro_delimited,
    util::{delimited::MacroDelimExt, lookahead_parse::LookaheadParse},
};

/// A delimited group, {}, [], (), which contains a single value.
#[derive(Debug, Clone)]
pub struct Delimited<T> {
    /// Group delimiter.
    pub delim: MacroDelimiter,
    /// Content of group.
    pub inner: T,
}

impl<T> ToCallable for Delimited<T>
where
    T: ToCallable,
{
    type Call = T::Call;

    fn to_callable(&self) -> Self::Call {
        self.inner.to_callable()
    }
}

impl<T> ToTokens for Delimited<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            delim,
            inner: content,
        } = self;
        delim.surround(tokens, |tokens| content.to_tokens(tokens));
    }
}

impl<T> LookaheadParse for Delimited<T>
where
    T: Parse,
{
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        if !MacroDelimiter::lookahead_peek(lookahead) {
            return Ok(None);
        }

        let content;
        let delim = macro_delimited!(content in input);

        let content = {
            let parsed_content = content.parse()?;
            ensure_empty(&content)?;
            parsed_content
        };

        Ok(Some(Self {
            delim,
            inner: content,
        }))
    }
}
