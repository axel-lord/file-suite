//! Produce random hexadecimal numbers with the same width as value array. (rounded down to the
//! closest multiple of 2)

use ::std::io::{Cursor, Write};

use ::rand::Rng;

use crate::{
    function::{Call, DefaultArgs},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor to generate random values.
#[derive(Debug, Clone, Copy)]
pub struct RandCallable;

impl DefaultArgs for RandCallable {
    fn default_args() -> Self {
        Self
    }
}

impl Call for RandCallable {
    fn call(&self, mut array: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let mut rng = ::rand::rng();
        let mut buf = [0u8; 32];
        for value in &mut array {
            let string = value.make_string();
            let mut bytes = Vec::<u8>::from(::std::mem::take(string));
            let mut len = bytes.len();
            bytes.truncate(len);

            while len > 0 {
                write!(
                    Cursor::new(buf.as_mut_slice()),
                    "{rand:0>32X}",
                    rand = rng.random::<u128>()
                )
                .unwrap_or_else(|_| unreachable!());
                let count = len.min(32);
                bytes[(len - count)..].copy_from_slice(&buf[..count]);
                len -= count;
            }

            *string = bytes.try_into().unwrap_or_else(|_| unreachable!());
        }
        Ok(array)
    }
}
#[cfg(test)]
mod tests {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use ::proc_macro2::TokenTree;
    use ::quote::quote;
    use ::syn::Lit;

    use crate::array_expr;

    #[test]
    fn random_lengths() {
        let expr = quote! { A AB ABC 1234 "hello" -> rand.ty(str) };
        let result = array_expr(expr).unwrap();

        for (token, i) in result.into_iter().zip(1..) {
            let TokenTree::Literal(lit) = token else {
                panic!("token {token:?} is not a literal");
            };
            let Lit::Str(lit_str) = Lit::new(lit.clone()) else {
                panic!("literal {lit} is not a string literal");
            };
            let value = lit_str.value();
            u64::from_str_radix(&value, 16)
                .unwrap_or_else(|err| panic!("could not convert '{value}' to decimal, {err}"));
            if value.len() != i {
                panic!("length of {lit_str:?} should be {i}",);
            }
        }
    }
}
