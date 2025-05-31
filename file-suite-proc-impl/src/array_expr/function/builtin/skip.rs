//! [SkipCallable] impl.

use ::file_suite_proc_lib::FromArg;

use crate::{
    array_expr::{function::Call, storage::Storage, value_array::ValueArray},
    util::spanned_int::SpannedInt,
};
/// Skip n values from array. If negative the last n values are skipped.
#[derive(Debug, Clone, Copy)]
pub struct SkipCallable {
    /// Amount of values to skip.
    n: isize,
}

impl FromArg for SkipCallable {
    type ArgFactory = SpannedInt<isize>;

    fn from_arg(n: isize) -> Self {
        Self { n }
    }
}

impl Call for SkipCallable {
    fn call(&self, mut array: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let n = self.n.unsigned_abs();
        if n == 0 {
            // do nothing
        } else if n >= array.len() {
            array = ValueArray::new();
        } else if self.n > 0 {
            array.make_vec().drain(..n);
        } else {
            let len = array.len() - n;
            array.make_vec().drain(len..);
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

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    pub fn skip() {
        assert_arr_expr!(
            { A B C D E -> .skip(1) },
            { B C D E },
        );

        assert_arr_expr!(
            { A B C D E -> .skip(-1) },
            { A B C D },
        );

        assert_arr_expr!(
            { A B C D E -> .skip(2) },
            { C D E },
        );

        assert_arr_expr!(
            { A B C D E -> .skip(-2) },
            { A B C },
        );

        assert_arr_expr!(
            { A B C D E -> .skip(0) },
            { A B C D E },
        );

        assert_arr_expr!(
            { A B C D E -> .skip(12) },
            { },
        );
    }
}
