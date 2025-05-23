//! [alias] impl.

use ::std::borrow::Cow;

use ::quote::ToTokens;
use ::syn::{Token, parse::Parse};

use crate::{
    array_expr::{
        function::{Call, Function, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::group_help::GroupSingle,
};

function_struct!(
    /// Set an alias.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    alias {
        /// Specification for what functions to chain for alias.
        spec: GroupSingle<Spec>,
    }
);

impl ToCallable for alias {
    type Call = AliasCallable;

    fn to_callable(&self) -> Self::Call {
        AliasCallable {
            chain: self
                .spec
                .content
                .chain
                .iter()
                .map(|(_, f)| f.to_callable())
                .collect(),
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
    chain: Vec<(Option<Token![.]>, Function)>,
}

impl Parse for Spec {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            chain: Function::parse_chain(input)?,
        })
    }
}

impl ToTokens for Spec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { chain } = self;

        for (dot, func) in chain {
            dot.to_tokens(tokens);
            func.to_tokens(tokens);
        }
    }
}
