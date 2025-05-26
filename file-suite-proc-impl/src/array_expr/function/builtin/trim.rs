//! [trim] impl

use crate::array_expr::{function::Call, storage::Storage, value_array::ValueArray};

/// Trim whitespace arround values in array.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrimCallable;

impl Call for TrimCallable {
    fn call(&self, mut array: ValueArray, _storage: &mut Storage) -> crate::Result<ValueArray> {
        for value in &mut array {
            // make_string may not be cheap. And the created string needs an addr.
            if value.is_empty() {
                continue;
            }

            // We make sure trim works on the same string as is drained and truncated.
            let string = value.make_string();

            let trimmed = string.trim();
            let len = trimmed.len();
            let start = trimmed.as_ptr().addr() - string.as_str().as_ptr().addr();

            string.drain(..start);
            string.truncate(len);
        }
        Ok(array)
    }
}
