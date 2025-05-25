//! [DelimitedOption] impl.

use ::quote::ToTokens;
use ::syn::{MacroDelimiter, parse::Parse};

use crate::{
    macro_delimited,
    util::{
        delimited::MacroDelimExt, ensure_empty, lookahead_parse::LookaheadParse,
        parse_wrap::ParseWrap,
    },
};

/// A delimited group, {}, [], (), which may be empty.
#[derive(Debug, Clone)]
pub struct DelimitedOption<T> {
    /// Group delimiter.
    pub delim: MacroDelimiter,
    /// Content of group, may be empty.
    pub inner: Option<T>,
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

impl<T> DelimitedOption<ParseWrap<T>>
where
    T: LookaheadParse,
{
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

impl<T> LookaheadParse for DelimitedOption<T>
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

        let content = if !content.is_empty() {
            let parsed_content = content.parse()?;
            ensure_empty(&content)?;
            Some(parsed_content)
        } else {
            None
        };

        Ok(Some(Self {
            delim,
            inner: content,
        }))
    }
}
