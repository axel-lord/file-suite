//! [Split] impl.
use ::quote::ToTokens;
use ::syn::{LitChar, LitStr, MacroDelimiter};

use crate::{
    array_expr::{
        function::{Call, ToCallable, spec_impl},
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

/// [Call] implementor for split.
#[derive(Debug, Clone)]
pub enum SplitCallable {
    /// Split by a string.
    Str(String),
    /// Split by a char.
    Char(char),
    /// Split according to a keyword.
    Kw(SpecKwKind),
}

impl ToCallable for Split {
    type Call = SplitCallable;

    fn to_callable(&self) -> Self::Call {
        match &self.spec {
            Spec::Str(lit_str) => SplitCallable::Str(lit_str.value()),
            Spec::Char(lit_char) => SplitCallable::Char(lit_char.value()),
            Spec::Kw(spec_kw) => SplitCallable::Kw(spec_kw.kind),
        }
    }
}

impl Call for SplitCallable {
    fn call(&self, values: ValueArray) -> syn::Result<ValueArray> {
        Ok(match self {
            Self::Str(pat) => values.split_by_str(pat),
            Self::Char(pat) => {
                let mut buf = [0u8; 4];
                let pat = pat.encode_utf8(&mut buf);
                values.split_by_str(pat)
            }
            Self::Kw(kw_kind) => match kw_kind {
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
                SpecKwKind::kebab => values.split_by_str("-"),
                SpecKwKind::snake => values.split_by_str("_"),
                SpecKwKind::path => values.split_by_str("::"),
                SpecKwKind::space => values.split_by_str(" "),
                SpecKwKind::dot => values.split_by_str("."),
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
