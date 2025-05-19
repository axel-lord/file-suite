//! [Enumerate] impl.

use ::quote::ToTokens;
use ::syn::{LitInt, MacroDelimiter, parse::End};

use crate::{
    array_expr::{function::Call, value_array::ValueArray},
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
    offset: Option<LitInt>,
}

impl Call for Enumerate {
    fn call(&self, input: ValueArray) -> syn::Result<ValueArray> {
        let mut offset = self
            .offset
            .as_ref()
            .map(|off| off.base10_parse::<isize>())
            .transpose()?
            .unwrap_or(1);

        let mut output = Vec::with_capacity(input.len().checked_mul(2).unwrap());

        for value in input {
            output.push(Value::new_int(offset).with_span_of(&value));
            output.push(value);
            offset = offset.checked_add(1).unwrap();
        }

        Ok(output.into())
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
                        offset = LitInt::lookahead_parse(input, &lookahead)?;

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
            delim.surround(tokens, |tokens| offset.to_tokens(tokens));
        }
    }
}
