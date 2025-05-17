//! [Split] impl.
use ::quote::ToTokens;
use ::syn::{LitChar, LitStr, MacroDelimiter};

use crate::{
    array_expr::function::{Call, spec_impl},
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
fn split_by_char(pat: char, values: Vec<Value>) -> Vec<Value> {
    Value::split(&values, |value| {
        value.split(pat).map(String::from).collect()
    })
}

/// Split values by a [str].
#[inline]
fn split_by_str(pat: &str, values: Vec<Value>) -> Vec<Value> {
    Value::split(&values, |value| {
        value.split(pat).map(String::from).collect()
    })
}

impl Call for Split {
    fn call(&self, values: Vec<Value>) -> syn::Result<Vec<Value>> {
        Ok(match &self.spec {
            Spec::Str(lit_str) => split_by_str(&lit_str.value(), values),
            Spec::Char(lit_char) => split_by_char(lit_char.value(), values),
            Spec::Kw(spec_kw) => match spec_kw.kind {
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
