//! [ChainArgs] impl.

use ::quote::ToTokens;
use ::syn::{Token, parse::Parse, punctuated::Punctuated};
use proc_macro2::TokenStream;
use syn::parse::ParseStream;

use crate::array_expr::{
    ArrayExpr, Node,
    function::{Call, DefaultArgs, ToCallable},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor for [ChainArgs].
#[derive(Debug, Clone)]
pub struct ChainCallable {
    /// Array expressions.
    exprs: Vec<ArrayExpr>,
}

impl Call for ChainCallable {
    fn call(&self, mut array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        for expr in &self.exprs {
            array.extend(storage.with_local_layer(|storage| expr.compute_with_storage(storage))?)
        }
        Ok(array)
    }
}

impl DefaultArgs for ChainCallable {
    fn default_args() -> Self {
        Self { exprs: Vec::new() }
    }
}

impl ToCallable for ChainArgs {
    type Call = ChainCallable;

    fn to_callable(&self) -> Self::Call {
        ChainCallable {
            exprs: self.exprs.iter().map(Node::to_array_expr).collect(),
        }
    }
}

/// Array expressions to chain after current.
#[derive(Debug, Clone)]
pub struct ChainArgs {
    /// Array expressions to chain.
    exprs: Punctuated<Node, Token![,]>,
}

impl Parse for ChainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            exprs: Node::parse_multiple(input)?,
        })
    }
}

impl ToTokens for ChainArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { exprs } = self;
        exprs.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use ::quote::quote;

    use crate::array_expr;

    #[test]
    fn chain_fn() {
        let expr = quote! {
            A B C ->
                .join
                .chain(D E F ->
                    .join
                    .chain( G H I -> .join )
                )
                .join(snake)
                .ty(ident)
                .chain // Empty does nothing
                .chain()
        };
        let expected = quote! { ABC_DEF_GHI };
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());
    }
}
