use ::std::{
    borrow::Borrow,
    fmt::{Debug, Display, Write},
    hash::Hash,
    ops::{Deref, DerefMut},
};

use ::derive_more::{AsMut, AsRef, Deref, DerefMut, From, Into};

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
    pub fn from_ref(b: &u8) -> &Self {
        ::bytemuck::TransparentWrapper::wrap_ref(b)
    }

    pub fn from_mut(b: &mut u8) -> &mut Self {
        ::bytemuck::TransparentWrapper::wrap_mut(b)
    }

    pub fn to_ref(&self) -> &u8 {
        ::bytemuck::TransparentWrapper::peel_ref(self)
    }

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

/// A byte string.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, ::bytemuck::TransparentWrapper)]
pub struct ByteStr([Byte]);

impl ByteStr {
    /// Construct a new byte string from a byte slice.
    pub fn new(bytes: &[u8]) -> &Self {
        ::bytemuck::TransparentWrapper::wrap_ref(::bytemuck::TransparentWrapper::wrap_slice(bytes))
    }

    /// Construct a new mutable byte string from a byte slice.
    pub fn new_mut(bytes: &mut [u8]) -> &Self {
        ::bytemuck::TransparentWrapper::wrap_mut(::bytemuck::TransparentWrapper::wrap_slice_mut(
            bytes,
        ))
    }

    /// Get self as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        ::bytemuck::TransparentWrapper::peel_slice(::bytemuck::TransparentWrapper::peel_ref(self))
    }

    /// Get self as a mutable byte slice.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        ::bytemuck::TransparentWrapper::peel_slice_mut(::bytemuck::TransparentWrapper::peel_mut(
            self,
        ))
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
    fn borrow(&self) -> &[u8] {
        todo!()
    }
}

impl Hash for ByteStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl<'i> From<&'i [u8]> for &'i ByteStr {
    fn from(value: &'i [u8]) -> Self {
        ByteStr::new(value)
    }
}

impl<'i> From<&'i ByteStr> for &'i [u8] {
    fn from(value: &'i ByteStr) -> Self {
        value.as_bytes()
    }
}

impl AsRef<[u8]> for ByteStr {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for ByteStr {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

impl Deref for ByteStr {
    type Target = [Byte];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ByteStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
