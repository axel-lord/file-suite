//! [clear] impl.

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::group_help::EmptyDelimited,
};

function_struct!(
    /// Consume array outputing nothing.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    clear {
        /// Optional macro delimiter.
        [optional] delim: Option<EmptyDelimited>,
    }
);

impl ToCallable for clear {
    type Call = ClearCallable;

    fn to_callable(&self) -> Self::Call {
        ClearCallable
    }
}

/// [Call] implementor for [clear]
#[derive(Debug, Clone, Copy)]
pub struct ClearCallable;

impl Call for ClearCallable {
    fn call(&self, _: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        Ok(ValueArray::new())
    }
}
