//! [count] impl.

use std::borrow::Cow;

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        value::{TyKind, Value},
        value_array::ValueArray,
    },
    util::group_help::EmptyGroup,
};

function_struct!(
    /// Count amount of values passed.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    count {
        /// Optional macro delimiter.
        [optional] delim: Option<EmptyGroup>,
    }
);

impl ToCallable for count {
    type Call = CountCallable;

    fn to_callable(&self) -> Self::Call {
        CountCallable
    }
}

/// [Call] implementor for [Count].
#[derive(Debug, Clone, Copy)]
pub struct CountCallable;

impl Call for CountCallable {
    fn call(
        &self,
        input: crate::array_expr::value_array::ValueArray,
    ) -> Result<ValueArray, Cow<'static, str>> {
        let mut value = Value::new_int(
            input
                .len()
                .try_into()
                .unwrap_or_else(|_| unreachable!("all lengths should fit in an isize")),
        );
        if let Some(span) = input.span() {
            value.set_span(span);
        }
        value.set_ty(TyKind::int);
        Ok(ValueArray::from_value(value))
    }
}
