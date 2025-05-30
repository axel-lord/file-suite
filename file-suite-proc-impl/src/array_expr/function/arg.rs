//! Utilities for arguments which may be gotten from variables.

use ::quote::ToTokens;

use crate::{
    array_expr::{
        from_values::FromValues, function::ToArg, storage::Storage, typed_value::TypedValue,
    },
    util::lookahead_parse::{LookaheadParse, lookahead_parse},
};

/// An argument that may be either a variable access or a value.
#[derive(Debug, Clone)]
pub enum Arg<V> {
    /// Argument is variable access.
    Variable(String),
    /// Argument is a value.
    Value(V),
}

impl<V> Arg<V> {
    /// Get value, from self if possible otherwise try storage.
    ///
    /// # Errors
    /// If the variable does not exists or cannot be converted to required value.
    pub fn get(&self, storage: &Storage) -> crate::Result<V>
    where
        V: FromValues + Clone,
    {
        match self {
            Arg::Variable(key) => storage
                .get(key)
                .ok_or_else(|| {
                    crate::Error::from(format!("could not get variable with key '{key}'"))
                })
                .and_then(|values| V::from_values(values)),
            Arg::Value(value) => Ok(value.clone()),
        }
    }
}

/// Argument that can be parsed, and may be a variable access.
#[derive(Debug, Clone)]
pub enum ParsedArg<V> {
    /// Argument is variable access.
    Variable {
        /// '=' token.
        eq_token: ::syn::token::Eq,
        /// Variable to get.
        var: TypedValue,
    },
    /// Argument is a value passed as-is.
    Value(V),
}

impl<V> ToArg for ParsedArg<V>
where
    V: ToArg,
{
    type Arg = Arg<V::Arg>;

    fn to_arg(&self) -> Self::Arg {
        match self {
            ParsedArg::Variable { eq_token: _, var } => Arg::Variable(var.to_value().into()),
            ParsedArg::Value(value) => Arg::Value(value.to_arg()),
        }
    }
}

impl<V> LookaheadParse for ParsedArg<V>
where
    V: LookaheadParse,
{
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        if let Some(eq_token) = lookahead_parse(input, lookahead)? {
            Ok(Some(Self::Variable {
                eq_token,
                var: input.call(TypedValue::parse)?,
            }))
        } else if let Some(value) = lookahead_parse(input, lookahead)? {
            Ok(Some(Self::Value(value)))
        } else {
            Ok(None)
        }
    }
}

impl<V> ToTokens for ParsedArg<V>
where
    V: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ParsedArg::Variable { eq_token, var } => {
                eq_token.to_tokens(tokens);
                var.to_tokens(tokens);
            }
            ParsedArg::Value(val) => val.to_tokens(tokens),
        }
    }
}
