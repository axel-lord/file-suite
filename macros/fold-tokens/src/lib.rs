//! Utilities to fold tokens.

pub use crate::{
    cursor::Cursor,
    fold_tokens::{FoldTokens, fold_tokens},
    visit_tokens::{VisitTokens, visit_tokens},
};

mod cursor;
mod fold_tokens;
mod visit_tokens;

/// Crate result type.
pub type Result<T = ()> = ::syn::Result<T>;

/// How to handle passed token.
#[derive(Debug, Clone, Copy, Default)]
pub enum Response {
    /// Continue as normal, pushin the token to the output stream
    /// and further folding it if it is a group.
    #[default]
    Default,
    /// Skip the given amount of tokens.
    /// Should probably be used with a count of 1 if
    /// something alternative was pushed to the token stream.
    Skip(usize),
}
