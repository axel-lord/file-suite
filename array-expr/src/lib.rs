//! Expressions working with arrays of strings at compile-time.

use ::file_suite_proc_lib::Lookahead;
use ::proc_macro2::{Span, TokenStream};
use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Lookahead1, Parse, ParseStream, Parser},
    punctuated::Punctuated,
};

use crate::{
    function::{Call, Function, FunctionChain, ToCallable},
    input::{Input, NodeInput},
    storage::Storage,
    value::Value,
    value_array::ValueArray,
};

pub(crate) use paste::ArrayExprPaste;

mod error;
mod paste;

pub mod from_values;
pub mod function;
pub mod input;
pub mod storage;
pub mod typed_value;
pub mod value;
pub mod value_array;

/// Crate result type.
pub type Result<T = ()> = ::std::result::Result<T, crate::error::Error>;

pub use error::Error;

/// Find array expression in input tokens and compute them, replacing them with their result.
///
/// # Errors
/// If the expression cannot be parsed.
/// Or if it cannot be computed.
pub fn array_expr_paste(input: TokenStream) -> ::syn::Result<TokenStream> {
    ::fold_tokens::fold_tokens(
        &mut ArrayExprPaste {
            storage: &mut Storage::initial(),
        },
        input.into(),
    )
}

/// Compute array expression expressed as macro input.
///
/// # Errors
/// If the expression cannot be parsed.
/// Or if it cannot be computed.
pub fn array_expr(input: TokenStream) -> ::syn::Result<TokenStream> {
    let mut tokens = TokenStream::default();
    let mut storage = Storage::initial();
    for node in Node::parse_multiple.parse2(input)? {
        for value in storage
            .with_local_layer(|storage| node.to_array_expr().compute_with_storage(storage))?
        {
            value.try_to_typed()?.to_tokens(&mut tokens);
        }
    }
    Ok(tokens)
}

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
                Input::Var(key) => value_vec.extend(storage.try_get(key)?.iter().cloned()),
                Input::WeakVar(key) => {
                    value_vec.extend(storage.try_get(key).into_iter().flatten().cloned())
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
        self.compute_with_storage(&mut Storage::initial())
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
        chain: FunctionChain,
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
                let value = Value::new_tokens(remainder.clone());
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
                let chain = chain.to_call_chain();

                ArrayExpr { input, chain }
            }
        }
    }

    /// Parse multiple array expressions, terminated by commas ','.
    ///
    /// # Errors
    /// If input cannot be parsed.
    pub fn parse_multiple(input: ParseStream) -> ::syn::Result<Punctuated<Self, Token![,]>> {
        Punctuated::parse_terminated_with(input, |input| {
            Self::parse_terminated(input, |lookahead| {
                lookahead.peek(End) || lookahead.peek(Token![,])
            })
        })
    }

    /// Parse with a custom termination condition.
    ///
    /// # Errors
    /// If input cannot be parsed to Self.
    pub fn parse_terminated(
        input: ParseStream,
        should_terminate: fn(&Lookahead1) -> bool,
    ) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();

        if should_terminate(&lookahead) {
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

                if should_terminate(&lookahead) {
                    return Ok(Self::Transform {
                        input: input_vec,
                        arrow_token: None,
                        chain: FunctionChain::default(),
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
            chain: FunctionChain::parse_terminated(input, should_terminate)?,
        })
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_terminated(input, |lookahead| lookahead.peek(End))
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
                chain.to_tokens(tokens);
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

    #[test]
    fn stringify() {
        assert_arr_expr!(
            { (!stringify(expression)) -> ty(str) },
            { "stringify (expression)" },
        );
    }

    /// Assert the result of an array expression.
    macro_rules! assert_arr_expr {
        ({$($expr:tt)*}, {$($expected:tt)*} $(,)?) => {{
            let result = $crate::array_expr(::quote::quote!($($expr)*)).unwrap().to_string();
            let expected = ::quote::quote!($($expected)*).to_string();
            assert_eq!(result, expected);
        }};
    }
    pub(crate) use assert_arr_expr;
}
