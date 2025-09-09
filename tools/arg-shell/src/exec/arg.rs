//! arg exec impl.

use ::std::io::Write;

use crate::{
    ByteStr,
    exec::{Env, Exec},
};

/// Executable state for an argument.
#[derive(Debug, Clone)]
pub enum Arg<'i> {
    /// Argument is a string value.
    String(&'i ByteStr),
}

impl Exec for Arg<'_> {
    fn close(&mut self, _env: &mut Env, row: &mut impl Write) -> ::std::io::Result<usize> {
        match self {
            Arg::String(byte_str) => {
                row.write_all(byte_str.as_bytes())?;
                Ok(byte_str.len())
            }
        }
    }
}
