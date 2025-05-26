//! [Ty] impl.

use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::array_expr::{
    function::{Call, ToCallable},
    storage::Storage,
    value::{Ty, TyKind},
    value_array::ValueArray,
};

/// Convert type of array.
#[derive(Debug, Clone)]
pub struct TyArgs {
    /// Type to convert to.
    ty: Ty,
}

impl Parse for TyArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ty: input.call(Ty::parse)?,
        })
    }
}

impl ToTokens for TyArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ty } = self;
        ty.to_tokens(tokens);
    }
}

impl ToCallable for TyArgs {
    type Call = TyKind;

    fn to_callable(&self) -> Self::Call {
        self.ty.kind
    }
}

impl Call for TyKind {
    fn call(&self, mut input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        for value in &mut input {
            value.ty = *self;
        }
        Ok(input)
    }
}
