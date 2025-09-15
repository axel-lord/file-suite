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

