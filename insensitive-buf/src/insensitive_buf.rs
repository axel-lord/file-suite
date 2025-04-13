//! Implementation of [FilenameBuf].

extern crate alloc;

use ::core::{
    borrow::Borrow,
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use ::tinyvec::TinyVec;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Insensitive;

/// Internal buffer size of [InsensitiveBuf] after which it allocates.
const BUFSIZE: usize = size_of::<usize>() * 3;

/// Owned [Insensitive], stores small names internally larger ones need to be allocated.
#[repr(transparent)]
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(from = "alloc::vec::Vec<u8>", into = "alloc::vec::Vec<u8>")
)]
pub struct InsensitiveBuf(TinyVec<[u8; BUFSIZE]>);

impl InsensitiveBuf {
    /// Construct a new [InsensitiveBuf].
    pub fn new<S: AsRef<[u8]> + ?Sized>(s: &S) -> Self {
        let mut arr = TinyVec::new();
        arr.extend_from_slice(s.as_ref());
        Self(arr)
    }

    /// Extend with the contents of a &[u8] slice.
    pub fn extend_from_slice<S: AsRef<[u8]> + ?Sized>(&mut self, s: &S) -> &mut Self {
        self.0.extend_from_slice(s.as_ref());
        self
    }

    /// Extend with the reversed contents of a [u8] slice.
    pub fn extend_from_slice_reversed<S: AsRef<[u8]> + ?Sized>(&mut self, s: &S) -> &mut Self {
        let start = self.0.len();
        self.0.extend_from_slice(s.as_ref());
        let end = self.0.len();
        self.0[start..end].reverse();
        self
    }

    /// Get self as a [Insensitive] reference.
    pub fn as_insensitive(&self) -> &Insensitive {
        Insensitive::from_bytes(&self.0)
    }

    /// Get self as a mutable [Insensitive] reference.
    pub fn as_insensitive_mut(&mut self) -> &mut Insensitive {
        Insensitive::from_bytes_mut(&mut self.0)
    }
}

impl From<&str> for InsensitiveBuf {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<&[u8]> for InsensitiveBuf {
    fn from(value: &[u8]) -> Self {
        Self::new(value)
    }
}

impl From<alloc::vec::Vec<u8>> for InsensitiveBuf {
    fn from(value: alloc::vec::Vec<u8>) -> Self {
        Self::new(&value)
    }
}

impl From<InsensitiveBuf> for alloc::vec::Vec<u8> {
    fn from(value: InsensitiveBuf) -> Self {
        value.as_bytes().to_vec()
    }
}

impl Debug for InsensitiveBuf {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_tuple("FilenameBuf").field(&self.display()).finish()
    }
}

impl Clone for InsensitiveBuf {
    fn clone(&self) -> Self {
        Self::new(self.as_insensitive())
    }
}

impl PartialEq for InsensitiveBuf {
    fn eq(&self, other: &Self) -> bool {
        self.as_insensitive().eq(other.as_insensitive())
    }
}
impl Eq for InsensitiveBuf {}
impl Ord for InsensitiveBuf {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.as_insensitive().cmp(other.as_insensitive())
    }
}
impl PartialOrd for InsensitiveBuf {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for InsensitiveBuf {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        self.as_insensitive().hash(state)
    }
}

impl Deref for InsensitiveBuf {
    type Target = Insensitive;

    fn deref(&self) -> &Self::Target {
        self.as_insensitive()
    }
}

impl DerefMut for InsensitiveBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_insensitive_mut()
    }
}

impl Borrow<Insensitive> for InsensitiveBuf {
    fn borrow(&self) -> &Insensitive {
        self
    }
}

impl AsRef<[u8]> for InsensitiveBuf {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<Insensitive> for InsensitiveBuf {
    fn as_ref(&self) -> &Insensitive {
        self
    }
}

#[cfg(all(feature = "std", unix))]
impl AsRef<::std::ffi::OsStr> for InsensitiveBuf {
    fn as_ref(&self) -> &std::ffi::OsStr {
        use ::std::os::unix::ffi::OsStrExt;
        ::std::ffi::OsStr::from_bytes(self.as_bytes())
    }
}

#[cfg(all(feature = "std", unix))]
impl AsRef<::std::path::Path> for InsensitiveBuf {
    fn as_ref(&self) -> &std::path::Path {
        ::std::path::Path::new(self)
    }
}

impl<'any> Extend<&'any u8> for InsensitiveBuf {
    fn extend<T: IntoIterator<Item = &'any u8>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().copied())
    }
}

impl Extend<u8> for InsensitiveBuf {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl FromIterator<u8> for InsensitiveBuf {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(TinyVec::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_eq() {
        _ = env_logger::builder()
            .parse_default_env()
            .default_format()
            .is_test(true)
            .try_init();

        let inv = &[0xfe];

        let mut l = alloc::vec::Vec::from("ÅäÖ".as_bytes());
        l.extend(inv);
        l.extend("A".as_bytes());
        let l = Insensitive::new(&l);

        let mut r = alloc::vec::Vec::from("åäö".as_bytes());
        r.extend(inv);
        r.extend("a".as_bytes());
        let r = Insensitive::new(&r);

        assert_eq!(l, r);

        assert_eq!(InsensitiveBuf::new(l), InsensitiveBuf::new(r));
        assert_eq!(size_of::<InsensitiveBuf>(), size_of::<[usize; 4]>())
    }
}
