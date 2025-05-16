//! Lookahead helpers for tokenstream parsing.'
#![allow(dead_code)]

use ::proc_macro2::{Punct, Spacing, TokenTree};
use ::quote::{ToTokens, TokenStreamExt};
use proc_macro2::TokenStream;

use crate::util::tcmp::TokenEq;

/// Lookahead iterator for tokens.
#[derive(Debug, Clone)]
pub struct TokenLookahead<I: IntoIterator<Item = TokenTree>, const COUNT: usize = 3> {
    /// Lookahead storage.
    st: LookaheadStorage<COUNT>,
    /// Iterator.
    it: I::IntoIter,
}

impl<I: IntoIterator<Item = TokenTree>, const COUNT: usize> TokenLookahead<I, COUNT> {
    /// Get a new lookahead iterator.
    pub fn new(it: I) -> Self {
        Self {
            st: LookaheadStorage::default(),
            it: it.into_iter(),
        }
    }

    /// Peek at a value.
    pub fn peek<const N: usize>(&mut self) -> Option<&mut TokenTree> {
        const {
            if N >= COUNT {
                panic!("N value shoult not be higher than or equal to COUNT")
            }
        }
        let Self { st, it } = self;

        while N >= st.len {
            st.pull(it)?;
        }

        Some(&mut st.buf[N])
    }

    /// Implementation of matches functions, has no compile time sanity checks.
    #[inline]
    fn matches_impl<'a>(&mut self, seq: impl IntoIterator<Item = &'a dyn TokenEq>) -> bool {
        let Self { st, it } = self;
        for (i, seq) in seq.into_iter().enumerate() {
            while i >= st.len() {
                if st.pull(it).is_none() {
                    return false;
                }
            }
            if !seq.token_cmp(&st.buf[i]) {
                return false;
            }
        }

        true
    }

    /// Check if a sequence of [TokenEq] matches future results.
    pub fn matches<const N: usize>(&mut self, seq: [&dyn TokenEq; N]) -> bool {
        const {
            if N > COUNT {
                panic!("match sequence should not be longer than COUNT")
            }
        }
        self.matches_impl(seq)
    }

    /// Check if a sequence of [TokenEq] matches value and future results.
    pub fn matches_after<const N: usize, V>(&mut self, value: V, seq: [&dyn TokenEq; N]) -> bool
    where
        TokenTree: From<V>,
    {
        const {
            if N == 0 {
                panic!("match sequence should not be empty")
            }
            if N > (COUNT + 1) {
                panic!("match sequence should not be longer than COUNT + 1")
            }
        }
        let mut seq = seq.into_iter();
        if !seq
            .next()
            .is_some_and(|seq| seq.token_cmp(&TokenTree::from(value)))
        {
            return false;
        }

        self.matches_impl(seq)
    }

    /// Discard all peeked at values.
    pub fn discard(&mut self) -> <LookaheadStorage<COUNT> as IntoIterator>::IntoIter {
        self.st.take()
    }
}

impl<I: IntoIterator<Item = TokenTree>, const COUNT: usize> Iterator for TokenLookahead<I, COUNT> {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        if self.st.is_empty() {
            self.it.next()
        } else {
            self.st.advance()
        }
    }
}

/// Lookahead.
#[derive(Debug, Clone)]
pub struct LookaheadStorage<const COUNT: usize = 3> {
    /// Token storage.
    buf: [TokenTree; COUNT],
    /// Current length.
    len: usize,
}

impl<const COUNT: usize> Default for LookaheadStorage<COUNT> {
    fn default() -> Self {
        Self {
            buf: ::std::array::from_fn(|_| TokenTree::Punct(Punct::new('.', Spacing::Alone))),
            len: 0,
        }
    }
}

impl<const COUNT: usize> LookaheadStorage<COUNT> {
    /// Set size to 0.
    pub const fn clear(&mut self) {
        self.len = 0;
    }

    /// Get amount of stored tokens.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if storage is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get iterator of stored tokens.
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    /// Get iterator of stored tokens as mutable references.
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    /// Push a token.
    ///
    /// # Panics
    /// If the storage is full.
    pub fn push(&mut self, token: TokenTree) {
        if self.len >= COUNT {
            panic!("max capacity of LookaheadStorage reached")
        }
        self.buf[self.len] = token;
        self.len += 1;
    }

    /// Take all stored tokens.
    pub fn take(&mut self) -> <Self as IntoIterator>::IntoIter {
        ::std::mem::take(self).into_iter()
    }

    /// Pull a value from a [TokenTree] [Iterator] into storage and return a mutable reference to
    /// it, if it exists.
    ///
    /// # Panics
    /// If used when buffer is full.
    pub fn pull<'s>(
        &'s mut self,
        iter: &mut dyn Iterator<Item = TokenTree>,
    ) -> Option<&'s mut TokenTree> {
        if self.len == COUNT {
            panic!("LookaheadStorage cannot pull more items as inner buffer is full")
        }
        let idx = self.len;
        self.push(iter.next()?);
        Some(&mut self.buf[idx])
    }

    /// Remove one token from the front.
    pub fn advance(&mut self) -> Option<TokenTree> {
        if self.len == 0 {
            return None;
        };
        let mut out = TokenTree::Punct(Punct::new('.', Spacing::Alone));
        ::std::mem::swap(&mut self.buf[0], &mut out);
        self.buf[..self.len].rotate_left(1);
        self.len -= 1;
        Some(out)
    }
}

impl<const COUNT: usize> IntoIterator for LookaheadStorage<COUNT> {
    type Item = TokenTree;
    type IntoIter = ::std::iter::Take<::std::array::IntoIter<TokenTree, COUNT>>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { buf, len } = self;
        buf.into_iter().take(len)
    }
}
impl<'a, const COUNT: usize> IntoIterator for &'a LookaheadStorage<COUNT> {
    type Item = &'a TokenTree;
    type IntoIter = ::std::iter::Take<::std::slice::Iter<'a, TokenTree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter().take(self.len)
    }
}
impl<'a, const COUNT: usize> IntoIterator for &'a mut LookaheadStorage<COUNT> {
    type Item = &'a mut TokenTree;
    type IntoIter = ::std::iter::Take<::std::slice::IterMut<'a, TokenTree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut().take(self.len)
    }
}
impl<const COUNT: usize> ToTokens for LookaheadStorage<COUNT> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for token in self.iter().cloned() {
            tokens.append(token);
        }
    }
    fn into_token_stream(self) -> TokenStream
    where
        Self: Sized,
    {
        TokenStream::from_iter(self)
    }
}
