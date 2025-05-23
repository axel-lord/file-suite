//! [Input] impl.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{MacroDelimiter, Token};

use crate::{
    array_expr::{ArrayExpr, Node, typed_value::TypedValue, value::Value},
    macro_delimited,
    util::{delimited::MacroDelimExt, lookahead_parse::LookaheadParse},
};

/// Input value of an array expression.
#[derive(Debug, Clone)]
pub enum Input {
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
pub enum NodeInput {
    /// Input is variable access.
    Access {
        /// '=' token.
        eq_token: Token![=],
        /// Key of variable access.
        key: TypedValue,
    },
    /// Input is weak variable access.
    WeakAccess {
        /// '?' token.
        question_token: Token![?],
        /// Key of variable access.
        key: TypedValue,
    },
    /// Input is a nested expression.
    Nested {
        /// Delimiter around expression.
        delim: MacroDelimiter,
        /// Nested array expression.
        expr: Node,
    },
    /// Input is a [TypedValue].
    Value(TypedValue),
}

impl NodeInput {
    /// Get an [Input].
    pub fn to_input(&self) -> Input {
        match self {
            NodeInput::Nested { delim: _, expr } => Input::Expr(expr.to_array_expr()),
            NodeInput::Value(typed_value) => Input::Value(typed_value.to_value()),
            NodeInput::Access { eq_token: _, key } => Input::Var(key.to_value().into()),
            NodeInput::WeakAccess {
                question_token: _,
                key,
            } => Input::WeakVar(key.to_value().into()),
        }
    }
}

impl LookaheadParse for NodeInput {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        Ok(Some(if MacroDelimiter::lookahead_peek(lookahead) {
            let content;
            let delim = macro_delimited!(content in input);
            let expr = content.parse()?;

            Self::Nested { delim, expr }
        } else if lookahead.peek(Token![=]) {
            Self::Access {
                eq_token: input.parse()?,
                key: input.call(LookaheadParse::parse)?,
            }
        } else if lookahead.peek(Token![?]) {
            Self::WeakAccess {
                question_token: input.parse()?,
                key: input.call(LookaheadParse::parse)?,
            }
        } else if let Some(value) = TypedValue::lookahead_parse(input, lookahead)? {
            Self::Value(value)
        } else {
            return Ok(None);
        }))
    }
}

impl ToTokens for NodeInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NodeInput::Nested { delim, expr } => {
                delim.surround(tokens, |tokens| expr.to_tokens(tokens))
            }
            NodeInput::Value(typed_value) => typed_value.to_tokens(tokens),
            NodeInput::Access { eq_token, key } => {
                eq_token.to_tokens(tokens);
                key.to_tokens(tokens);
            }
            NodeInput::WeakAccess {
                question_token,
                key,
            } => {
                question_token.to_tokens(tokens);
                key.to_tokens(tokens);
            }
        }
    }
}
