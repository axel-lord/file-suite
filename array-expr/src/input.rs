//! [Input] impl.

use ::file_suite_proc_lib::{
    Lookahead, ToArg, lookahead::ParseBufferExt, macro_delim::MacroDelimExt, macro_delimited,
    to_arg::ToArgCollection,
};
use ::proc_macro2::{Punct, TokenStream};
use ::quote::ToTokens;
use ::syn::{MacroDelimiter, Token};
use syn::parse::Parse;

use crate::{
    ArrayExpr, ParsedArrayExpr, typed_value::TypedValue, value::Value, value_array::ValueArray,
};

/// Input value of an array expression.
#[derive(Debug, Clone)]
pub enum ExprInput {
    /// A single value.
    Value(Value),
    /// A Nested array expression.
    Expr(ArrayExpr),
    /// A variable access.
    Var(String),
    /// A weak variable access.
    WeakVar(String),
}

/// [ArrayExpr] input values.
#[derive(Debug, Clone)]
pub enum ParsedExprInput {
    /// Input is variable access.
    Access {
        /// '=' token.
        eq_token: Token![=],
        /// Key of variable access.
        key: InputValue,
    },
    /// Input is weak variable access.
    WeakAccess {
        /// '?' token.
        question_token: Token![?],
        /// Key of variable access.
        key: InputValue,
    },
    /// Input is a nested expression.
    Nested {
        /// Delimiter around expression.
        delim: MacroDelimiter,
        /// Nested array expression.
        expr: ParsedArrayExpr,
    },
    /// Input is an [InputValue].
    Value(InputValue),
}

impl ParsedExprInput {
    /// Get an [ExprInput].
    pub fn to_input(&self) -> ExprInput {
        match self {
            ParsedExprInput::Nested { delim: _, expr } => ExprInput::Expr(expr.to_array_expr()),
            ParsedExprInput::Value(typed_value) => ExprInput::Value(typed_value.to_arg()),
            ParsedExprInput::Access { eq_token: _, key } => ExprInput::Var(key.to_arg().into()),
            ParsedExprInput::WeakAccess {
                question_token: _,
                key,
            } => ExprInput::WeakVar(key.to_arg().into()),
        }
    }
}

impl Lookahead for ParsedExprInput {
    fn lookahead_peek(lookahead: &syn::parse::Lookahead1) -> bool {
        MacroDelimiter::lookahead_peek(lookahead)
            || lookahead.peek(Token![=])
            || lookahead.peek(Token![?])
            || TypedValue::lookahead_peek(lookahead)
    }

    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>>
    where
        Self: Parse,
    {
        Ok(Some(if MacroDelimiter::lookahead_peek(lookahead) {
            let content;
            let delim = macro_delimited!(content in input);
            let expr = content.parse()?;

            Self::Nested { delim, expr }
        } else if lookahead.peek(Token![=]) {
            Self::Access {
                eq_token: input.parse()?,
                key: input.parse()?,
            }
        } else if lookahead.peek(Token![?]) {
            Self::WeakAccess {
                question_token: input.parse()?,
                key: input.parse()?,
            }
        } else if let Some(value) = input.lookahead_parse(lookahead)? {
            Self::Value(value)
        } else {
            return Ok(None);
        }))
    }
}

impl Parse for ParsedExprInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        match input.lookahead_parse(&lookahead)? {
            Some(value) => Ok(value),
            None => Err(lookahead.error()),
        }
    }
}

impl ToTokens for ParsedExprInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ParsedExprInput::Nested { delim, expr } => {
                delim.surround(tokens, |tokens| expr.to_tokens(tokens))
            }
            ParsedExprInput::Value(typed_value) => typed_value.to_tokens(tokens),
            ParsedExprInput::Access { eq_token, key } => {
                eq_token.to_tokens(tokens);
                key.to_tokens(tokens);
            }
            ParsedExprInput::WeakAccess {
                question_token,
                key,
            } => {
                question_token.to_tokens(tokens);
                key.to_tokens(tokens);
            }
        }
    }
}

/// A [TypedValue] which may also be an escaped punctuation character.
#[derive(Debug, Clone)]
pub enum InputValue {
    /// Input is a typed value.
    TypedValue(TypedValue),
    /// Input is some escaped punctuation.
    Escaped {
        /// '/' character
        slash: Token![/],
        /// Punctuation
        punct: Punct,
    },
}

impl Lookahead for InputValue {
    fn lookahead_peek(lookahead: &syn::parse::Lookahead1) -> bool {
        lookahead.peek(Token![/]) || TypedValue::lookahead_peek(lookahead)
    }

    fn input_peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![/]) || TypedValue::input_peek(input)
    }

    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>>
    where
        Self: Parse,
    {
        let value = if lookahead.peek(Token![/]) {
            Self::Escaped {
                slash: input.parse()?,
                punct: input.step(|cursor| match cursor.punct() {
                    Some((punct, cursor)) => Ok((punct, cursor)),
                    None => Err(cursor.error("expected a punctuation character after '/'")),
                })?,
            }
        } else if let Some(value) = input.lookahead_parse(lookahead)? {
            Self::TypedValue(value)
        } else {
            return Ok(None);
        };
        Ok(Some(value))
    }
}

impl Parse for InputValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        match input.lookahead_parse(&lookahead)? {
            Some(value) => Ok(value),
            None => Err(lookahead.error()),
        }
    }
}

impl ToTokens for InputValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            InputValue::TypedValue(typed_value) => typed_value.to_tokens(tokens),
            InputValue::Escaped { slash, punct } => {
                slash.to_tokens(tokens);
                punct.to_tokens(tokens);
            }
        }
    }
}

impl ToArg for InputValue {
    type Arg = Value;

    fn to_arg(&self) -> Self::Arg {
        match self {
            InputValue::TypedValue(typed_value) => typed_value.to_value(),
            InputValue::Escaped { slash: _, punct } => Value::new_tokens(punct.to_token_stream()),
        }
    }
}

impl ToArgCollection for InputValue {
    type Collection = ValueArray;
}
