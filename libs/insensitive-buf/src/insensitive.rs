//! Utilities for case insensitive byte arrays.

use ::core::{
    char::{ToLowercase, ToUppercase},
    hash::Hash,
    iter::{Flatten, FusedIterator},
    marker::PhantomData,
    str::{Chars, Utf8Chunk, Utf8Chunks},
};

use crate::insensitive::private::Sealed;

mod private {
    //! Sealed trait module.

    /// Sealed trait.
    pub trait Sealed {}
}

/// Generic trait over cased mapping of chars.
pub trait CaseMap<'a>:
    'a + Iterator<Item = Self::Iter> + DoubleEndedIterator + FusedIterator + Sealed
{
    /// Cased iterator returned for every char.
    type Iter: Iterator<Item = char> + DoubleEndedIterator + FusedIterator;

    /// Create an instance of self from a [Chars] iterator.
    fn from_chars(chars: Chars<'a>) -> Self;
}

/// Nameable map for chaining [char::to_uppercase] from [str::chars].
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CharsUppercaseMap<'a>(Chars<'a>);

impl Iterator for CharsUppercaseMap<'_> {
    type Item = ToUppercase;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(char::to_uppercase)
    }
}

impl DoubleEndedIterator for CharsUppercaseMap<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(char::to_uppercase)
    }
}

impl FusedIterator for CharsUppercaseMap<'_> {}

impl Sealed for CharsUppercaseMap<'_> {}
impl<'a> CaseMap<'a> for CharsUppercaseMap<'a> {
    type Iter = ToUppercase;

    fn from_chars(chars: Chars<'a>) -> Self {
        Self(chars)
    }
}

/// Nameable map for chaining [char::to_lowercase] from [str::chars].
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CharsLowercaseMap<'a>(Chars<'a>);

impl Iterator for CharsLowercaseMap<'_> {
    type Item = ToLowercase;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(char::to_lowercase)
    }
}

impl DoubleEndedIterator for CharsLowercaseMap<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(char::to_lowercase)
    }
}

impl FusedIterator for CharsLowercaseMap<'_> {}

impl Sealed for CharsLowercaseMap<'_> {}
impl<'a> CaseMap<'a> for CharsLowercaseMap<'a> {
    type Iter = ToLowercase;

    fn from_chars(chars: Chars<'a>) -> Self {
        Self(chars)
    }
}

/// A [Utf8Chunk] like struct where the valid part is an uppercase iterator.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct CasedChunk<'a, M>(Utf8Chunk<'a>, PhantomData<fn() -> M>);

impl<'a, M> CasedChunk<'a, M>
where
    M: CaseMap<'a>,
{
    /// Iterator over valid chunk part as uppercase.
    pub fn valid(&self) -> Flatten<M> {
        M::from_chars(self.0.valid().chars()).flatten()
    }

    /// Get invalid [u8] slice.
    pub fn invalid(&self) -> &'a [u8] {
        self.0.invalid()
    }
}

impl<'a, M> PartialEq for CasedChunk<'a, M>
where
    M: CaseMap<'a>,
{
    fn eq(&self, other: &Self) -> bool {
        self.valid().eq(other.valid()) && self.invalid() == other.invalid()
    }
}
impl<'a, M> Eq for CasedChunk<'a, M> where M: CaseMap<'a> {}

impl<'a, M> PartialOrd for CasedChunk<'a, M>
where
    M: CaseMap<'a>,
{
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<'a, M> Ord for CasedChunk<'a, M>
where
    M: CaseMap<'a>,
{
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.valid()
            .cmp(other.valid())
            .then_with(|| self.invalid().cmp(other.invalid()))
    }
}
impl<'a, M> Hash for CasedChunk<'a, M>
where
    M: CaseMap<'a>,
{
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        for c in self.valid() {
            H::write_u32(state, c.into());
        }
        H::write(state, self.invalid());
    }
}

/// Iterator over uppercase [Utf8Chunk], similar to [Utf8Chunks].
#[derive(Debug)]
#[repr(transparent)]
pub struct CasedChunks<'a, M>(Utf8Chunks<'a>, PhantomData<fn() -> M>);

impl<'a, M> CasedChunks<'a, M>
where
    M: CaseMap<'a>,
{
    /// Create a new instance from a [u8] slice.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes.utf8_chunks(), PhantomData)
    }
}

impl<'a, M> Iterator for CasedChunks<'a, M>
where
    M: CaseMap<'a>,
{
    type Item = CasedChunk<'a, M>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|b| CasedChunk(b, PhantomData))
    }
}

impl<'a, M> FusedIterator for CasedChunks<'a, M> where M: CaseMap<'a> {}
