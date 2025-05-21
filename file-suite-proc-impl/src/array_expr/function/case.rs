//! [Case] impl.

use ::quote::ToTokens;
use ::syn::MacroDelimiter;

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        value_array::ValueArray,
    },
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

impl ToCallable for Case {
    type Call = SpecKind;

    fn to_callable(&self) -> Self::Call {
        self.spec.kind
    }
}

impl Call for SpecKind {
    fn call(&self, mut input: ValueArray) -> syn::Result<ValueArray> {
        match self {
            SpecKind::pascal => {
                for value in &mut input {
                    let mut capitalized = capitalize(value);
                    capitalized.shrink_to_fit();
                    value.set_content(capitalized);
                }

                Ok(input)
            }
            SpecKind::camel => {
                let mut values = input.iter_mut();

                if let Some(first) = values.next() {
                    first.set_content(first.to_lowercase());
                }

                for value in values {
                    value.set_content(capitalize(value));
                }

                Ok(input)
            }
            SpecKind::upper => {
                for value in &mut input {
                    value.set_content(value.to_uppercase());
                }

                Ok(input)
            }
            SpecKind::lower => {
                for value in &mut input {
                    value.set_content(value.to_lowercase());
                }

                Ok(input)
            }
        }
    }
}

/// Get capitalized version of a string slice.
fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    let mut capitalized = String::with_capacity(value.len());

    if let Some(first) = chars.next() {
        capitalized.extend(first.to_uppercase());
    }

    for chr in chars {
        capitalized.extend(chr.to_lowercase());
    }

    capitalized
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
