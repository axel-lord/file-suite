//! [Ty] impl.

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value::{Ty, TyKind},
        value_array::ValueArray,
    },
    util::{group_help::Delimited, parse_wrap::ParseWrap},
};

function_struct!(
    /// Convert type of array.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    ty {
        /// Specification for which type to apply.
        ty: Delimited<ParseWrap<Ty>>,
    }
);

impl ToCallable for ty {
    type Call = TyKind;

    fn to_callable(&self) -> Self::Call {
        self.ty.inner.inner.kind
    }
}

impl Call for TyKind {
    fn call(&self, mut input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        for value in &mut input {
            value.ty = *self;
        }
        Ok(input)
    }
}
