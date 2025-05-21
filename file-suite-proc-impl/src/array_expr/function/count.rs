//! [Count] impl.

use ::quote::ToTokens;

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        value_array::ValueArray,
    },
    util::{group_help::EmptyGroup, lookahead_parse::LookaheadParse},
    value::{TyKind, Value},
};

#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(count);
}

/// Count amount of values passed.
#[derive(Debug, Clone)]
pub struct Count {
    /// Count keyword.
    kw: kw::count,
    /// Optional macro delimiter.
    delim: Option<EmptyGroup>,
}

impl ToCallable for Count {
    type Call = CountCallable;

    fn to_callable(&self) -> Self::Call {
        CountCallable
    }
}

/// [Call] implementor for [Count].
#[derive(Debug, Clone, Copy)]
pub struct CountCallable;

impl Call for CountCallable {
    fn call(
        &self,
        input: crate::array_expr::value_array::ValueArray,
    ) -> syn::Result<crate::array_expr::value_array::ValueArray> {
        let mut value = Value::new_int(
            input
                .len()
                .try_into()
                .unwrap_or_else(|_| unreachable!("all lengths should fit in an isize")),
        );
        if let Some(span) = input.span() {
            value.set_span(span);
        }
        value.set_ty(TyKind::int);
        Ok(ValueArray::from_value(value))
    }
}

impl LookaheadParse for Count {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        Ok(if lookahead.peek(kw::count) {
            Some(Self {
                kw: input.parse()?,
                delim: input.call(EmptyGroup::optional_parse)?,
            })
        } else {
            None
        })
    }
}

impl ToTokens for Count {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim } = self;
        kw.to_tokens(tokens);
        delim.to_tokens(tokens);
    }
}
