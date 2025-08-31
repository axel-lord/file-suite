//! Function to get variables using array as key.

use crate::{
    function::{Call, DefaultArgs},
    storage::Storage,
    value_array::ValueArray,
};

/// Use array values as keys to get values.
#[derive(Debug, Clone, Copy)]
pub struct GetCallable;

impl DefaultArgs for GetCallable {
    fn default_args() -> Self {
        Self
    }
}

impl Call for GetCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        match array.as_ref() {
            [] => Ok(ValueArray::new()),
            [value] => Ok(storage.try_get(value)?.clone()),
            values => values
                .iter()
                .try_fold(ValueArray::new(), |mut array, value| {
                    array.extend(storage.try_get(value)?.iter().cloned());
                    Ok(array)
                }),
        }
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
    fn get() {
        assert_arr_expr!(
            {
                1 2 -> global(seq),
                A B C -> global(1),
                D E F -> global(2),
                =seq -> get
            },
            { A B C D E F},
        );
    }
}
