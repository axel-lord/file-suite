//! [Ty] impl.

use ::quote::ToTokens;
use ::syn::MacroDelimiter;

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        value_array::ValueArray,
    },
    util::{MacroDelimExt, ensure_empty, lookahead_parse::LookaheadParse, macro_delimited},
    value::{Ty, TyKind},
};

#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(ty);
}

/// Convert type of array.
#[derive(Debug, Clone)]
pub struct Type {
    /// Type keyword.
    kw: kw::ty,
    /// Delim for spec.
    delim: MacroDelimiter,
    /// Specification for which type to apply.
    ty: Ty,
}

impl ToCallable for Type {
    type Call = TyKind;

    fn to_callable(&self) -> Self::Call {
        self.ty.kind
    }
}

impl Call for TyKind {
    fn call(&self, mut input: ValueArray) -> syn::Result<ValueArray> {
        for value in &mut input {
            value.set_ty(*self);
        }
        Ok(input)
    }
}

impl LookaheadParse for Type {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(kw::ty)
            .then(|| {
                let content;
                let value = Self {
                    kw: input.parse()?,
                    delim: macro_delimited!(content in input),
                    ty: content.call(Ty::parse)?,
                };

                ensure_empty(&content)?;

                Ok(value)
            })
            .transpose()
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim, ty } = self;
        kw.to_tokens(tokens);
        delim.surround(tokens, |tokens| ty.to_tokens(tokens));
    }
}
