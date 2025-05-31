//! [ForkArgs] impl.

use crate::array_expr::{
    function::{ArgTy, Call, FromArg, FunctionCallable, FunctionChain, chain::FunctionChains},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor fo [ForkArgs].
#[derive(Debug, Clone)]
pub struct ForkCallable {
    /// Function chains.
    chains: Vec<Vec<FunctionCallable>>,
}

impl FromArg for ForkCallable {
    type ArgFactory = FunctionChains;

    fn from_arg(chains: ArgTy<Self>) -> Self {
        Self { chains }
    }
}

impl Call for ForkCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        let mut output_array = ValueArray::new();

        for chain in &self.chains {
            output_array.extend(storage.with_local_layer(|storage| {
                FunctionChain::call_chain(chain, array.clone(), storage)
            })?);
        }

        Ok(output_array)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    fn fork_join() {
        assert_arr_expr!(
            {
                A B C ->
                    .fork {
                        .join(space).ty(str),
                        .join.case(pascal).ty(ident),
                        ,
                    }
            },
            {
                "A B C" Abc A B C
            }
        );
    }
}
