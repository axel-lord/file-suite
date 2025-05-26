//! [alias] impl.

use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::array_expr::{
    function::{Call, Function, FunctionChain, ToCallable},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor for [AliasArgs].
#[derive(Debug, Clone)]
pub struct AliasCallable {
    /// Function chain to store.
    chain: Vec<<Function as ToCallable>::Call>,
}

impl Call for AliasCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        for value in array {
            storage.set_alias(value.into(), self.chain.clone());
        }

        Ok(ValueArray::new())
    }
}

impl ToCallable for AliasArgs {
    type Call = AliasCallable;

    fn to_callable(&self) -> Self::Call {
        AliasCallable {
            chain: self.chain.to_call_chain(),
        }
    }
}

/// Alias input specification.
#[derive(Debug, Clone)]
pub struct AliasArgs {
    /// Function chain for alias.
    chain: FunctionChain,
}

impl Parse for AliasArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            chain: FunctionChain::parse(input)?,
        })
    }
}

impl ToTokens for AliasArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { chain } = self;
        chain.to_tokens(tokens);
    }
}
