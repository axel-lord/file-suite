//! Function to get the nth value of an array, if negative from the back.
use ::std::num::NonZero;

use ::file_suite_proc_lib::{ArgTy, FromArg};

use crate::{
    array_expr::{function::Call, storage::Storage, value_array::ValueArray},
    util::spanned_int::SpannedInt,
};

/// Get the nth valu of an array, may not be 0, and negative values
/// results in lookup from the back. Will error on failure to get value.
#[derive(Debug, Clone, Copy)]
pub struct NthCallable {
    /// Position of value to get.
    n: NonZero<isize>,
}

impl FromArg for NthCallable {
    type Factory = SpannedInt<NonZero<isize>>;

    fn from_arg(n: ArgTy<Self>) -> Self {
        Self { n }
    }
}

impl Call for NthCallable {
    fn call(&self, array: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let n = if self.n.get() < 0 {
            array.len() - self.n.get().unsigned_abs()
        } else {
            self.n.get().unsigned_abs() - 1
        };

        if n > array.len() {
            return Err(crate::Error::from(format!(
                "cannot get value {} of array with length {}",
                self.n.get(),
                array.len()
            )));
        }

        Ok(ValueArray::from_value(array[n].clone()))
    }
}
#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    fn nth_value() {
        assert_arr_expr!(
            { A B C D -> nth(2) },
            { B }
        );
        assert_arr_expr!(
            { A B C D -> nth(-2) },
            { C }
        );
        assert_arr_expr!(
            {
                A B C D ->
                    .fork{, .count.global(n) }
                    .nth(=n)
            },
            { D }
        );
    }
}
