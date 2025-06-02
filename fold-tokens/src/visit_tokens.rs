//! [VisitTokens] trait an [visit_tokens] function.

use ::std::num::NonZero;

use ::proc_macro2::{Literal, Punct};
use ::syn::Ident;
use ::tokens_rc::{Group, OpaqueGroup, TokensRc};

use crate::{Cursor, Response, Result};

/// Trait for token visitors to implement.
pub trait VisitTokens {
    /// Visit a punctuation token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn visit_punct(&mut self, punct: &Punct, cursor: &Cursor) -> Result<Response> {
        _ = (punct, cursor);
        Ok(Response::Default)
    }

    /// Visit a literal token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn visit_literal(&mut self, literal: &Literal, cursor: &Cursor) -> Result<Response> {
        _ = (literal, cursor);
        Ok(Response::Default)
    }

    /// Visit an ident token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn visit_ident(&mut self, ident: &Ident, cursor: &Cursor) -> Result<Response> {
        _ = (ident, cursor);
        Ok(Response::Default)
    }

    /// Visit n group token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn visit_group(&mut self, group: &OpaqueGroup, cursor: &Cursor) -> Result<Response> {
        _ = (group, cursor);
        Ok(Response::Default)
    }
}

/// Visit a [TokensRc] using the given [VisitTokens] implementor.
///
/// # Errors
/// Should any visit function of `v` error, said error will be forwarded.
///
/// # Panics
/// If the [FoldTokens] implementor returns `Response::Skip(0)`.
pub fn visit_tokens(v: &mut dyn VisitTokens, tokens: TokensRc) -> Result<()> {
    let mut context = Vec::new();
    context.push(Cursor::new(tokens));

    loop {
        let cursor = context.last_mut().unwrap_or_else(|| unreachable!());

        match cursor.first() {
            Some(token) => {
                let response = match token {
                    ::tokens_rc::TokenTree::Literal(literal) => v.visit_literal(literal, cursor),
                    ::tokens_rc::TokenTree::Ident(ident) => v.visit_ident(ident, cursor),
                    ::tokens_rc::TokenTree::Punct(punct) => v.visit_punct(punct, cursor),
                    ::tokens_rc::TokenTree::Group(opaque_group) => {
                        v.visit_group(opaque_group, cursor)
                    }
                }?;

                match response {
                    Response::Default => match token {
                        ::tokens_rc::TokenTree::Group(group) => {
                            let Group { stream, .. } = group.as_group();
                            let cursor = Cursor::new(stream.clone());
                            context.push(cursor);
                        }
                        _ => {
                            cursor.forward(1);
                        }
                    },
                    Response::Skip(value) => cursor.forward(
                        NonZero::new(value)
                            .unwrap_or_else(|| panic!("skip amount should not be 0"))
                            .get(),
                    ),
                }
            }
            None => {
                _ = context.pop();
                match context.last_mut() {
                    Some(cursor) => {
                        cursor.forward(1);
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}
