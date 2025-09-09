//! Execution of ast.

use ::std::io::Write;

use ::enum_dispatch::enum_dispatch;

use crate::ByteStr;

/// Result of trying to accept some bytes.
#[must_use]
#[derive(Debug)]
pub enum AcceptResult {
    /// Bytes were accepted.
    Ok,
    /// Next has to be called first.
    Full,
    /// Executor has finished.
    Fin,
    /// An io error occurred.
    IoError(::std::io::Error),
}

/// Result of trying to get some bytes.
#[must_use]
#[derive(Debug)]
pub enum NextResult {
    /// N Bytes were written.
    Ok(usize),
    /// Accept has to be called with some bytes first.
    /// Some data may exist.
    Incomplete,
    /// No more data will become available.
    Fin,
    /// An io error occurred.
    IoError(::std::io::Error),
}

/// Execution environment
#[derive(Debug)]
pub struct Env {}

/// Trait for executable ast nodes.
#[enum_dispatch]
pub trait Exec {
    /// Accept some bytes.
    fn accept(&mut self, env: &mut Env, row: &ByteStr) -> AcceptResult;

    /// Get some bytes.
    fn next(&mut self, env: &mut Env, row: &mut impl Write) -> NextResult;

    /// Close exector, may write some bytes if last result was incomplete.
    /// an implementor may always return incomplete.
    ///
    /// Returns count of written bytes.
    fn close(&mut self, env: &mut Env, row: &mut impl Write) -> ::std::io::Result<usize>;
}

/// Trait for types which may be converted to executables.
pub trait ToExec {
    /// Exec implementor.
    type Exec<'a>: 'a + Exec
    where
        Self: 'a;

    /// Get an exec implementor.
    fn to_exec<'a>(&'a self) -> ::std::io::Result<Self::Exec<'a>>;
}
