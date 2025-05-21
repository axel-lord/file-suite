//! [Enumerate] impl.

use ::proc_macro2::Span;
use ::quote::{ToTokens, quote_spanned};
use ::syn::{LitInt, MacroDelimiter, parse::End};

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        value_array::ValueArray,
    },
    util::{MacroDelimExt, ensure_empty, lookahead_parse::LookaheadParse, macro_delimited},
    value::Value,
};

#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(enumerate);
}

/// Enumerate array.
#[derive(Debug, Clone)]
pub struct Enumerate {
    /// Enumerate keyword
    kw: kw::enumerate,
    /// Delim for spec,
    delim: Option<MacroDelimiter>,
    /// Offset for enumeration.
    offset: Option<(isize, Span)>,
}

impl ToCallable for Enumerate {
    type Call = EnumerateCallable;

    fn to_callable(&self) -> Self::Call {
        EnumerateCallable {
            offset: self.offset.as_ref().map(|(i, _)| *i).unwrap_or(1),
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
    fn call(&self, input: ValueArray) -> syn::Result<ValueArray> {
        let mut offset = self.offset;

        let mut output = Vec::with_capacity(input.len().checked_mul(2).ok_or_else(|| {
            ::syn::Error::new(
                input.span().unwrap_or_else(Span::call_site),
                format!(
                    "value array length should be multipliable by 2, is {}",
                    input.len()
                ),
            )
        })?);
        for value in input {
            let next_offset = offset.checked_add(1).ok_or_else(|| {
                ::syn::Error::new(
                    value.span().unwrap_or_else(Span::call_site),
                    format!("offset should not exceed isize::MAX, is {offset}"),
                )
            })?;

            output.push(Value::new_int(offset).with_span_of(&value));
            output.push(value);
            offset = next_offset;
        }

        Ok(ValueArray::from_vec(output))
    }
}

impl LookaheadParse for Enumerate {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(kw::enumerate)
            .then(|| {
                let kw = input.parse()?;

                let mut delim = None;
                let mut offset = None;

                if MacroDelimiter::input_peek(input) {
                    let content;
                    delim = Some(macro_delimited!(content in input));

                    let lookahead = content.lookahead1();
                    if !lookahead.peek(End) {
                        if let Some(lit_int) = LitInt::lookahead_parse(input, &lookahead)? {
                            offset = Some((lit_int.base10_parse()?, lit_int.span()));
                        }

                        if offset.is_none() {
                            return Err(lookahead.error());
                        }

                        ensure_empty(&content)?;
                    }
                }

                Ok(Self { kw, delim, offset })
            })
            .transpose()
    }
}

impl ToTokens for Enumerate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim, offset } = self;
        kw.to_tokens(tokens);
        if let Some(delim) = delim {
            delim.surround(tokens, |tokens| {
                if let Some((i, span)) = offset {
                    tokens.extend(quote_spanned! {*span=> #i});
                }
            });
        }
    }
}
