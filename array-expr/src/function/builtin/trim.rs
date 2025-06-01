//! [TrimCallable] impl

use crate::{
    function::{Call, DefaultArgs},
    storage::Storage,
    value_array::ValueArray,
};

/// Trim whitespace arround values in array.
#[derive(Debug, Clone, Copy)]
pub struct TrimCallable;

impl DefaultArgs for TrimCallable {
    fn default_args() -> Self {
        Self
    }
}

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

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::test::assert_arr_expr;

    #[test]
    fn trim() {
        assert_arr_expr!(
            { "  hello     " -> trim.ty(ident) },
            { hello },
        );

        assert_arr_expr!(
            { "use, when, splitting" -> .split_by(",").trim.ty(ident) },
            { use when splitting },
        );
    }
}
