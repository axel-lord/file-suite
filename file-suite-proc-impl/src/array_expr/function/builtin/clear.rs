//! [ClearCallable] impl.

use crate::array_expr::{function::Call, storage::Storage, value_array::ValueArray};

/// [Call] implementor to clear array.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClearCallable;

impl Call for ClearCallable {
    fn call(&self, _: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        Ok(ValueArray::new())
    }
}
