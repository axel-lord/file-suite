//! [rev] impl.

use std::borrow::Cow;

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::group_help::EmptyGroup,
};

function_struct!(
    /// Reverse array.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    rev {
        /// Optional delimiter.
        [optional] delim: Option<EmptyGroup>,
    }
);

impl ToCallable for rev {
    type Call = RevCallable;

    fn to_callable(&self) -> Self::Call {
        RevCallable
    }
}

/// [Call] implementor for [rev].
#[derive(Debug, Clone, Copy)]
pub struct RevCallable;

impl Call for RevCallable {
    fn call(&self, input: ValueArray, _: &mut Storage) -> Result<ValueArray, Cow<'static, str>> {
        let mut values = input;
        values.reverse();
        Ok(values)
    }
}
