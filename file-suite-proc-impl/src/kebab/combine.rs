//! Implementation of combine part of expression.

use crate::{
    kebab::value::{TyKind, Value},
    util::kw_kind,
};

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
        /// Values should be counted.
        Count count,
        /// Only the first value should be used.
        First first,
        /// Only the last value should be used.
        Last last,
        /// Values should not be combined.
        Split split,
    }
);

impl CombineKind {
    /// Join input arguments.
    pub fn join(self, values: Vec<Value>) -> Vec<Value> {
        if matches!(self, CombineKind::Split) {
            values
        } else {
            vec![Value::join(values, |values| match self {
                CombineKind::Concat => values.join(""),
                CombineKind::Kebab => values.join("-"),
                CombineKind::Snake => values.join("_"),
                CombineKind::Space => values.join(" "),
                CombineKind::Count => values.len().to_string(),
                CombineKind::First => values
                    .into_iter()
                    .next()
                    .map(String::from)
                    .unwrap_or_default(),
                CombineKind::Last => values
                    .into_iter()
                    .next_back()
                    .map(String::from)
                    .unwrap_or_default(),
                CombineKind::Split => unreachable!(),
            })]
        }
    }

    /// Preferred [TyKind] of variant.
    pub const fn default_ty(self) -> Option<TyKind> {
        Some(match self {
            Self::Count => TyKind::LitInt,
            Self::Space | Self::Kebab => TyKind::LitStr,
            _ => return None,
        })
    }
}
