//! Expressions working with arrays of strings at compile-time.

use ::proc_macro2::{Span, TokenStream};
use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Parse, ParseStream},
    spanned::Spanned,
};

use crate::{
    array_expr::{
        function::{Call, Function, ToCallable},
        input::{Input, NodeInput},
        storage::Storage,
        value::Value,
        value_array::ValueArray,
    },
    util::lookahead_parse::LookaheadParse,
};

pub(crate) use paste::ArrayExprPaste;

mod paste;

pub mod storage;

pub mod value_array;

pub mod input;

pub mod function;

pub mod value;

pub mod typed_value;

/// Array expression. Without parsing details.
#[derive(Debug, Clone, Default)]
pub struct ArrayExpr {
    /// Input values of expression.
    input: Vec<Input>,
    /// Function chain transforming input.
    chain: Vec<<Function as ToCallable>::Call>,
}

impl ArrayExpr {
    /// Compute array expression.
    /// Passed storage is used as furthest backing storage.
    ///
    /// # Errors
    /// If any function errors.
    pub fn compute_with_storage(&self, storage: &mut Storage) -> ::syn::Result<ValueArray> {
        let Self { input, chain } = self;
        let mut value_array = ValueArray::new();
        let value_vec = value_array.make_vec();

        for input in input {
            match input {
                Input::Value(value) => value_vec.push(value.clone()),
                Input::Expr(array_expr) => {
                    value_vec.extend(array_expr.compute_with_storage(storage)?)
                }
                Input::Var(key) => value_vec.extend(
                    storage
                        .get(key)
                        .ok_or_else(|| {
                            ::syn::Error::new(
                                Span::call_site(),
                                format!("could not find variable '{key}'"),
                            )
                        })?
                        .iter()
                        .cloned(),
                ),
                Input::WeakVar(key) => {
                    value_vec.extend(storage.get(key).into_iter().flatten().cloned())
                }
            }
        }

        let span = value_array.span();
        for func in chain {
            value_array = match func.call(value_array, storage) {
                Ok(value_array) => value_array,
                Err(msg) => {
                    return Err(::syn::Error::new(span.unwrap_or_else(Span::call_site), msg));
                }
            }
        }

        Ok(value_array)
    }

    /// Compute array expression.
    ///
    /// # Errors
    /// If any function errors.
    pub fn compute(&self) -> ::syn::Result<ValueArray> {
        self.compute_with_storage(&mut Storage::default())
    }
}

/// Parsed array expression.
#[derive(Debug, Default, Clone)]
pub enum Node {
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
        input: Vec<NodeInput>,
        /// '->' token.
        arrow_token: Option<Token![->]>,
        /// Transform chain.
        chain: Vec<(Option<Token![.]>, Function)>,
    },
}

impl Node {
    /// Get [ArrayExpr] from this node.
    pub fn to_array_expr(&self) -> ArrayExpr {
        match self {
            Node::Empty => ArrayExpr::default(),
            Node::Stringify {
                not_token: _,
                remainder,
            } => {
                let mut value = Value::new_str(remainder.to_string());
                value.set_span(remainder.span());
                ArrayExpr {
                    input: vec![Input::Value(value)],
                    ..Default::default()
                }
            }
            Node::Transform {
                input,
                arrow_token: _,
                chain,
            } => {
                let input = input.iter().map(NodeInput::to_input).collect();
                let chain = chain.iter().map(|(_, f)| f.to_callable()).collect();

                ArrayExpr { input, chain }
            }
        }
    }
}

impl Parse for Node {
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
        } else if let Some(first) = NodeInput::lookahead_parse(input, &lookahead)? {
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

                if let Some(value) = NodeInput::lookahead_parse(input, &lookahead)? {
                    input_vec.push(value);
                    continue;
                }

                return Err(lookahead.error());
            }
        } else {
            return Err(lookahead.error());
        };

        Ok(Self::Transform {
            input: input_vec,
            arrow_token,
            chain: Function::parse_chain(input)?,
        })
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Node::Empty => {}
            Node::Stringify {
                not_token,
                remainder,
            } => {
                not_token.to_tokens(tokens);
                remainder.to_tokens(tokens);
            }
            Node::Transform {
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
