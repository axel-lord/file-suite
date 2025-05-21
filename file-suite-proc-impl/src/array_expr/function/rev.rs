//! [Rev] imp.

use ::quote::ToTokens;
use ::syn::MacroDelimiter;

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        value_array::ValueArray,
    },
    util::{MacroDelimExt, ensure_empty, lookahead_parse::LookaheadParse, macro_delimited},
};

#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(rev);
}

/// Reverse array.
#[derive(Debug, Clone)]
pub struct Rev {
    /// Rev keyword.
    kw: kw::rev,
    /// Optional delimiter.
    delim: Option<MacroDelimiter>,
}

impl ToCallable for Rev {
    type Call = RevCallable;

    fn to_callable(&self) -> Self::Call {
        RevCallable
    }
}

/// [Call] implementor for [Rev].
#[derive(Debug, Clone, Copy)]
pub struct RevCallable;

impl Call for RevCallable {
    fn call(&self, input: ValueArray) -> syn::Result<ValueArray> {
        let mut values = input;
        values.reverse();
        Ok(values)
    }
}

impl LookaheadParse for Rev {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(kw::rev)
            .then(|| {
                let kw = input.parse()?;
                let mut delim = None;
                if MacroDelimiter::input_peek(input) {
                    let content;
                    delim = Some(macro_delimited!(content in input));
                    ensure_empty(&content)?
                }

                Ok(Self { kw, delim })
            })
            .transpose()
    }
}

impl ToTokens for Rev {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim } = self;
        kw.to_tokens(tokens);
        if let Some(delim) = delim {
            delim.surround(tokens, |_| ());
        }
    }
}
