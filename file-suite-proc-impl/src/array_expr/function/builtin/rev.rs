//! [rev] impl.

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
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

/// [Call] implementor for [Rev].
#[derive(Debug, Clone, Copy)]
pub struct RevCallable;

impl Call for RevCallable {
    fn call(&self, input: ValueArray) -> syn::Result<ValueArray> {
        let mut values = input;
        values.reverse();
        Ok(values)
    }
}
