//! Function to run a chain for every array value.

use ::file_suite_proc_lib::{ArgTy, FromArg};

use crate::{
    function::{Call, Callable, Function, FunctionChain},
    storage::Storage,
    value_array::ValueArray,
};

/// Run a function chain for every array value.
/// Same as chunks with a chunk size of 1.
#[derive(Debug, Clone)]
pub struct ForEachCallable {
    /// Chain to run.
    chain: Vec<Callable<Function>>,
}

impl FromArg for ForEachCallable {
    type Factory = FunctionChain;

    fn from_arg(chain: ArgTy<Self>) -> Self {
        Self { chain }
    }
}

impl Call for ForEachCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        let mut out = ValueArray::new();

        for value in array {
            out.extend(storage.with_local_layer(|storage| {
                FunctionChain::call_chain(&self.chain, ValueArray::from_value(value), storage)
            })?);
        }

        Ok(out)
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::test::assert_arr_expr;

    #[test]
    fn chunks() {
        assert_arr_expr!(
            { A B C -> for_each( chain(_).join ).ty(ident) },
            { A_ B_ C_ }
        );
    }
}
