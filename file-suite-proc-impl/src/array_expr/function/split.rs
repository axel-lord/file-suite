//! [Split] impl.
use ::quote::ToTokens;
use ::syn::{LitChar, LitStr, MacroDelimiter};

use crate::{
    array_expr::{
        function::{Call, spec_impl},
        value_array::ValueArray,
    },
    util::{
        MacroDelimExt, ensure_empty, kw_kind, lookahead_parse::LookaheadParse, macro_delimited,
    },
    value::Value,
};

#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(split);
}

kw_kind!(
    /// Keyword specified split
    SpecKw;
    /// Enum containing possible values for [SpecKw].
    #[expect(non_camel_case_types)]
    SpecKwKind {
        pascal,
        camel,
        kebab,
        snake,
        path,
        space,
        dot,
    }
);

spec_impl!(
    /// Specification for how to split a value.
    #[derive(Debug, Clone)]
    Spec {
        /// Split is specified by a string literal.
        Str(LitStr),
        /// Split is specified by a char literal.
        Char(LitChar),
        /// Split is specified by a keyword.
        Kw(SpecKw),
    }
);

/// Split input further.
#[derive(Debug, Clone)]
pub struct Split {
    /// Split keyword
    kw: kw::split,
    /// Delim for spec.
    delim: MacroDelimiter,
    /// Specification for to split value
    spec: Spec,
}

/// Split values by a [char].
#[inline]
fn split_by_char(pat: char, values: ValueArray) -> ValueArray {
    let mut buf = [0u8; 4];
    let pat = pat.encode_utf8(&mut buf);
    values.split_by_str(pat)
}

/// Split values by a [str].
#[inline]
fn split_by_str(pat: &str, values: ValueArray) -> ValueArray {
    values.split_by_str(pat)
}

impl Call for Split {
    fn call(&self, values: ValueArray) -> syn::Result<ValueArray> {
        Ok(match &self.spec {
            Spec::Str(lit_str) => split_by_str(&lit_str.value(), values),
            Spec::Char(lit_char) => split_by_char(lit_char.value(), values),
            Spec::Kw(spec_kw) => match spec_kw.kind {
                /*
                SpecKwKind::pascal => Value::split(&values, |value| {
                    value
                        .split(char::is_uppercase)
                        .skip(if value.starts_with(char::is_uppercase) {
                            1
                        } else {
                            0
                        })
                        .map(String::from)
                        .collect()
                }),
                SpecKwKind::camel => Value::split(&values, |value| {
                    let mut values = Vec::new();
                    let mut value = value;
                    while let Some(idx) = value.rfind(char::is_uppercase) {
                        let found;
                        (value, found) = value.split_at(idx);
                        values.push(String::from(found));
                    }
                    values.push(String::from(value));
                    values.reverse();
                    values
                }),
                */
                SpecKwKind::camel => {
                    let mut value_vec = Vec::with_capacity(values.len());
                    for mut value in values {
                        let mut value_str = value.as_str();
                        while let Some(idx) = value_str.rfind(char::is_uppercase) {
                            let found;
                            (value_str, found) = value_str.split_at(idx);
                            value_vec.push(
                                Value::with_content(found.into())
                                    .with_ty_of(&value)
                                    .with_span_of(&value),
                            );
                        }
                        value.set_content(value_str.into());
                        value_vec.push(value);
                    }
                    value_vec.reverse();
                    value_vec.into()
                }
                SpecKwKind::pascal => {
                    let mut value_vec = Vec::with_capacity(values.len());
                    for mut value in values {
                        let mut value_str = value.as_str();
                        while let Some(idx) = value_str.rfind(char::is_uppercase) {
                            let found;
                            (value_str, found) = value_str.split_at(idx);
                            value_vec.push(
                                Value::with_content(found.into())
                                    .with_ty_of(&value)
                                    .with_span_of(&value),
                            );
                        }
                        // pascal expects value_str to be empty but handles it not being so
                        // anyways, whilst camel always adds the value_str value even if it is
                        // empty.
                        if !value_str.is_empty() {
                            value.set_content(value_str.into());
                            value_vec.push(value)
                        };
                    }
                    value_vec.reverse();
                    value_vec.into()
                }
                SpecKwKind::kebab => split_by_char('-', values),
                SpecKwKind::snake => split_by_char('_', values),
                SpecKwKind::path => split_by_str("::", values),
                SpecKwKind::space => split_by_char(' ', values),
                SpecKwKind::dot => split_by_char('.', values),
            },
        })
    }
}

impl LookaheadParse for Split {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(kw::split)
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

impl ToTokens for Split {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim, spec } = self;
        kw.to_tokens(tokens);
        delim.surround(tokens, |tokens| spec.to_tokens(tokens));
    }
}
