//! [AliasArgs] impl.

use ::file_suite_proc_lib::{ArgTy, FromArg};

use crate::array_expr::{
    function::{Call, Callable, Function, FunctionChain},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor for [AliasArgs].
#[derive(Debug, Clone)]
pub struct AliasCallable {
    /// Function chain to store.
    chain: Vec<Callable<Function>>,
}

impl FromArg for AliasCallable {
    type Factory = FunctionChain;

    fn from_arg(chain: ArgTy<Self>) -> Self {
        Self { chain }
    }
}

impl Call for AliasCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        for value in array {
            storage.set_alias(value.into(), self.chain.clone());
        }

        Ok(ValueArray::new())
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
    fn define_and_use() {
        assert_arr_expr!(
            {
                toKebab -> alias{ case(lower).join(kebab) },
                fromCamel -> alias { split(camel) },
                camelToKebabConv -> =fromCamel.=toKebab.ty(str),
            },
            { "camel-to-kebab-conv" },
        );
    }
}
