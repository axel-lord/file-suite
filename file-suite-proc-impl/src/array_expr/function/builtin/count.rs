//! [CountCallable] impl.

use crate::array_expr::{
    function::{Call, DefaultArgs},
    storage::Storage,
    value::Value,
    value_array::ValueArray,
};

/// [Call] implementor to count array elements.
#[derive(Debug, Clone, Copy)]
pub struct CountCallable;

impl DefaultArgs for CountCallable {
    fn default_args() -> Self {
        Self
    }
}

impl Call for CountCallable {
    fn call(
        &self,
        input: crate::array_expr::value_array::ValueArray,
        _: &mut Storage,
    ) -> crate::Result<ValueArray> {
        let mut value = Value::new_int(input.len().try_into().unwrap_or_else(|_| unreachable!()));
        if let Some(span) = input.span() {
            value.set_span(span);
        }
        Ok(ValueArray::from_value(value))
    }
}
