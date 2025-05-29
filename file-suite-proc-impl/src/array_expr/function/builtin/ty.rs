//! [TyArgs] impl.

use crate::array_expr::{function::Call, storage::Storage, value_array::ValueArray};

#[doc(inline)]
pub use crate::array_expr::value::Ty;
#[doc(inline)]
pub use crate::array_expr::value::TyKind;

impl Call for TyKind {
    fn call(&self, mut input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        for value in &mut input {
            value.ty = *self;
        }
        Ok(input)
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    fn round_trip() {
        assert_arr_expr!(
            { (! enum Item { Roundtrip } ) -> ty(tokens) },
            { enum Item { Roundtrip } },
        );

        assert_arr_expr!(
            {
                str -> global(ty),
                Ident -> ty(=ty),
            },
            { "Ident" },
        );
    }
}
