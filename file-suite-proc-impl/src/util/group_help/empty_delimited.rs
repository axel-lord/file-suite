//! [EmptyDelimited] impl.

use ::file_suite_proc_lib::{Lookahead, ensure_empty, macro_delim::MacroDelimExt, macro_delimited};
use ::quote::ToTokens;
use ::syn::{MacroDelimiter, parse::Parse};
use syn::parse::{Lookahead1, ParseStream};

/// A delimited empty group, {}, [], ().
#[derive(Debug, Clone)]
pub struct EmptyDelimited {
    /// Group delimiter.
    pub delim: MacroDelimiter,
}

impl Lookahead for EmptyDelimited {
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        MacroDelimiter::lookahead_peek(lookahead)
    }
}

impl Parse for EmptyDelimited {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let delim = macro_delimited!(content in input);
        ensure_empty(&content)?;
        Ok(Self { delim })
    }
}

impl ToTokens for EmptyDelimited {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.delim.surround(tokens, |_| ())
    }
}
