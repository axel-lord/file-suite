//! [stairs] impl.

use crate::{
    array_expr::{
        function::{Call, FunctionCallable, FunctionChain, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::group_help::Delimited,
};

function_struct!(
    /// Run array through input chain in stairs such that an array [A, B, C]
    /// Results in the cain being called on [A], [A, B] and [A, B, C].
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    stairs {
        /// Chain to call on arrays.
        chain: Delimited<FunctionChain>,
    }
);

impl ToCallable for stairs {
    type Call = StairsCallable;

    fn to_callable(&self) -> Self::Call {
        StairsCallable {
            chain: self.chain.inner.to_call_chain(),
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
