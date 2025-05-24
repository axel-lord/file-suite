//! [split] impl.
use std::borrow::Cow;

use ::syn::{LitChar, LitStr};

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct, spec_impl},
        storage::Storage,
        value::Value,
        value_array::ValueArray,
    },
    util::{group_help::GroupSingle, kw_kind, lookahead_parse::ParseWrap},
};

kw_kind!(
    /// Keyword specified split
    SpecKeyword;
    /// Enum containing possible values for [SpecKw].
    #[expect(non_camel_case_types)]
    SplitKind {
        /// Split by PascalCase.
        pascal,
        /// Split by camelCase.
        camel,
        /// Split by dashes '-'.
        kebab,
        /// Split by underscores '_'.
        snake,
        /// Split by double colons '::'.
        path,
        /// Split by spaces ' '.
        space,
        /// Split by dots '.'.
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
        Kw(SpecKeyword),
    }
);

function_struct!(
    /// Split input further.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    split {
        /// Specification for to split value
        spec: GroupSingle<ParseWrap<Spec>>,
    }
);

/// [Call] implementor for split.
#[derive(Debug, Clone)]
pub enum SplitCallable {
    /// Split by a string.
    Str(String),
    /// Split by a char.
    Char(char),
    /// Split according to a keyword.
    Kw(SplitKind),
}

impl ToCallable for split {
    type Call = SplitCallable;

    fn to_callable(&self) -> Self::Call {
        match &self.spec.content.0 {
            Spec::Str(lit_str) => SplitCallable::Str(lit_str.value()),
            Spec::Char(lit_char) => SplitCallable::Char(lit_char.value()),
            Spec::Kw(spec_kw) => SplitCallable::Kw(spec_kw.kind),
        }
    }
}

impl Call for SplitCallable {
    fn call(&self, values: ValueArray, _: &mut Storage) -> Result<ValueArray, Cow<'static, str>> {
        Ok(match self {
            Self::Str(pat) => values.split_by_str(pat),
            Self::Char(pat) => {
                let mut buf = [0u8; 4];
                let pat = pat.encode_utf8(&mut buf);
                values.split_by_str(pat)
            }
            Self::Kw(kw_kind) => match kw_kind {
                SplitKind::camel => {
                    let mut value_vec = Vec::with_capacity(values.len());
                    for value in values {
                        let mut value_str = value.as_str();
                        while let Some(idx) = value_str.rfind(char::is_uppercase) {
                            let found;
                            (value_str, found) = value_str.split_at(idx);
                            value_vec.push(
                                Value::new(found.into())
                                    .with_ty_of(&value)
                                    .with_span_of(&value),
                            );
                        }
                        let content = String::from(value_str);
                        value_vec.push(value.with_content(content));
                    }
                    value_vec.reverse();
                    value_vec.into()
                }
                SplitKind::pascal => {
                    let mut value_vec = Vec::with_capacity(values.len());
                    for value in values {
                        let mut value_str = value.as_str();
                        while let Some(idx) = value_str.rfind(char::is_uppercase) {
                            let found;
                            (value_str, found) = value_str.split_at(idx);
                            value_vec.push(
                                Value::new(found.into())
                                    .with_ty_of(&value)
                                    .with_span_of(&value),
                            );
                        }
                        // pascal expects value_str to be empty but handles it not being so
                        // anyways, whilst camel always adds the value_str value even if it is
                        // empty.
                        if !value_str.is_empty() {
                            // value.set_content(value_str.into());
                            let content = String::from(value_str);
                            value_vec.push(value.with_content(content))
                        };
                    }
                    value_vec.reverse();
                    value_vec.into()
                }
                SplitKind::kebab => values.split_by_str("-"),
                SplitKind::snake => values.split_by_str("_"),
                SplitKind::path => values.split_by_str("::"),
                SplitKind::space => values.split_by_str(" "),
                SplitKind::dot => values.split_by_str("."),
            },
        })
    }
}
