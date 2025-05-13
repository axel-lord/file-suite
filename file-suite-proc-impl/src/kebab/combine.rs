//! Implementation of combine part of expression.

use ::quote::ToTokens;
use ::syn::{
    LitChar, LitStr,
    parse::{Lookahead1, Parse, ParseStream},
};

use crate::{
    util::kw_kind,
    value::{TyKind, Value},
};

/// Part of expression deciding how output should be combined.
#[derive(Debug, Clone)]
pub enum Combine {
    /// Combine by given value.
    LitStr(LitStr),
    /// Combine by given value.
    LitChar(LitChar),
    /// Decide by a single keyword.
    Keyword(CombineKeyword),
}

impl Combine {
    /// Parse an instance if lookahead peek matches.
    ///
    /// # Errors
    /// If a valid value peeked by lookahead cannot be parsed.
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> ::syn::Result<Option<Self>> {
        Ok(Some(
            if let Some(value) = CombineKeyword::lookahead_parse(input, lookahead)? {
                Self::Keyword(value)
            } else if lookahead.peek(LitStr) {
                Self::LitStr(input.parse()?)
            } else if lookahead.peek(LitChar) {
                Self::LitChar(input.parse()?)
            } else {
                return Ok(None);
            },
        ))
    }
}

impl Parse for Combine {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if let Some(value) = Self::lookahead_parse(input, &lookahead)? {
            Ok(value)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Combine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Combine::Keyword(combine_keyword) => combine_keyword.to_tokens(tokens),
            Combine::LitStr(lit_str) => lit_str.to_tokens(tokens),
            Combine::LitChar(lit_char) => lit_char.to_tokens(tokens),
        }
    }
}

kw_kind!(
    /// Keyword for how output should be combined.
    CombineKeyword
    /// How output should be combined.
    CombineKeywordKind (Default) {
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

impl CombineKeywordKind {
    /// Join input arguments.
    pub fn join(self, values: Vec<Value>) -> Vec<Value> {
        if matches!(self, CombineKeywordKind::Split) {
            values
        } else {
            vec![Value::join(values, |values| match self {
                Self::Concat => values.join(""),
                Self::Kebab => values.join("-"),
                Self::Snake => values.join("_"),
                Self::Space => values.join(" "),
                Self::Count => values.len().to_string(),
                Self::First => values
                    .into_iter()
                    .next()
                    .map(String::from)
                    .unwrap_or_default(),
                Self::Last => values
                    .into_iter()
                    .next_back()
                    .map(String::from)
                    .unwrap_or_default(),
                Self::Split => unreachable!(),
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
