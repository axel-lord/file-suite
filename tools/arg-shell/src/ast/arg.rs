//! Command argument ast types.

use crate::{ByteStr, ast::Ast, withspan::WithSpan};

/// Arguments for calls.
#[derive(Debug, Clone)]
pub enum Arg<'i> {
    /// Pass string as is.
    String(WithSpan<&'i ByteStr>),
    /// A format string.
    FString(WithSpan<&'i ByteStr>),
    /// Group as an argument.
    Group(Ast<'i>),
}
