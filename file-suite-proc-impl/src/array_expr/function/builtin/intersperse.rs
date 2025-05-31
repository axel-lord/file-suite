//! [IntersperseCallable] impl.

use ::syn::Token;

use crate::array_expr::{
    function::{ArgTy, Call, FromArg},
    storage::Storage,
    typed_value::TypedValues,
    value_array::ValueArray,
};

/// Intersperse array elements with input.
#[derive(Debug, Clone)]
pub struct IntersperseCallable {
    /// Valueas to intersperse array with.
    values: ValueArray,
}

impl FromArg for IntersperseCallable {
    type ArgFactory = TypedValues<Token![,]>;

    fn from_arg(values: ArgTy<Self>) -> Self {
        Self { values }
    }
}

impl Call for IntersperseCallable {
    fn call(&self, array: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        if array.len() <= 1 {
            return Ok(array);
        }
        let capacity = array.len().saturating_sub(1) * self.values.len() + array.len();
        let mut out = Vec::with_capacity(capacity);

        let mut values = array.into_iter();
        out.extend(values.next());

        for value in values {
            out.extend_from_slice(&self.values);
            out.push(value);
        }

        Ok(ValueArray::from_vec(out))
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
    fn intersperse() {
        assert_arr_expr!(
            {
                "," -> .ty(tokens).global(sep),
                A B C -> intersperse(=sep),
            },
            { A, B, C },
        );
    }
}
