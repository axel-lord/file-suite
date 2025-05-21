//! [Input] impl.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::MacroDelimiter;

use crate::{
    array_expr::ArrayExpr,
    macro_delimited,
    typed_value::TypedValue,
    util::{delimited::MacroDelimExt, lookahead_parse::LookaheadParse},
};

/// [ArrayExpr] input values.
#[derive(Debug)]
pub enum Input {
    /// Input is a nested expression.
    Nested {
        /// Delimiter around expression.
        delim: MacroDelimiter,
        /// Nested array expression.
        expr: ArrayExpr,
    },
    /// Input is a [TypedValue].
    Value(TypedValue),
}

impl LookaheadParse for Input {
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

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Input::Nested { delim, expr } => {
                delim.surround(tokens, |tokens| expr.to_tokens(tokens))
            }
            Input::Value(typed_value) => typed_value.to_tokens(tokens),
        }
    }
}
