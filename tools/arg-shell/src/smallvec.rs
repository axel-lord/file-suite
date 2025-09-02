use ::chumsky::container::Container;
use ::derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator};

/// Wrapper of [SmallVec][::smallvec::SmallVec] implementing
/// [Container][::chumsky::container::Container].
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
    Deref,
    DerefMut,
    AsRef,
    AsMut,
    IntoIterator,
    Index,
    IndexMut,
)]
#[into_iterator(owned, ref, ref_mut)]
#[from(::smallvec::SmallVec<[T; INLINE]>, Vec<T>)]
#[into(::smallvec::SmallVec<[T; INLINE]>)]
pub struct SmallVec<const INLINE: usize, T>(pub ::smallvec::SmallVec<[T; INLINE]>);

impl<const INLINE: usize, T> SmallVec<INLINE, T> {
    /// Create a new empty smallvec.
    #[inline]
    pub const fn new() -> Self {
        Self(::smallvec::SmallVec::new_const())
    }
}

impl<const INLINE: usize, T> Container<T> for SmallVec<INLINE, T> {
    #[inline]
    fn push(&mut self, item: T) {
        self.0.push(item);
    }

    #[inline]
    fn with_capacity(n: usize) -> Self {
        Self(::smallvec::SmallVec::with_capacity(n))
    }
}

impl<const INLINE: usize, T> Default for SmallVec<INLINE, T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<const INLINE: usize, T> From<SmallVec<INLINE, T>> for Vec<T> {
    #[inline]
    fn from(value: SmallVec<INLINE, T>) -> Self {
        value.0.into_vec()
    }
}

impl<const INLINE: usize, T> AsMut<[T]> for SmallVec<INLINE, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<const INLINE: usize, T> AsRef<[T]> for SmallVec<INLINE, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}
