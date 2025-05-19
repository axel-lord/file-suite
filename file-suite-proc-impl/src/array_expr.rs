//! Expressions working with arrays of strings at compile-time.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Parse, ParseStream},
    spanned::Spanned,
};

use crate::{
    array_expr::{
        function::{Call, Function},
        input::Input,
        value_array::ValueArray,
    },
    util::lookahead_parse::LookaheadParse,
    value::Value,
};

pub(crate) use paste::ArrayExprPaste;

mod paste;

pub mod value_array;

pub mod input;

pub mod function;

/// Parsed array expression.
#[derive(Debug, Default)]
pub enum ArrayExpr {
    /// Empty Expression.
    #[default]
    Empty,
    /// Stringify given input.
    Stringify {
        /// '!' token.
        not_token: Token![!],
        /// Remaining tokens.
        remainder: TokenStream,
    },
    /// Take and transform input.
    Transform {
        /// Input values.
        input: Vec<Input>,
        /// '->' token.
        arrow_token: Option<Token![->]>,
        /// Transform chain.
        chain: Vec<(Option<Token![.]>, Function)>,
    },
}

impl ArrayExpr {
    /// Compute parsed expression.
    ///
    /// # Errors
    /// If the expression cannot be computed.
    pub fn compute(&self) -> ::syn::Result<ValueArray> {
        let (input, chain) = match self {
            ArrayExpr::Empty => return Ok(ValueArray::new()),
            ArrayExpr::Stringify {
                not_token: _,
                remainder,
            } => {
                return Ok(ValueArray::from_value({
                    let mut value = Value::from(remainder.to_string());
                    value.set_span(remainder.span());
                    value
                }));
            }
            ArrayExpr::Transform {
                input,
                arrow_token: _,
                chain,
            } => (input, chain),
        };

        let mut values = ValueArray::new();
        let value_vec = values.make_vec();
        for input in input {
            match input {
                Input::Nested { delim: _, expr } => {
                    let extend_with = expr.compute()?;
                    value_vec.reserve(extend_with.len());
                    value_vec.extend(extend_with);
                }
                Input::Value(typed_value) => value_vec.push(typed_value.try_to_value()?),
            }
        }

        for (_, f) in chain {
            values = f.call(values)?;
        }

        Ok(values)
    }
}

impl Parse for ArrayExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(End) {
            return Ok(Self::default());
        }

        if lookahead.peek(Token![!]) {
            return Ok(Self::Stringify {
                not_token: input.parse()?,
                remainder: input.parse()?,
            });
        }

        let (input_vec, arrow_token) = if lookahead.peek(Token![->]) {
            (Vec::new(), Some(input.parse()?))
        } else if let Some(first) = Input::lookahead_parse(input, &lookahead)? {
            let mut input_vec = Vec::from([first]);
            loop {
                let lookahead = input.lookahead1();

                if lookahead.peek(End) {
                    return Ok(Self::Transform {
                        input: input_vec,
                        arrow_token: None,
                        chain: Vec::new(),
                    });
                }

                if lookahead.peek(Token![->]) {
                    break (input_vec, Some(input.parse()?));
                }

                if let Some(value) = Input::lookahead_parse(input, &lookahead)? {
                    input_vec.push(value);
                    continue;
                }

                return Err(lookahead.error());
            }
        } else {
            return Err(lookahead.error());
        };

        let lookahead = input.lookahead1();

        // Parse first element with optional leading dot.
        let first = if lookahead.peek(End) {
            return Ok(Self::Transform {
                input: input_vec,
                arrow_token,
                chain: Vec::new(),
            });
        } else if lookahead.peek(Token![.]) {
            (Some(input.parse()?), input.call(Function::parse)?)
        } else if let Some(f) = Function::lookahead_parse(input, &lookahead)? {
            (None, f)
        } else {
            return Err(lookahead.error());
        };

        let mut chain = Vec::from([first]);
        loop {
            let lookahead = input.lookahead1();

            if lookahead.peek(End) {
                return Ok(Self::Transform {
                    input: input_vec,
                    arrow_token,
                    chain,
                });
            }

            if lookahead.peek(Token![.]) {
                chain.push((Some(input.parse()?), input.call(Function::parse)?));
                continue;
            }

            return Err(lookahead.error());
        }
    }
}

impl ToTokens for ArrayExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ArrayExpr::Empty => {}
            ArrayExpr::Stringify {
                not_token,
                remainder,
            } => {
                not_token.to_tokens(tokens);
                remainder.to_tokens(tokens);
            }
            ArrayExpr::Transform {
                input,
                arrow_token,
                chain,
            } => {
                for value in input {
                    value.to_tokens(tokens);
                }
                arrow_token.to_tokens(tokens);
                for (dot, f) in chain {
                    dot.to_tokens(tokens);
                    f.to_tokens(tokens);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use ::quote::quote;

    use crate::array_expr;

    #[test]
    fn case_convert() {
        let expr = quote! {"from-kebab-to-camel" -> split(kebab).case(camel).join.ty(ident)};
        let exected = quote! {fromKebabToCamel};
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), exected.to_string());

        let expr = quote! {1 0 0 0 -> join.ty(int)};
        let expected = quote! {1000};
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());

        let expr = quote! {CamelToSnake -> split(camel).case(lower).join(snake).ty(ident) };
        let expected = quote! {_camel_to_snake};
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());

        let expr = quote! {(!stringify(expression)) -> ty(str)};
        let expected = quote! {"stringify (expression)"};
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());

        let expr = quote! {(!split::a::path) -> split(path).ty(ident)};
        let expected = quote! {split a path};
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());

        let expr = quote! {(! enum Item { Roundtrip }) -> ty(item)};
        let expected = quote! {enum Item { Roundtrip }};
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());
    }
}
