//! [StairsCallable] impl.

use ::file_suite_proc_lib::{ArgTy, FromArg};

use crate::{
    function::{Call, Callable, Function, FunctionChain},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor for [StairsArgs].
#[derive(Debug, Clone)]
pub struct StairsCallable {
    /// Chain to call.
    chain: Vec<Callable<Function>>,
}

impl FromArg for StairsCallable {
    type Factory = FunctionChain;

    fn from_arg(chain: ArgTy<Self>) -> Self {
        Self { chain }
    }
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

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use ::quote::quote;

    use crate::array_expr;

    #[test]
    fn stairs() {
        let expr = quote! { A -> repeat(3).stairs(join).ty(ident) };
        let expected = quote! { A AA AAA };
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());
    }
}
