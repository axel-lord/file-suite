//! Casing conversion part of expression.

use crate::{util::kw_kind, value::Value};

kw_kind!(
    /// A parsed output case (has span).
    Case
    /// How output case should be modified.
    [expect(non_camel_case_types)]
    CaseKind (Default) {
        /// Keep case as is.
        [default]
        keep,
        /// Use camelCase.
        camel,
        /// Use PascalCase.
        pascal,
        /// Use UPPERCASE.
        upper,
        /// Use LOWERCASE.
        lower,
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
            CaseKind::keep => (),
            CaseKind::camel => {
                let mut values = values.iter_mut();
                if let Some(first) = values.next() {
                    first.remap_value(|value| value.to_lowercase());
                }
                for value in values {
                    value.remap_value(|value| capitalize(&value));
                }
            }
            CaseKind::pascal => {
                for value in values.iter_mut() {
                    value.remap_value(|value| capitalize(&value));
                }
            }
            CaseKind::upper => {
                for value in values.iter_mut() {
                    value.remap_value(|value| value.to_uppercase());
                }
            }
            CaseKind::lower => {
                for value in values.iter_mut() {
                    value.remap_value(|value| value.to_lowercase());
                }
            }
        };
        values
    }
}
