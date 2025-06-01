//! [RepeatCallable] impl

use ::std::num::NonZero;

use ::file_suite_proc_lib::{FromArg, spanned_int::SpannedInt};

use crate::array_expr::{function::Call, storage::Storage, value_array::ValueArray};

/// [Call] implementor for [RepeatArgs].
#[derive(Debug, Clone)]
pub struct RepeatCallable {
    /// Times to repeat.
    times: NonZero<usize>,
}

impl FromArg for RepeatCallable {
    type Factory = SpannedInt<NonZero<usize>>;

    fn from_arg(times: NonZero<usize>) -> Self {
        Self { times }
    }
}

impl Call for RepeatCallable {
    fn call(&self, array: ValueArray, _storage: &mut Storage) -> crate::Result<ValueArray> {
        Ok(::std::iter::repeat_n((), self.times.get())
            .flat_map(|_| array.iter().cloned())
            .collect())
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
    fn repeat_value() {
        assert_arr_expr!(
            { ABC -> repeat(3).ty(ident) },
            { ABC ABC ABC },
        );

        assert_arr_expr!(
            {
                3 -> global(by),
                ABC -> repeat(=by).ty(ident),
            },
            { ABC ABC ABC },
        );
    }
}
