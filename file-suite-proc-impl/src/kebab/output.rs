//! [Combine], [Case] and [Ty] impls.

use ::syn::parse::Parse;

use crate::util::kw_kind;

kw_kind!(
    Combine
    /// How output should be combined.
    CombineKind (Default) {
        /// Values should be concatenated without any separator.
        [default]
        Concat concat,
        /// Values should be joined by a dash,
        Kebab kebab,
        /// Values should be joined by an underscore.
        Snake snake,
        /// Values should be joined by a space.
        Space space,
    }
);

impl CombineKind {
    /// Join input arguments.
    pub fn join(self, values: Vec<String>) -> String {
        values.join(match self {
            CombineKind::Concat => "",
            CombineKind::Kebab => "-",
            CombineKind::Snake => "_",
            CombineKind::Space => " ",
        })
    }
}

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
    pub fn apply(self, mut values: Vec<String>) -> Vec<String> {
        fn titlecase(value: &str) -> String {
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
                    *first = first.to_lowercase();
                }
                for value in values {
                    *value = titlecase(value);
                }
            }
            CaseKind::Pascal => {
                for value in values.iter_mut() {
                    *value = titlecase(value);
                }
            }
            CaseKind::Upper => {
                for value in values.iter_mut() {
                    *value = value.to_uppercase();
                }
            }
            CaseKind::Lower => {
                for value in values.iter_mut() {
                    *value = value.to_lowercase();
                }
            }
        };
        values
    }
}

kw_kind!(
    /// A parsed output type (has span).
    Ty
    /// What kind of output tokens to produce.
    TyKind (Default) {
        /// Output an identifier.
        [default]
        Ident ident,
        /// Output a string literal.
        LitStr str,
    }
);
