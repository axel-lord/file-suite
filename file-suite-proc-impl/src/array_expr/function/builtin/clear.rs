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
