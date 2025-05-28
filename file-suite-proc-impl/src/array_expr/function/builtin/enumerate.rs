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
    util::{lookahead_parse::optional_lookahead_parse, spanned_int::SpannedInt},
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

/// Step of specification.
#[derive(Debug, Clone)]
pub struct Step {
    /// ':' token.
    pub colon: Token![:],
    /// Step value.
    pub step: Option<SpannedInt<isize>>,
}

/// Array span of specification.
#[derive(Debug, Clone)]
pub struct ArraySpan {
    /// ':' token.
    pub colon: Token![:],
    /// Span value.
    pub array_span: Option<SpannedInt<NonZero<usize>>>,
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

        let offset = optional_lookahead_parse(input, &mut lookahead)?;

        let step = if let Some(colon) = optional_lookahead_parse(input, &mut lookahead)? {
            let step = optional_lookahead_parse(input, &mut lookahead)?;
            Some(Step { colon, step })
        } else if lookahead.peek(End) {
            return Ok(Self {
                offset,
                ..Default::default()
            });
        } else {
            return Err(lookahead.error());
        };

        let array_span = if let Some(colon) = optional_lookahead_parse(input, &mut lookahead)? {
            let array_span = optional_lookahead_parse(input, &mut lookahead)?;
            Some(ArraySpan { colon, array_span })
        } else if lookahead.peek(End) {
            return Ok(Self {
                offset,
                step,
                ..Default::default()
            });
        } else {
            return Err(lookahead.error());
        };

        if !lookahead.peek(End) {
            return Err(lookahead.error());
        }

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

impl ToTokens for Step {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { colon, step } = self;
        colon.to_tokens(tokens);
        step.to_tokens(tokens);
    }
}

impl ToTokens for ArraySpan {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { colon, array_span } = self;
        colon.to_tokens(tokens);
        array_span.to_tokens(tokens);
    }
}
