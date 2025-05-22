//! [enumerate] impl.

use std::borrow::Cow;

use ::proc_macro2::{Literal, Span};
use ::quote::{ToTokens, TokenStreamExt};
use ::syn::{LitInt, parse::Parse};

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        value::Value,
        value_array::ValueArray,
    },
    util::group_help::GroupOption,
};

function_struct!(
    /// Enumerate array.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    enumerate {
        /// Delim for spec,
        [optional] offset: Option<GroupOption<Offset>>,
    }
);

impl ToCallable for enumerate {
    type Call = EnumerateCallable;

    fn to_callable(&self) -> Self::Call {
        EnumerateCallable {
            offset: self
                .offset
                .as_ref()
                .and_then(|offset| offset.content.as_ref())
                .map(|content| content.value)
                .unwrap_or(1),
        }
    }
}

/// [Call] Implementor fo [Enumerate].
#[derive(Debug, Clone, Copy)]
pub struct EnumerateCallable {
    /// Offset of enumeration.
    offset: isize,
}

impl Call for EnumerateCallable {
    fn call(&self, input: ValueArray) -> Result<ValueArray, Cow<'static, str>> {
        let mut offset = self.offset;

        let mut output = Vec::with_capacity(input.len().checked_mul(2).ok_or_else(|| {
            Cow::Owned(format!(
                "value array length should be multipliable by 2, is {}",
                input.len()
            ))
        })?);
        for value in input {
            let next_offset = offset.checked_add(1).ok_or_else(|| {
                Cow::Owned(format!("offset should not exceed isize::MAX, is {offset}"))
            })?;

            output.push(Value::new_int(offset).with_span_of(&value));
            output.push(value);
            offset = next_offset;
        }

        Ok(ValueArray::from_vec(output))
    }
}

/// Offset value for enumeration.
#[derive(Debug, Clone, Copy)]
pub struct Offset {
    /// Value of offset.
    pub value: isize,
    /// Span of offset.
    pub span: Span,
}

impl Parse for Offset {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_int = input.parse::<LitInt>()?;
        Ok(Self {
            value: lit_int.base10_parse()?,
            span: lit_int.span(),
        })
    }
}

impl ToTokens for Offset {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { value, span } = *self;
        let mut val = Literal::isize_unsuffixed(value);
        val.set_span(span);
        tokens.append(val);
    }
}
