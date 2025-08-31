//! Filename implementation.

use ::core::{fmt::Debug, hash::Hash};

use bytemuck::TransparentWrapper;

use crate::{
    insensitive::{CaseMap, CasedChunks, CharsLowercaseMap, CharsUppercaseMap},
    insensitive_display::InsensitiveDisplay,
};

#[cfg(feature = "alloc")]
extern crate alloc;

/// Filename DST, wraps a u8 slice and does case insensitive, Ord, Eq and Hash.
#[repr(transparent)]
#[derive(TransparentWrapper)]
pub struct Insensitive([u8]);

impl Insensitive {
    /// Construct a new insinsensitive.
    pub fn new<S: AsRef<[u8]> + ?Sized>(s: &S) -> &Self {
        Self::wrap_ref(s.as_ref())
    }

    /// Construct a new insensitive from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        Self::wrap_ref(bytes)
    }

    /// Construct a new mutable insensitive from a byte slice.
    pub fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        Self::wrap_mut(bytes)
    }

    /// Get internal bytes as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        Self::peel_ref(self)
    }

    /// Get internal bytes as a slice.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        Self::peel_mut(self)
    }

    /// Get an object that can be used to print self.
    pub const fn display(&self) -> InsensitiveDisplay<'_> {
        InsensitiveDisplay(self)
    }

    /// Get byte count of self. Two equal insensitives may have different lengths.
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    /// Iterate over self as [CasedChunks].
    pub fn cased_chunks<'a, M: CaseMap<'a>>(&'a self) -> CasedChunks<'a, M> {
        CasedChunks::new(self.as_bytes())
    }

    /// Iterate over self as uppercased [CasedChunks].
    pub fn upper_chunks<'a>(&'a self) -> CasedChunks<'a, CharsUppercaseMap<'a>> {
        Self::cased_chunks::<'a, CharsUppercaseMap<'a>>(self)
    }

    /// Iterate over self as lowercased [CasedChunks].
    pub fn lower_chunks<'a>(&'a self) -> CasedChunks<'a, CharsLowercaseMap<'a>> {
        Self::cased_chunks::<'a, CharsLowercaseMap<'a>>(self)
    }

    #[cfg(feature = "alloc")]
    /// Encode self as case mapped.
    pub fn encode<'a, M: CaseMap<'a>>(&'a self, buf: &mut alloc::vec::Vec<u8>) {
        let mut ebuf = [0u8; 4];
        for chunk in self.cased_chunks::<'a, M>() {
            for c in chunk.valid() {
                buf.extend_from_slice(c.encode_utf8(&mut ebuf).as_bytes());
            }

            buf.extend_from_slice(chunk.invalid());
        }
    }

    #[cfg(feature = "alloc")]
    /// Encode self as lower case.
    pub fn encode_lower(&self, buf: &mut alloc::vec::Vec<u8>) {
        self.encode::<CharsLowercaseMap<'_>>(buf)
    }

    #[cfg(feature = "alloc")]
    /// Encode self as upper case.
    pub fn encode_upper(&self, buf: &mut alloc::vec::Vec<u8>) {
        self.encode::<CharsUppercaseMap<'_>>(buf)
    }

    #[cfg(all(feature = "std", unix))]
    /// Get self as a [Path][std::path::Path].
    pub fn as_path(&self) -> &::std::path::Path {
        self.as_ref()
    }

    #[cfg(all(feature = "std", unix))]
    /// Get self as a [OsStr][std::ffi::OsStr].
    pub fn as_os_str(&self) -> &::std::ffi::OsStr {
        self.as_ref()
    }
}

#[cfg(feature = "alloc")]
impl alloc::borrow::ToOwned for Insensitive {
    type Owned = crate::InsensitiveBuf;

    fn to_owned(&self) -> Self::Owned {
        crate::InsensitiveBuf::new(self)
    }
}

impl AsRef<[u8]> for Insensitive {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<Insensitive> for Insensitive {
    fn as_ref(&self) -> &Insensitive {
        self
    }
}

impl Debug for Insensitive {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_tuple("Filename").field(&self.display()).finish()
    }
}

impl Eq for Insensitive {}
impl PartialEq for Insensitive {
    fn eq(&self, other: &Self) -> bool {
        self.upper_chunks().eq(other.cased_chunks())
    }
}
impl Ord for Insensitive {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.upper_chunks().cmp(other.cased_chunks())
    }
}
impl PartialOrd for Insensitive {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Hash for Insensitive {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        for unit in self.upper_chunks() {
            unit.hash(state)
        }
    }
}

#[cfg(all(feature = "std", unix))]
impl AsRef<::std::ffi::OsStr> for Insensitive {
    fn as_ref(&self) -> &std::ffi::OsStr {
        use ::std::os::unix::ffi::OsStrExt;
        std::ffi::OsStr::from_bytes(self.as_bytes())
    }
}

#[cfg(all(feature = "std", unix))]
impl AsRef<::std::path::Path> for Insensitive {
    fn as_ref(&self) -> &std::path::Path {
        ::std::path::Path::new(self)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn filename_eq() {
        _ = env_logger::builder()
            .parse_default_env()
            .default_format()
            .is_test(true)
            .try_init();

        assert_eq!(Insensitive::new(""), Insensitive::new(""));
        assert_eq!(Insensitive::new("abc"), Insensitive::new("abc"));
        assert_eq!(Insensitive::new("abc"), Insensitive::new("Abc"));
        assert_eq!(Insensitive::new("åäö"), Insensitive::new("åäö"));
        assert_eq!(Insensitive::new("åäö"), Insensitive::new("ÅÄÖ"));

        assert_ne!(Insensitive::new(""), Insensitive::new("ABC"));
        assert_ne!(Insensitive::new("ABC"), Insensitive::new("ABCD"));
        assert_ne!(Insensitive::new("ÅÄÖ"), Insensitive::new("ABCD"));
    }
}
