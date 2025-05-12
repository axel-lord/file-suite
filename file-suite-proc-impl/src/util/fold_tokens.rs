//! Fold trait for TokenStreams.

use ::proc_macro2::{Group, Literal, Punct, TokenStream};
use ::quote::TokenStreamExt;
use ::syn::Ident;
use proc_macro2::TokenTree;

use crate::util::TokenLookahead;

/// Fold a tokenstream.
pub trait FoldTokens<const LOOKAHEAD: usize>
where
    Self: Sized,
{
    /// Handle a punctuation characters.
    ///
    /// # Errors
    /// If the implementator needs to.
    fn fold_punct(
        &mut self,
        punct: Punct,
        lookahead: &mut TokenLookahead<TokenStream, LOOKAHEAD>,
        tokens: &mut TokenStream,
    ) -> ::syn::Result<()> {
        _ = lookahead;
        fold_punct(punct, tokens);
        Ok(())
    }

    /// Handle literals.
    ///
    /// # Errors
    /// If the implementator needs to.
    fn fold_literal(
        &mut self,
        literal: Literal,
        lookahead: &mut TokenLookahead<TokenStream, LOOKAHEAD>,
        tokens: &mut TokenStream,
    ) -> ::syn::Result<()> {
        _ = lookahead;
        fold_literal(literal, tokens);
        Ok(())
    }

    /// Handle Idents.
    ///
    /// # Errors
    /// If the implementator needs to.
    fn fold_ident(
        &mut self,
        ident: Ident,
        lookahead: &mut TokenLookahead<TokenStream, LOOKAHEAD>,
        tokens: &mut TokenStream,
    ) -> ::syn::Result<()> {
        _ = lookahead;
        fold_ident(ident, tokens);
        Ok(())
    }

    /// Handle groups.
    ///
    /// # Errors
    /// If the implementator needs to.
    fn fold_group(
        &mut self,
        group: Group,
        lookahead: &mut TokenLookahead<TokenStream, LOOKAHEAD>,
        tokens: &mut TokenStream,
    ) -> ::syn::Result<()> {
        _ = lookahead;
        fold_group(self, group, tokens)
    }
}

/// Fold a tokenstream.
///
/// # Errors
/// If the [FoldTokens] Errors.
pub fn fold_token_stream<F: FoldTokens<LOOKAHEAD>, const LOOKAHEAD: usize>(
    f: &mut F,
    tokens: TokenStream,
) -> ::syn::Result<TokenStream> {
    let mut lookahead = TokenLookahead::<_, LOOKAHEAD>::new(tokens);
    let mut tokens = TokenStream::default();

    while let Some(token) = lookahead.next() {
        let lookahead = &mut lookahead;
        let tokens = &mut tokens;
        match token {
            TokenTree::Group(group) => f.fold_group(group, lookahead, tokens),
            TokenTree::Ident(ident) => f.fold_ident(ident, lookahead, tokens),
            TokenTree::Punct(punct) => f.fold_punct(punct, lookahead, tokens),
            TokenTree::Literal(literal) => f.fold_literal(literal, lookahead, tokens),
        }?;
    }

    Ok(tokens)
}

/// Default fold method for a punctuation character.
pub fn fold_punct(punct: Punct, tokens: &mut TokenStream) {
    tokens.append(punct);
}

/// Default fold method for Literals.
pub fn fold_literal(lit: Literal, tokens: &mut TokenStream) {
    tokens.append(lit);
}

/// Default fold method for Literals.
pub fn fold_ident(ident: Ident, tokens: &mut TokenStream) {
    tokens.append(ident);
}

/// Default fold method for groups.
///
/// # Errors
/// If further fold errors.
pub fn fold_group<F: FoldTokens<LOOKAHEAD>, const LOOKAHEAD: usize>(
    f: &mut F,
    group: Group,
    tokens: &mut TokenStream,
) -> ::syn::Result<()> {
    let span = group.span();
    let delim = group.delimiter();
    let stream = fold_token_stream(f, group.stream())?;
    let mut group = Group::new(delim, stream);
    group.set_span(span);

    tokens.append(group);
    Ok(())
}
