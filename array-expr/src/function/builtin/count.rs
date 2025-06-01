//! [CountCallable] impl.

use crate::{
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
    fn call(&self, input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let mut value = Value::new_int(input.len().try_into().unwrap_or_else(|_| unreachable!()));
        if let Some(span) = input.span() {
            value.set_span(span);
        }
        Ok(ValueArray::from_value(value))
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::test::assert_arr_expr;

    #[test]
    fn count() {
        assert_arr_expr!(
            { A B C -> .count },
            { 3 },
        );

        assert_arr_expr!(
            { A_B_C_D -> .split(snake).count },
            { 4 },
        );

        assert_arr_expr!(
            {
                "Count these words, please?" ->
                    .fork(
                        ,
                        .split(space)
                        .count
                        .local(wordcount)
                        .block{
                            There are =wordcount words -> join(space).ty(str)
                        }
                    )
            },
            {
                "Count these words, please?"
                "There are 4 words"
            },
        );
    }
}
