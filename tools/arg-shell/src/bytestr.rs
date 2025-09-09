//! Printable u8 wrappers.

use ::std::{
    borrow::{Borrow, BorrowMut},
    fmt::{Debug, Display, Write},
    hash::Hash,
    ops::{Deref, DerefMut},
};

use ::derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into};

/// A single printable byte.
#[repr(transparent)]
#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    From,
    Into,
    AsRef,
    AsMut,
    Deref,
    DerefMut,
    ::bytemuck::TransparentWrapper,
)]
pub struct Byte(pub u8);

impl Byte {
    /// Convert a u8 reference to a byte reference.
    #[inline]
    pub fn from_ref(b: &u8) -> &Self {
        ::bytemuck::TransparentWrapper::wrap_ref(b)
    }

    /// Convert a u8 reference to a byte reference.
    #[inline]
    pub fn from_mut(b: &mut u8) -> &mut Self {
        ::bytemuck::TransparentWrapper::wrap_mut(b)
    }

    /// Convert a byte reference to a u8 reference.
    #[inline]
    pub fn to_ref(&self) -> &u8 {
        ::bytemuck::TransparentWrapper::peel_ref(self)
    }

    /// Convert a byte reference to a u8 reference.
    #[inline]
    pub fn to_mut(&mut self) -> &mut u8 {
        ::bytemuck::TransparentWrapper::peel_mut(self)
    }
}

impl Display for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match str::from_utf8(&[self.0]) {
            Ok(s) => f.write_str(s),
            Err(..) => write!(f, "\\x{:02X}", self.0),
        }
    }
}

impl Debug for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("b'")?;
        match str::from_utf8(&[self.0]) {
            Ok(s) => Display::fmt(&s.escape_debug(), f),
            Err(..) => write!(f, "\\x{:02X}", self.0),
        }?;
        f.write_str("'")
    }
}

/// A byte string reference.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, ::bytemuck::TransparentWrapper)]
pub struct ByteStr([Byte]);

impl ByteStr {
    /// Construct a new byte string from a byte slice.
    #[inline]
    pub fn new(bytes: &[u8]) -> &Self {
        ::bytemuck::TransparentWrapper::wrap_ref(::bytemuck::TransparentWrapper::wrap_slice(bytes))
    }

    /// Construct a new mutable byte string from a byte slice.
    #[inline]
    pub fn new_mut(bytes: &mut [u8]) -> &mut Self {
        ::bytemuck::TransparentWrapper::wrap_mut(::bytemuck::TransparentWrapper::wrap_slice_mut(
            bytes,
        ))
    }

    /// Get self as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        ::bytemuck::TransparentWrapper::peel_slice(::bytemuck::TransparentWrapper::peel_ref(self))
    }

    /// Get self as a mutable byte slice.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        ::bytemuck::TransparentWrapper::peel_slice_mut(::bytemuck::TransparentWrapper::peel_mut(
            self,
        ))
    }
}

impl ToOwned for ByteStr {
    type Owned = ByteString;

    fn to_owned(&self) -> Self::Owned {
        ByteString::from(self)
    }
}

impl Display for ByteStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for chunk in self.as_bytes().utf8_chunks() {
            f.write_str(chunk.valid())?;
            for b in chunk.invalid() {
                write!(f, "\\x{b:02X}")?;
            }
        }
        Ok(())
    }
}

impl Debug for ByteStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("b\"")?;
        for chunk in self.as_bytes().utf8_chunks() {
            Display::fmt(&chunk.valid().escape_debug(), f)?;

            for b in chunk.invalid() {
                write!(f, "\\x{b:02X}")?;
            }
        }
        f.write_char('"')
    }
}

impl Borrow<[u8]> for ByteStr {
    #[inline]
    fn borrow(&self) -> &[u8] {
        todo!()
    }
}

impl Hash for ByteStr {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl<'i> From<&'i [u8]> for &'i ByteStr {
    #[inline]
    fn from(value: &'i [u8]) -> Self {
        ByteStr::new(value)
    }
}

impl<'i> From<&'i ByteStr> for &'i [u8] {
    #[inline]
    fn from(value: &'i ByteStr) -> Self {
        value.as_bytes()
    }
}

impl AsRef<[u8]> for ByteStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for ByteStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

/// A printable, growable, dynamic array of bytes.
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Index, IndexMut, From, Into)]
pub struct ByteString(Vec<Byte>);

impl ByteString {
    /// Construct a new empty byte string.
    #[inline]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Get bytes as a byte str.
    #[inline]
    pub fn as_byte_str(&self) -> &ByteStr {
        ByteStr::new(::bytemuck::TransparentWrapper::peel_slice(
            self.0.as_slice(),
        ))
    }

    /// Get bytes as a mutable byte str.
    #[inline]
    pub fn as_byte_str_mut(&mut self) -> &mut ByteStr {
        ByteStr::new_mut(::bytemuck::TransparentWrapper::peel_slice_mut(
            self.0.as_mut_slice(),
        ))
    }
}

impl Borrow<ByteStr> for ByteString {
    #[inline]
    fn borrow(&self) -> &ByteStr {
        self.as_byte_str()
    }
}

impl BorrowMut<ByteStr> for ByteString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut ByteStr {
        self.as_byte_str_mut()
    }
}

impl Borrow<[u8]> for ByteString {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.as_bytes()
    }
}
impl BorrowMut<[u8]> for ByteString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

impl Hash for ByteString {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_byte_str().hash(state);
    }
}

impl Deref for ByteString {
    type Target = ByteStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_byte_str()
    }
}

impl DerefMut for ByteString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_byte_str_mut()
    }
}

impl Debug for ByteString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_byte_str(), f)
    }
}

impl Display for ByteString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_byte_str(), f)
    }
}

impl AsRef<ByteStr> for ByteString {
    #[inline]
    fn as_ref(&self) -> &ByteStr {
        self
    }
}

impl AsMut<ByteStr> for ByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut ByteStr {
        self
    }
}

impl AsRef<[Byte]> for ByteString {
    #[inline]
    fn as_ref(&self) -> &[Byte] {
        &self.0
    }
}

impl AsMut<[Byte]> for ByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut [Byte] {
        &mut self.0
    }
}

impl AsRef<[u8]> for ByteString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for ByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

impl From<&ByteStr> for ByteString {
    #[inline]
    fn from(value: &ByteStr) -> Self {
        Self(<Byte as ::bytemuck::TransparentWrapper<u8>>::wrap_slice(value.as_bytes()).into())
    }
}

impl From<&[Byte]> for ByteString {
    #[inline]
    fn from(value: &[Byte]) -> Self {
        Self(value.into())
    }
}

impl From<&[u8]> for ByteString {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self(<Byte as ::bytemuck::TransparentWrapper<u8>>::wrap_slice(value).into())
    }
}
