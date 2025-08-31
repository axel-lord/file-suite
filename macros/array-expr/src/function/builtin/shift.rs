//! [ShiftCallable] impl.

use ::file_suite_proc_lib::{FromArg, spanned_int::SpannedInt};

use crate::{
    function::{Call, DefaultArgs},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] impl for [ShiftArgs].
#[derive(Debug, Clone, Copy)]
pub struct ShiftCallable {
    /// What to shift by.
    by: isize,
}

impl DefaultArgs for ShiftCallable {
    fn default_args() -> Self {
        Self { by: 1 }
    }
}

impl FromArg for ShiftCallable {
    type Factory = SpannedInt<isize>;

    fn from_arg(by: isize) -> Self {
        Self { by }
    }
}

impl Call for ShiftCallable {
    fn call(&self, mut array: ValueArray, _storage: &mut Storage) -> crate::Result<ValueArray> {
        if array.len() <= 1 || self.by == 0 {
            Ok(array)
        } else if self.by < 0 {
            let by = (-self.by) as usize % array.len();
            array.rotate_left(by);

            Ok(array)
        } else {
            let by = self.by as usize % array.len();
            array.rotate_right(by);

            Ok(array)
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
    fn shift_array() {
        assert_arr_expr!(
            { A B C D -> shift(1) },
            { D A B C },
        );
        assert_arr_expr!(
            { A B C D -> shift(-1) },
            { B C D A },
        );
        assert_arr_expr!(
            {
                -1 -> global(by),
                A B C D -> shift(=by),
            },
            { B C D A },
        );
    }
}
