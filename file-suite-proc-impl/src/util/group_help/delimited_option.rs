//! [DelimitedOption] impl.

use ::file_suite_proc_lib::{Lookahead, ensure_empty};
use ::quote::ToTokens;
use ::syn::{MacroDelimiter, parse::Parse};

use crate::{
    array_expr::function::ToCallable,
    macro_delimited,
    util::{delimited::MacroDelimExt, parse_wrap::ParseWrap},
};

/// A delimited group, {}, [], (), which may be empty.
#[derive(Debug, Clone)]
pub struct DelimitedOption<T> {
    /// Group delimiter.
    pub delim: MacroDelimiter,
    /// Content of group, may be empty.
    pub inner: Option<T>,
}

impl<T> ToCallable for DelimitedOption<T>
where
    Option<T>: ToCallable,
{
    type Call = <Option<T> as ToCallable>::Call;

    fn to_callable(&self) -> Self::Call {
        self.inner.to_callable()
    }
}

impl<T> AsRef<Option<T>> for DelimitedOption<T> {
    fn as_ref(&self) -> &Option<T> {
        &self.inner
    }
}

impl<T> AsMut<Option<T>> for DelimitedOption<T> {
    fn as_mut(&mut self) -> &mut Option<T> {
        &mut self.inner
    }
}

impl<T> From<DelimitedOption<T>> for Option<T> {
    fn from(value: DelimitedOption<T>) -> Self {
        value.inner
    }
}

impl<T> DelimitedOption<ParseWrap<T>> {
    /// Get a reference to the value wrapped by ParseWrap, if any.
    pub fn unwrap_parsed(&self) -> Option<&T> {
        self.inner.as_ref().map(|content| &content.inner)
    }
}

impl<T> ToTokens for DelimitedOption<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            delim,
            inner: content,
        } = self;
        delim.surround(tokens, |tokens| content.to_tokens(tokens))
    }
}

impl<T> Lookahead for DelimitedOption<T> {
    fn lookahead_peek(lookahead: &syn::parse::Lookahead1) -> bool {
        MacroDelimiter::lookahead_peek(lookahead)
    }
}

impl<T> Parse for DelimitedOption<T>
where
    T: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let delim = macro_delimited!(content in input);

        let inner = if !content.is_empty() {
            let parsed_content = content.parse()?;
            ensure_empty(&content)?;
            Some(parsed_content)
        } else {
            None
        };

        Ok(Self { delim, inner })
    }
}
