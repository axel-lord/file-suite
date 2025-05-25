//! [repeat] impl

use ::std::{borrow::Cow, num::NonZero};

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{group_help::GroupSingle, lookahead_parse::ParseWrap, spanned_int::SpannedInt},
};

function_struct!(
    /// Repeat a value, the input decides the amount of times.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    repeat {
        /// Amount of times to repeat array.
        times: GroupSingle<ParseWrap<SpannedInt<NonZero<usize>>>>,
    }
);

impl ToCallable for repeat {
    type Call = RepeatCallable;

    fn to_callable(&self) -> Self::Call {
        RepeatCallable {
            times: self.times.content.0.value,
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
    fn call(
        &self,
        array: ValueArray,
        _storage: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        Ok(::std::iter::repeat_n((), self.times.get())
            .flat_map(|_| array.iter().cloned())
            .collect())
    }
}
