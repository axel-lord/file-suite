use ::chumsky::span::SimpleSpan;
use ::derive_more::{AsMut, AsRef, Deref, DerefMut};

/// A value with a span.
#[derive(Debug, Clone, Copy, Deref, DerefMut, AsRef, AsMut, PartialEq, Eq, Hash)]
pub struct WithSpan<T> {
    #[deref]
    #[deref_mut]
    #[as_ref]
    #[as_mut]
    pub value: T,
    pub span: SimpleSpan,
}

impl<T> PartialOrd for WithSpan<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for WithSpan<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> From<(T, SimpleSpan)> for WithSpan<T> {
    fn from((t, span): (T, SimpleSpan)) -> Self {
        Self { value: t, span }
    }
}
