//! [ClearCallable] impl.

use crate::array_expr::{
    function::{Call, DefaultArgs},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor to clear array.
#[derive(Debug, Clone, Copy)]
pub struct ClearCallable;

impl DefaultArgs for ClearCallable {
    fn default_args() -> Self {
        Self
    }
}

impl Call for ClearCallable {
    fn call(&self, _: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
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
    fn clear() {
        assert_arr_expr!(
            { A B C -> .clear },
            {},
        );

        assert_arr_expr!(
            { A B C -> .fork(.clear, .join) },
            { ABC },
        );
    }
}
