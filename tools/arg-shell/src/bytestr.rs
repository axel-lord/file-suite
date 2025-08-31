use ::std::{
    borrow::Borrow,
    fmt::{Debug, Display, Write},
    hash::Hash,
    ops::{Deref, DerefMut},
};

/// A byte string.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, ::bytemuck::TransparentWrapper)]
pub struct ByteStr([u8]);

impl ByteStr {
    /// Construct a new byte string from a byte slice.
    pub fn new(bytes: &[u8]) -> &Self {
        ::bytemuck::TransparentWrapper::wrap_ref(bytes)
    }

    /// Construct a new mutable byte string from a byte slice.
    pub fn new_mut(bytes: &mut [u8]) -> &Self {
        ::bytemuck::TransparentWrapper::wrap_mut(bytes)
    }

    /// Get self as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        ::bytemuck::TransparentWrapper::peel_ref(self)
    }

    /// Get self as a mutable byte slice.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        ::bytemuck::TransparentWrapper::peel_mut(self)
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
            for chr in chunk.valid().chars() {
                write!(f, "{}", chr.escape_debug())?;
            }

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
        self.0.hash(state);
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
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl DerefMut for ByteStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_bytes_mut()
    }
}
