//! [TakeCallable] impl.

use ::file_suite_proc_lib::{FromArg, spanned_int::SpannedInt};

use crate::{function::Call, storage::Storage, value_array::ValueArray};

/// Take n values from value array and discard the remainder. If negative
/// the last n values are taken.
#[derive(Debug, Clone, Copy)]
pub struct TakeCallable {
    /// Amount of values to take.
    n: isize,
}

impl FromArg for TakeCallable {
    type Factory = SpannedInt<isize>;

    fn from_arg(n: isize) -> Self {
        Self { n }
    }
}

impl Call for TakeCallable {
    fn call(&self, mut array: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let n = self.n.unsigned_abs();
        if n == 0 {
            array = ValueArray::new();
        } else if n >= array.len() {
            // do nothing
        } else if self.n > 0 {
            array.make_vec().drain(n..);
        } else {
            let len = array.len() - n;
            array.make_vec().drain(..len);
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
    pub fn take() {
        assert_arr_expr!(
            { A B C D E -> .take(1) },
            { A },
        );

        assert_arr_expr!(
            { A B C D E -> .take(-1) },
            { E },
        );

        assert_arr_expr!(
            { A B C D E -> .take(2) },
            { A B },
        );

        assert_arr_expr!(
            { A B C D E -> .take(-2) },
            { D E },
        );

        assert_arr_expr!(
            { A B C D E -> .take(0) },
            {  },
        );

        assert_arr_expr!(
            { A B C D E -> .take(12) },
            { A B C D E },
        );
    }
}
