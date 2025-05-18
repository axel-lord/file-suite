#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

use ::core::cmp::Ordering;

pub use self::{insensitive_display::InsensitiveDisplay, insensitive_ref::Insensitive};

mod insensitive_ref;

#[cfg(feature = "alloc")]
mod insensitive_buf;

#[cfg(feature = "alloc")]
pub use self::insensitive_buf::InsensitiveBuf;

mod insensitive_display;

#[cfg(feature = "alloc")]
mod encode {
    //! Encoding utilities.

    use alloc::vec::Vec;

    use crate::Insensitive;
    extern crate alloc;

    /// Encode byte slice as upper case, invalid utf-8 will be encoded as-is.
    pub fn encode_upper(bytes: &[u8], buf: &mut Vec<u8>) {
        Insensitive::new(bytes).encode_upper(buf)
    }

    /// Encode byte slice as lower case, invalid utf-8 will be encoded as-is.
    pub fn encode_lower(bytes: &[u8], buf: &mut Vec<u8>) {
        Insensitive::new(bytes).encode_lower(buf)
    }

    /// Create a vec of [u8] where the valid utf-segments are uppercase.
    pub fn to_upper(bytes: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        encode_upper(bytes, &mut buf);
        buf
    }

    /// Create a vec of [u8] where the valid utf-segments are lowrcase.
    pub fn to_lower(bytes: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        encode_lower(bytes, &mut buf);
        buf
    }
}

#[cfg(feature = "alloc")]
pub use encode::{encode_lower, encode_upper, to_lower, to_upper};

pub mod insensitive;

/// Case insensitive compare two values.
pub fn insensitive_cmp<S1, S2>(s1: &S1, s2: &S2) -> Ordering
where
    S1: AsRef<[u8]> + ?Sized,
    S2: AsRef<[u8]> + ?Sized,
{
    insensitive_cmp_bytes(s1.as_ref(), s2.as_ref())
}

/// Case insensitive compare two byte slices.
pub fn insensitive_cmp_bytes(s1: &[u8], s2: &[u8]) -> Ordering {
    Insensitive::from_bytes(s1).cmp(Insensitive::from_bytes(s2))
}
