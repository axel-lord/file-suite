//! [Group] impl.

use ::proc_macro2::{Delimiter, Span, extra::DelimSpan};
use ::quote::{ToTokens, TokenStreamExt};

/// [Group][::proc_macro2::Group] replacement for [TokensRc].
#[derive(Debug, Clone)]
pub struct Group {
    /// Span of group.
    pub span: Span,
    /// Span of group delims.
    pub delim_span: DelimSpan,
    /// Group delimiter.
    pub delimiter: Delimiter,
    /// Tokens of group.
    pub stream: crate::TokensRc,
}

impl ToTokens for Group {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let stream = self.stream.to_token_stream();
        let mut group = ::proc_macro2::Group::new(self.delimiter, stream);
        group.set_span(self.span);
        tokens.append(group);
    }
}

impl From<::proc_macro2::Group> for Group {
    fn from(value: ::proc_macro2::Group) -> Self {
        Self {
            span: value.span(),
            delim_span: value.delim_span(),
            delimiter: value.delimiter(),
            stream: value.stream().into(),
        }
    }
}
