//! [EmptyDelimited] impl.

use ::quote::ToTokens;
use ::syn::MacroDelimiter;
use syn::parse::{Lookahead1, ParseStream};

use crate::{
    macro_delimited,
    util::{delimited::MacroDelimExt, ensure_empty, lookahead_parse::LookaheadParse},
};

/// A delimited empty group, {}, [], ().
#[derive(Debug, Clone)]
pub struct EmptyDelimited {
    /// Group delimiter.
    pub delim: MacroDelimiter,
}

impl LookaheadParse for EmptyDelimited {
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>> {
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

impl ToTokens for EmptyDelimited {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.delim.surround(tokens, |_| ())
    }
}
