#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

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
