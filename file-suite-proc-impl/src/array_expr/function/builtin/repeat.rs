//! [repeat] impl

use ::std::num::NonZero;

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{group_help::Delimited, parse_wrap::ParseWrap, spanned_int::SpannedInt},
};

function_struct!(
    /// Repeat a value, the input decides the amount of times.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    repeat {
        /// Amount of times to repeat array.
        times: Delimited<ParseWrap<SpannedInt<NonZero<usize>>>>,
    }
);

impl ToCallable for repeat {
    type Call = RepeatCallable;

    fn to_callable(&self) -> Self::Call {
        RepeatCallable {
            times: self.times.inner.inner.value,
        }
    }
}

/// [Call] implementor for [repeat].
#[derive(Debug, Clone)]
pub struct RepeatCallable {
    /// Times to repeat.
    times: NonZero<usize>,
}

impl Call for RepeatCallable {
    fn call(&self, array: ValueArray, _storage: &mut Storage) -> crate::Result<ValueArray> {
        Ok(::std::iter::repeat_n((), self.times.get())
            .flat_map(|_| array.iter().cloned())
            .collect())
    }
}
