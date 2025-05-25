//! [alias] impl.

use ::std::borrow::Cow;

use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::{
    array_expr::{
        function::{Call, Function, FunctionChain, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::group_help::Delimited,
};

function_struct!(
    /// Set an alias.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    alias {
        /// Specification for what functions to chain for alias.
        spec: Delimited<Spec>,
    }
);

impl ToCallable for alias {
    type Call = AliasCallable;

    fn to_callable(&self) -> Self::Call {
        AliasCallable {
            chain: self.spec.inner.chain.to_call_chain(),
        }
    }
}

/// [Call] implementor for [alias].
#[derive(Debug, Clone)]
pub struct AliasCallable {
    /// Function chain to store.
    chain: Vec<<Function as ToCallable>::Call>,
}

impl Call for AliasCallable {
    fn call(
        &self,
        array: ValueArray,
        storage: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        for value in array {
            storage.set_alias(value.into(), self.chain.clone());
        }

        Ok(ValueArray::new())
    }
}

/// Alias input specification.
#[derive(Debug, Clone)]
pub struct Spec {
    /// Function chain for alias.
    chain: FunctionChain,
}

impl Parse for Spec {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            chain: FunctionChain::parse(input)?,
        })
    }
}

impl ToTokens for Spec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { chain } = self;
        chain.to_tokens(tokens);
    }
}
