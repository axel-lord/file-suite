//! [RepeatArgs] impl

use ::std::num::NonZero;

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

/// Arguments for repeat.
#[derive(Debug, Clone, Copy)]
pub struct RepeatArgs {
    /// How many times to repeat array.
    times: SpannedInt<NonZero<usize>>,
}

impl Parse for RepeatArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            times: input.call(SpannedInt::parse)?,
        })
    }
}

impl ToTokens for RepeatArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { times } = self;
        times.to_tokens(tokens);
    }
}

impl ToCallable for RepeatArgs {
    type Call = RepeatCallable;

    fn to_callable(&self) -> Self::Call {
        RepeatCallable {
            times: self.times.value,
        }
    }
}

/// [Call] implementor for [RepeatArgs].
#[derive(Debug, Clone)]
pub struct RepeatCallable {
    /// Times to repeat.
    times: NonZero<usize>,
}

impl Call for RepeatCallable {
    fn call(&self, array: ValueArray, _storage: &mut Storage) -> crate::Result<ValueArray> {
        Ok(::std::iter::repeat_n((), self.times.get())
            .flat_map(|_| array.iter().cloned())
            .collect())
    }
}
