//! Execution of ast.

use ::std::io::Write;

use crate::ByteStr;

pub mod arg;
pub mod fstring;
pub mod ast {
    //! Execute asts.

    use ::std::marker::PhantomData;

    use crate::exec::Exec;

    /// [Exec] implementor for ast.
    #[derive(Debug)]
    pub struct Ast<'ast> {
        _p: PhantomData<&'ast ()>,
    }

    impl<'ast> Ast<'ast> {
        /// Create an ast exec from some data.
        pub fn from_ast(ast: &mut crate::ast::Ast<'ast>) -> Self {
            Self { _p: PhantomData }
        }
    }

    impl Exec for Ast<'_> {}
}
pub mod cmdline {
    //! Execute single command line calls.

    use ::std::marker::PhantomData;

    use crate::exec::Exec;

    /// [Exec] implementor for cmdline.
    #[derive(Debug)]
    pub struct Cmdline<'cmdline> {
        _p: PhantomData<&'cmdline ()>,
    }

    impl<'cmdline> Exec for Cmdline<'cmdline> {}

    impl<'cmdline> Cmdline<'cmdline> {
        /// Create from an ast node.
        pub fn from_ast(node: &mut crate::ast::Cmdline<'cmdline>) -> Self {
            todo!()
        }
    }
}
pub mod call {}

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
pub trait Exec {
    /// Accept some bytes.
    fn accept(&mut self, env: &mut Env, row: &ByteStr) -> AcceptResult {
        _ = (env, row);
        AcceptResult::Fin
    }

    /// Get some bytes.
    fn next(&mut self, env: &mut Env, row: &mut impl Write) -> NextResult {
        _ = (env, row);
        NextResult::Fin
    }

    /// Close exector, may write some bytes if last result was incomplete.
    /// an implementor may always return incomplete.
    ///
    /// Returns count of written bytes.
    fn close(&mut self, env: &mut Env, row: &mut impl Write) -> ::std::io::Result<usize> {
        _ = (env, row);
        Ok(0)
    }
}
