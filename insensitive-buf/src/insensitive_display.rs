//! [InsensitiveDisplay] implementation.

use ::core::fmt::{self, Debug, Display};

use crate::Insensitive;

/// Display implementor for [Insensitive].
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct InsensitiveDisplay<'f>(pub &'f Insensitive);
impl Display for InsensitiveDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.0.as_bytes().utf8_chunks() {
            write!(f, "{}", chunk.valid())?;
            for c in chunk.invalid() {
                write!(f, "\\x'{:x}'", c)?;
            }
        }
        Ok(())
    }
}
impl Debug for InsensitiveDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}
