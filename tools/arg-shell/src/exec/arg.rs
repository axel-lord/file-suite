//! arg exec impl.

use ::std::io::Write;

use crate::ByteStr;

/// Executable state for an argument.
#[derive(Debug, Clone)]
pub enum Arg<'i> {
    /// Argument is a string value.
    String(&'i ByteStr),
}

impl<'i> Arg<'i> {
    /// Create from an ast node.
    pub fn from_ast(node: &mut crate::ast::arg::Arg<'i>) -> ::std::io::Result<Self> {
        match node {
            crate::ast::arg::Arg::String(s) => Ok(Self::String(s)),
            crate::ast::arg::Arg::FString(_fstring) => todo!(),
            crate::ast::arg::Arg::Group(_ast) => todo!(),
        }
    }
}

impl crate::exec::Arg for Arg<'_> {
    fn write_arg(self, buf: &mut impl Write) -> ::std::io::Result<usize> {
        match self {
            Arg::String(byte_str) => {
                buf.write_all(byte_str.as_bytes())?;
                Ok(byte_str.len())
            }
        }
    }
}
