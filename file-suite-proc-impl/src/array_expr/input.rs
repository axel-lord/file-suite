//! [Input] impl.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::MacroDelimiter;

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
}

/// [ArrayExpr] input values.
#[derive(Debug, Clone)]
pub enum NodeInput {
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
        }
    }
}

impl LookaheadParse for NodeInput {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        if MacroDelimiter::lookahead_peek(lookahead) {
            let content;
            let delim = macro_delimited!(content in input);
            let expr = content.parse()?;

            return Ok(Some(Self::Nested { delim, expr }));
        }

        if let Some(value) = TypedValue::lookahead_parse(input, lookahead)? {
            return Ok(Some(Self::Value(value)));
        }

        Ok(None)
    }
}

impl ToTokens for NodeInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NodeInput::Nested { delim, expr } => {
                delim.surround(tokens, |tokens| expr.to_tokens(tokens))
            }
            NodeInput::Value(typed_value) => typed_value.to_tokens(tokens),
        }
    }
}
