//! Types close to never.

use ::std::{
    fmt::Debug,
    hash::{self, Hash},
    marker::PhantomData,
};
use std::cmp::Ordering;

/// Enum without any variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoVariant {}

impl NoVariant {
    /// Unwrap into never type.
    pub const fn unwrap(self) -> ! {
        match self {}
    }
}

/// PhantomData which is not allowed to exist.
pub struct NoPhantomData<T> {
    /// [NoVariant] field.
    nv: NoVariant,

    /// PhantomData field.
    _p: PhantomData<T>,
}

impl<T> NoPhantomData<T> {
    /// Unwrap into a never type.
    pub const fn unwrap(self) -> ! {
        match self.nv {}
    }
}

impl<T> Hash for NoPhantomData<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.nv.hash(state);
    }
}

impl<T> PartialOrd for NoPhantomData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for NoPhantomData<T> {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<T> Eq for NoPhantomData<T> {}

impl<T> PartialEq for NoPhantomData<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Copy for NoPhantomData<T> {}

impl<T> Clone for NoPhantomData<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Debug for NoPhantomData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoPhantomData")
            .field("nv", &self.nv)
            .field("_p", &self._p)
            .finish()
    }
}
