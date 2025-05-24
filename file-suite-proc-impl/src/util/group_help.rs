//! Helpers for group parsing.

use ::quote::ToTokens;
use ::syn::{MacroDelimiter, parse::Parse};

use crate::{
    macro_delimited,
    util::{
        delimited::MacroDelimExt,
        ensure_empty,
        lookahead_parse::{LookaheadParse, ParseWrap},
    },
};

/// A delimited empty group, {}, [], ().
#[derive(Debug, Clone)]
pub struct EmptyGroup {
    /// Group delimiter.
    pub delim: MacroDelimiter,
}

impl LookaheadParse for EmptyGroup {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        if MacroDelimiter::lookahead_peek(lookahead) {
            let content;
            let delim = macro_delimited!(content in input);
            ensure_empty(&content)?;
            Ok(Some(Self { delim }))
        } else {
            Ok(None)
        }
    }
}

impl ToTokens for EmptyGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.delim.surround(tokens, |_| ())
    }
}

/// A delimited group, {}, [], (), which may be empty.
#[derive(Debug, Clone)]
pub struct GroupOption<T> {
    /// Group delimiter.
    pub delim: MacroDelimiter,
    /// Content of group, may be empty.
    pub content: Option<T>,
}

impl<T> GroupOption<ParseWrap<T>>
where
    T: LookaheadParse,
{
    /// Get a reference to the value wrapped by ParseWrap, if any.
    pub fn unwrap_parsed(&self) -> Option<&T> {
        self.content.as_ref().map(|content| &content.0)
    }
}

impl<T> ToTokens for GroupOption<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { delim, content } = self;
        delim.surround(tokens, |tokens| content.to_tokens(tokens))
    }
}

impl<T> LookaheadParse for GroupOption<T>
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

        Ok(Some(Self { delim, content }))
    }
}

/// A delimited group, {}, [], (), which contains a single value.
#[derive(Debug, Clone)]
pub struct GroupSingle<T> {
    /// Group delimiter.
    pub delim: MacroDelimiter,
    /// Content of group.
    pub content: T,
}

impl<T> ToTokens for GroupSingle<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { delim, content } = self;
        delim.surround(tokens, |tokens| content.to_tokens(tokens));
    }
}

impl<T> LookaheadParse for GroupSingle<T>
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

        Ok(Some(Self { delim, content }))
    }
}
