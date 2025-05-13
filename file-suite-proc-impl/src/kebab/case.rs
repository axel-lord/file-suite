//! Casing conversion part of expression.

use crate::{util::kw_kind, value::Value};

kw_kind!(
    /// A parsed output case (has span).
    Case
    /// How output case should be modified.
    CaseKind (Default) {
        /// Keep case as is.
        [default]
        Keep keep,
        /// Use camelCase.
        Camel camel,
        /// Use PascalCase.
        Pascal pascal,
        /// Use UPPERCASE.
        Upper upper,
        /// Use LOWERCASE.
        Lower lower,
    }
);

impl CaseKind {
    /// Apply casing to a string.
    pub fn apply(self, mut values: Vec<Value>) -> Vec<Value> {
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
        match self {
            CaseKind::Keep => (),
            CaseKind::Camel => {
                let mut values = values.iter_mut();
                if let Some(first) = values.next() {
                    first.remap_value(|value| value.to_lowercase());
                }
                for value in values {
                    value.remap_value(|value| capitalize(&value));
                }
            }
            CaseKind::Pascal => {
                for value in values.iter_mut() {
                    value.remap_value(|value| capitalize(&value));
                }
            }
            CaseKind::Upper => {
                for value in values.iter_mut() {
                    value.remap_value(|value| value.to_uppercase());
                }
            }
            CaseKind::Lower => {
                for value in values.iter_mut() {
                    value.remap_value(|value| value.to_lowercase());
                }
            }
        };
        values
    }
}
