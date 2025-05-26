//! [stairs] impl.

use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::array_expr::{
    function::{Call, FunctionCallable, FunctionChain, ToCallable},
    storage::Storage,
    value_array::ValueArray,
};

/// Run array through input chain in stairs such that an array [A, B, C]
/// Results in the cain being called on [A], [A, B] and [A, B, C].
#[derive(Debug, Clone)]
pub struct StairsArgs {
    /// Chain to call on arrays.
    chain: FunctionChain,
}

impl Parse for StairsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            chain: input.parse()?,
        })
    }
}

impl ToTokens for StairsArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { chain } = self;
        chain.to_tokens(tokens);
    }
}

impl ToCallable for StairsArgs {
    type Call = StairsCallable;

    fn to_callable(&self) -> Self::Call {
        StairsCallable {
            chain: self.chain.to_call_chain(),
        }
    }
}

/// [Call] implementor for [stairs].
#[derive(Debug, Clone)]
pub struct StairsCallable {
    /// Chain to call.
    chain: Vec<FunctionCallable>,
}

impl Call for StairsCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        if array.is_empty() {
            return storage.with_local_layer(|storage| {
                FunctionChain::call_chain(&self.chain, array, storage)
            });
        }

        let mut output_array = ValueArray::new();
        for end in 1..=array.len() {
            output_array.extend(storage.with_local_layer(|storage| {
                FunctionChain::call_chain(
                    &self.chain,
                    array[..end].iter().cloned().collect(),
                    storage,
                )
            })?);
        }

        Ok(output_array)
    }
}
