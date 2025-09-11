//! Command argument ast types.

use ::std::sync::OnceLock;

use crate::{ByteStr, alias::ByteParser, ast::Ast, withspan::WithSpan};

/// Arguments for calls.
#[derive(Debug, Clone)]
pub enum Arg<'i> {
    /// Pass string as is.
    String(WithSpan<&'i ByteStr>),
    /// A format string.
    FString(FString<'i>),
    /// Group as an argument.
    Group(Ast<'i>),
}

/// An fstring argument.
#[derive(Debug, Clone)]
pub struct FString<'i> {
    content: WithSpan<&'i ByteStr>,
    cache: OnceLock<Result<crate::exec::fstring::FString<'i>, Vec<::chumsky::error::Rich<'i, u8>>>>,
}

impl<'i> FString<'i> {
    /// Create a new instance from content.
    #[inline]
    pub const fn new(content: WithSpan<&'i ByteStr>) -> Self {
        Self {
            content,
            cache: OnceLock::new(),
        }
    }

    /// Get parsed fstring.
    pub fn parsed(
        &self,
        parser: &impl ByteParser<'i, crate::exec::fstring::FString<'i>>,
    ) -> Result<&'_ crate::exec::fstring::FString<'i>, &'_ [::chumsky::error::Rich<'i, u8>]> {
        self.cache
            .get_or_init(|| parser.parse(self.content.as_bytes()).into_result())
            .as_ref()
            .map_err(|err| err.as_slice())
    }
}
