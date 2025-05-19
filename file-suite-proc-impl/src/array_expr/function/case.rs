//! [Case] impl.

use ::quote::ToTokens;
use ::syn::MacroDelimiter;

use crate::{
    array_expr::{function::Call, value_array::ValueArray},
    util::{
        MacroDelimExt, ensure_empty, kw_kind, lookahead_parse::LookaheadParse, macro_delimited,
    },
};
#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(case);
}

kw_kind!(
    /// Casing to apply.
    Spec;
    /// Enum containing possible values for [Spec]
    #[expect(non_camel_case_types)]
    SpecKind {
        pascal,
        camel,
        upper,
        lower,
    }
);

/// Apply case to input.
#[derive(Debug, Clone)]
pub struct Case {
    /// Case keyword.
    kw: kw::case,
    /// Delim for spec.
    delim: MacroDelimiter,
    /// Specification for which case to apply.
    spec: Spec,
}

/// Get capitalized version of a string slice.
fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_uppercase())
        .into_iter()
        .flatten()
        .chain(chars.flat_map(char::to_lowercase))
        .collect()
}

impl Call for Case {
    fn call(&self, input: ValueArray) -> syn::Result<ValueArray> {
        let mut values = input;
        match self.spec.kind {
            SpecKind::pascal => {
                for value in &mut values {
                    value.remap_value(|v| capitalize(&v));
                }
            }

            SpecKind::camel => {
                let mut values = values.iter_mut();
                if let Some(first) = values.next() {
                    first.remap_value(|value| value.to_lowercase());
                }
                for value in values {
                    value.remap_value(|value| capitalize(&value));
                }
            }
            SpecKind::upper => {
                for value in &mut values {
                    value.remap_value(|value| value.to_uppercase());
                }
            }
            SpecKind::lower => {
                for value in &mut values {
                    value.remap_value(|value| value.to_lowercase());
                }
            }
        }

        Ok(values)
    }
}

impl LookaheadParse for Case {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(kw::case)
            .then(|| {
                let content;
                let value = Self {
                    kw: input.parse()?,
                    delim: macro_delimited!(content in input),
                    spec: content.call(Spec::parse)?,
                };

                ensure_empty(&content)?;

                Ok(value)
            })
            .transpose()
    }
}

impl ToTokens for Case {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim, spec } = self;
        kw.to_tokens(tokens);
        delim.surround(tokens, |tokens| spec.to_tokens(tokens));
    }
}
