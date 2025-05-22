//! [Ty] impl.

use std::borrow::Cow;

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value::{Ty, TyKind},
        value_array::ValueArray,
    },
    util::{group_help::GroupSingle, lookahead_parse::ParseWrap},
};

function_struct!(
    /// Convert type of array.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    ty {
        /// Specification for which type to apply.
        ty: GroupSingle<ParseWrap<Ty>>,
    }
);

impl ToCallable for ty {
    type Call = TyKind;

    fn to_callable(&self) -> Self::Call {
        self.ty.content.0.kind
    }
}

impl Call for TyKind {
    fn call(
        &self,
        mut input: ValueArray,
        _: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        for value in &mut input {
            value.set_ty(*self);
        }
        Ok(input)
    }
}
