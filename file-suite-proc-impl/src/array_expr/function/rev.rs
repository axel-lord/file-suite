//! [Rev] imp.

use ::quote::ToTokens;

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        value_array::ValueArray,
    },
    util::{group_help::EmptyGroup, lookahead_parse::LookaheadParse},
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
    delim: Option<EmptyGroup>,
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
                Ok(Self {
                    kw: input.parse()?,
                    delim: input.call(LookaheadParse::optional_parse)?,
                })
            })
            .transpose()
    }
}

impl ToTokens for Rev {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim } = self;
        kw.to_tokens(tokens);
        delim.to_tokens(tokens);
    }
}
