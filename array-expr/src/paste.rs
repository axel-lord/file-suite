//! Find array expressions in any token input.

use ::std::{borrow::Cow, iter, ops::ControlFlow};

use ::fold_tokens::{Cursor, FoldTokens, Response, VisitTokens, fold_tokens, visit_tokens};
use ::proc_macro2::{Punct, TokenStream};
use ::quote::ToTokens;
use ::syn::parse::Parser;
use ::tokens_rc::TokenTree;

use crate::{ParsedArrayExpr, storage::Storage, value_array::ValueArray};

/// [VisitTokens] to find variables that are interpolated.
#[derive(Debug, Default)]
struct FindVars {
    /// Found variables.
    vars: Vec<String>,
}

impl VisitTokens for FindVars {
    fn visit_ident(
        &mut self,
        ident: &syn::Ident,
        cursor: &Cursor,
    ) -> fold_tokens::Result<Response> {
        'body: {
            match cursor.get_relative(-1) {
                Some(TokenTree::Punct(punct)) if punct.as_char() == '#' => (),
                _ => break 'body,
            };

            self.vars.push(ident.to_string());
        }
        Ok(Response::Default)
    }

    fn visit_punct(&mut self, _punct: &Punct, cursor: &Cursor) -> fold_tokens::Result<Response> {
        let response = if cursor.punct_match("++!") {
            Response::Skip(4)
        } else {
            Response::Default
        };
        Ok(response)
    }

    fn visit_group(
        &mut self,
        group: &tokens_rc::OpaqueGroup,
        cursor: &Cursor,
    ) -> fold_tokens::Result<Response> {
        'body: {
            match group.delimiter() {
                ::proc_macro2::Delimiter::Parenthesis => match cursor.get_relative(-1) {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '#' => {
                        return Err(::syn::Error::new(
                            punct.span(),
                            "nested variable interpolation is not allowed",
                        ));
                    }
                    _ => break 'body,
                },
                _ => break 'body,
            }
        }
        Ok(Response::Default)
    }
}

/// [FoldTokens] for finding array expressions.
#[derive(Debug)]
pub(crate) struct ArrayExprPaste<'s> {
    /// Variable storage for paste expression.
    pub storage: &'s mut Storage,
}

impl FoldTokens for ArrayExprPaste<'_> {
    fn fold_punct(
        &mut self,
        punct: &Punct,
        cursor: &Cursor,
        tokens: &mut TokenStream,
    ) -> fold_tokens::Result<Response> {
        if punct.as_char() == '#' {
            let Some(tree) = cursor.get(1) else {
                return Ok(Response::default());
            };

            let mut skip = 2;
            let key = match tree {
                TokenTree::Literal(literal) => match ::syn::Lit::new(literal.clone()) {
                    ::syn::Lit::Str(lit_str) => lit_str.value(),
                    ::syn::Lit::Int(lit_int) => lit_int.base10_parse::<isize>()?.to_string(),
                    _ => return Ok(Response::Default),
                },
                TokenTree::Ident(ident) => ident.to_string(),
                TokenTree::Punct(punct) => match cursor.get(2) {
                    Some(TokenTree::Literal(lit)) if punct.as_char() == '-' => {
                        let ::syn::Lit::Int(lit_int) = ::syn::Lit::new(lit.clone()) else {
                            return Ok(Response::Default);
                        };
                        skip = 3;
                        lit_int.base10_parse::<isize>()?.to_string()
                    }
                    _ => return Ok(Response::Default),
                },
                TokenTree::Group(group) => {
                    match group.delimiter() {
                        ::proc_macro2::Delimiter::Parenthesis => (),
                        _ => return Ok(Response::Default),
                    }

                    let sep = match cursor.get(2) {
                        Some(TokenTree::Punct(p)) if p.as_char() == '*' => None,
                        Some(TokenTree::Punct(p)) => match cursor.get(3) {
                            Some(TokenTree::Punct(p2)) if p2.as_char() == '*' => Some(p),
                            _ => return Ok(Response::Default),
                        },
                        _ => return Ok(Response::Default),
                    };

                    let mut visitor = FindVars::default();
                    visit_tokens(&mut visitor, group.stream.clone())?;

                    visitor.vars.sort();
                    visitor.vars.dedup();

                    let mut iters = visitor
                        .vars
                        .into_iter()
                        .map(|key| {
                            let values = self.storage.try_get(&key)?.clone().into_iter();
                            Ok((key, values))
                        })
                        .collect::<Result<Vec<_>, crate::Error>>()?;

                    let stream = group.stream.clone();

                    let mut separators = iter::once(None).chain(iter::repeat(sep));

                    loop {
                        let control_flow = self.storage.with_local_layer(|storage| {
                            for (key, iter) in &mut iters {
                                let Some(value) = iter.next() else {
                                    return Ok(ControlFlow::Break(()));
                                };
                                match storage.insert(Cow::Borrowed(key), false) {
                                    Ok(values) => *values = ValueArray::from_value(value),
                                    Err(key) => return Err(crate::Error::from(format!("could not insert interpolated value '{key}' into storage"))),
                                }
                            }

                            separators.next().unwrap_or_else(|| unreachable!()).to_tokens(tokens);
                            storage.with_local_layer(|storage| {
                                fold_tokens(&mut ArrayExprPaste {storage}, stream.clone())
                            })?.to_tokens(tokens);

                            Ok(ControlFlow::Continue(()))
                        })?;

                        if matches!(control_flow, ControlFlow::Break(..)) {
                            break;
                        }
                    }

                    return Ok(Response::Skip(sep.map_or_else(|| 3, |_| 4)));
                }
            };
            let skip = skip;

            for value in self.storage.try_get(&key)? {
                value.try_to_typed()?.to_tokens(tokens);
            }

            Ok(Response::Skip(skip))
        } else if cursor.punct_match("++!") {
            let Some(::tokens_rc::TokenTree::Group(group)) = cursor.get(3) else {
                return Err(::syn::Error::new(
                    cursor[2].span(),
                    "expected delimited group following '++!'",
                ));
            };

            for node in ParsedArrayExpr::parse_multiple.parse2(group.stream())? {
                for value in self.storage.with_local_layer(|storage| {
                    node.to_array_expr().compute_with_storage(storage)
                })? {
                    value.try_to_typed()?.to_tokens(tokens);
                }
            }

            Ok(Response::Skip(4))
        } else {
            Ok(Response::Default)
        }
    }
}
