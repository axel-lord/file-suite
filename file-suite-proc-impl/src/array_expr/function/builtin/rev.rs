//! [RevCallable] impl.

use crate::array_expr::{
    function::{Call, DefaultArgs},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor to reverse array.
#[derive(Debug, Clone, Copy)]
pub struct RevCallable;

impl DefaultArgs for RevCallable {
    fn default_args() -> Self {
        Self
    }
}

impl Call for RevCallable {
    fn call(&self, input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let mut values = input;
        values.reverse();
        Ok(values)
    }
}
