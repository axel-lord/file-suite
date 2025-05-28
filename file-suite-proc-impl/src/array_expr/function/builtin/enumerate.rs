//! [EnumerateArgs] impl.

use ::std::num::NonZero;

use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Parse},
};

use crate::{
    array_expr::{
        function::{Call, DefaultArgs, ToCallable},
        storage::Storage,
        value::Value,
        value_array::ValueArray,
    },
    util::{
        ensure_empty,
        lookahead_parse::{LookaheadParse, lookahead_parse},
        spanned_int::SpannedInt,
    },
};

/// [Call] Implementor for [EnumerateArgs].
#[derive(Debug, Clone, Copy)]
pub struct EnumerateCallable {
    /// Offset of enumeration.
    offset: isize,
    /// Step of enumeration.
    step: isize,
    /// Span of enumeration.
    array_span: NonZero<usize>,
}

impl DefaultArgs for EnumerateCallable {
    fn default_args() -> Self {
        Self {
            offset: 1,
            step: 1,
            array_span: const { NonZero::new(1).unwrap() },
        }
    }
}

impl Call for EnumerateCallable {
    fn call(&self, input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        let mut offset = self.offset;
        let step = self.step;
        let span = self.array_span.get();

        let mut output = Vec::with_capacity(input.len());
        let mut input = input.into_iter();

        while let Some(first) = input.next() {
            output.push(Value::new_int(offset).with_span_of(&first));
            output.push(first);
            for _ in 1..span {
                output.extend(input.next());
            }
            offset = offset
                .checked_add(step)
                .ok_or_else(|| format!("integer over/underflow adding {step} to {offset}"))?;
        }

        Ok(ValueArray::from_vec(output))
    }
}

/// Enumeration specification.
#[derive(Debug, Clone, Default)]
pub struct EnumerateArgs {
    /// First value of enumeration.
    pub offset: Option<SpannedInt<isize>>,

    /// What to change enumeration value by for each step.
    pub step: Option<Step>,

    /// How many array elements to include between enumeration values.
    pub array_span: Option<ArraySpan>,
}

impl ToCallable for EnumerateArgs {
    type Call = EnumerateCallable;

    fn to_callable(&self) -> Self::Call {
        let default = EnumerateCallable::default_args();

        let offset = self
            .offset
            .as_ref()
            .map(|offset| offset.value)
            .unwrap_or(default.offset);

        let step = self
            .step
            .as_ref()
            .and_then(|step| step.step.as_ref())
            .map(|step| step.value)
            .unwrap_or(default.step);

        let array_span = self
            .array_span
            .as_ref()
            .and_then(|array_span| array_span.array_span.as_ref())
            .map(|array_span| array_span.value)
            .unwrap_or(default.array_span);

        EnumerateCallable {
            offset,
            step,
            array_span,
        }
    }
}

impl Parse for EnumerateArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut lookahead = input.lookahead1();
        let offset = if let Some(offset) = lookahead_parse(input, &lookahead)? {
            lookahead = input.lookahead1();
            Some(offset)
        } else {
            None
        };

        let step = if let Some(step) = lookahead_parse(input, &lookahead)? {
            Some(step)
        } else if !lookahead.peek(End) {
            return Err(lookahead.error());
        } else {
            None
        };

        let lookahead = input.lookahead1();
        let array_span = if let Some(array_span) = lookahead_parse(input, &lookahead)? {
            Some(array_span)
        } else if !lookahead.peek(End) {
            return Err(lookahead.error());
        } else {
            None
        };

        ensure_empty(input)?;

        Ok(Self {
            offset,
            step,
            array_span,
        })
    }
}

impl ToTokens for EnumerateArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            offset,
            step,
            array_span,
        } = self;
        offset.to_tokens(tokens);
        step.to_tokens(tokens);
        array_span.to_tokens(tokens);
    }
}

/// Step of specification.
#[derive(Debug, Clone)]
pub struct Step {
    /// ':' token.
    pub colon: Token![:],
    /// Step value.
    pub step: Option<SpannedInt<isize>>,
}

impl LookaheadParse for Step {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(Token![:])
            .then(|| {
                let colon = input.parse()?;
                let lookahead = input.lookahead1();
                let step = if let Some(step) = lookahead_parse(input, &lookahead)? {
                    Some(step)
                } else if lookahead.peek(Token![:]) || lookahead.peek(End) {
                    None
                } else {
                    return Err(lookahead.error());
                };

                Ok(Self { colon, step })
            })
            .transpose()
    }
}

impl ToTokens for Step {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { colon, step } = self;
        colon.to_tokens(tokens);
        step.to_tokens(tokens);
    }
}

/// Array span of specification.
#[derive(Debug, Clone)]
pub struct ArraySpan {
    /// ':' token.
    pub colon: Token![:],
    /// Span value.
    pub array_span: Option<SpannedInt<NonZero<usize>>>,
}

impl ToTokens for ArraySpan {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { colon, array_span } = self;
        colon.to_tokens(tokens);
        array_span.to_tokens(tokens);
    }
}
impl LookaheadParse for ArraySpan {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(Token![:])
            .then(|| {
                let colon = input.parse()?;
                let lookahead = input.lookahead1();
                let array_span = if let Some(step) = lookahead_parse(input, &lookahead)? {
                    Some(step)
                } else if lookahead.peek(End) {
                    None
                } else {
                    return Err(lookahead.error());
                };

                Ok(Self { colon, array_span })
            })
            .transpose()
    }
}
