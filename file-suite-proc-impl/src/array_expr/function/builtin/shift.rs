//! [shift] impl.

use ::std::borrow::Cow;

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{group_help::GroupOption, lookahead_parse::ParseWrap, spanned_int::SpannedInt},
};

function_struct!(
    /// Shift alements by given amount (defaults to 1).
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    shift {
        /// Amount to shift the elements by.
        [optional] shift: Option<GroupOption<ParseWrap<SpannedInt<isize>>>>,
    }
);

impl ToCallable for shift {
    type Call = ShiftCallable;

    fn to_callable(&self) -> Self::Call {
        ShiftCallable {
            by: self
                .shift
                .as_ref()
                .and_then(|by| Some(by.unwrap_parsed()?.value))
                .unwrap_or(1),
        }
    }
}

/// [Call] impl for [shift].
#[derive(Debug, Clone, Copy)]
pub struct ShiftCallable {
    /// What to shift by.
    by: isize,
}

impl Default for ShiftCallable {
    fn default() -> Self {
        Self { by: 1 }
    }
}

impl Call for ShiftCallable {
    fn call(
        &self,
        mut array: ValueArray,
        _storage: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
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
