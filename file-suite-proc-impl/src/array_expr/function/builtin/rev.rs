//! [rev] impl.

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::group_help::EmptyDelimited,
};

function_struct!(
    /// Reverse array.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    rev {
        /// Optional delimiter.
        [optional] delim: Option<EmptyDelimited>,
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
    fn call(&self, input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let mut values = input;
        values.reverse();
        Ok(values)
    }
}
