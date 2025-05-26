//! [shift] impl.

use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{lookahead_parse::LookaheadParse, spanned_int::SpannedInt},
};

/// Arguments for shifting array.
#[derive(Debug, Clone)]
pub struct ShiftArgs {
    /// How much to shift array, and in what direction.
    amount: SpannedInt<isize>,
}

impl Parse for ShiftArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            amount: input.call(SpannedInt::parse)?,
        })
    }
}

impl ToTokens for ShiftArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { amount } = self;
        amount.to_tokens(tokens);
    }
}

impl ToCallable for ShiftArgs {
    type Call = ShiftCallable;

    fn to_callable(&self) -> Self::Call {
        ShiftCallable {
            by: self.amount.value,
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
    fn call(&self, mut array: ValueArray, _storage: &mut Storage) -> crate::Result<ValueArray> {
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
