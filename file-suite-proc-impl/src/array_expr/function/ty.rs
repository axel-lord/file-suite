//! [Ty] impl.

use ::quote::ToTokens;

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        value_array::ValueArray,
    },
    util::{
        group_help::GroupSingle,
        lookahead_parse::{LookaheadParse, ParseWrap},
    },
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
    /// Specification for which type to apply.
    ty: GroupSingle<ParseWrap<Ty>>,
}

impl ToCallable for Type {
    type Call = TyKind;

    fn to_callable(&self) -> Self::Call {
        self.ty.content.0.kind
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
                Ok(Self {
                    kw: input.parse()?,
                    ty: input.call(LookaheadParse::parse)?,
                })
            })
            .transpose()
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, ty } = self;
        kw.to_tokens(tokens);
        ty.to_tokens(tokens);
    }
}
