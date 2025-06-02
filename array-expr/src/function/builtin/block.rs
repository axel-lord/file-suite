//! [BlockArgs] impl.

use ::quote::ToTokens;
use ::syn::{Token, parse::Parse, punctuated::Punctuated};

use crate::{
    ArrayExpr, ParsedArrayExpr,
    function::{Call, DefaultArgs, ToCallable},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor for [BlockArgs].
#[derive(Debug, Clone)]
pub struct BlockCallable {
    /// Array expressions of block.
    exprs: Vec<ArrayExpr>,
}

impl Call for BlockCallable {
    fn call(&self, mut array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        for expr in &self.exprs {
            array.extend(expr.compute_with_storage(storage)?);
        }
        Ok(array)
    }
}

impl DefaultArgs for BlockCallable {
    fn default_args() -> Self {
        Self { exprs: Vec::new() }
    }
}

/// Array expressions to chain after current.
#[derive(Debug, Clone)]
pub struct BlockArgs {
    /// Array expressions of block.
    exprs: Punctuated<ParsedArrayExpr, Token![,]>,
}

impl ToCallable for BlockArgs {
    type Call = BlockCallable;

    fn to_callable(&self) -> Self::Call {
        BlockCallable {
            exprs: self.exprs.iter().map(ParsedArrayExpr::to_array_expr).collect(),
        }
    }
}

impl Parse for BlockArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            exprs: ParsedArrayExpr::parse_multiple(input)?,
        })
    }
}

impl ToTokens for BlockArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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
    fn local_scope() {
        let expr = quote! {
            A B C ->
                .local(values)
                .block( =values D E F -> .local(values) )
                .block( =values 1 2 3 )
                .join
                .ty(ident)
        };
        let expected = quote! {ABCDEF123};
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn block_expressions() {
        let expr = quote! {
            A B C ->
                .join
                .block(D E F ->
                    .join
                    .block( G H I -> .join )
                )
                .join(snake)
                .ty(ident)
                .block // Empty does nothing
                .block()
        };
        let expected = quote! { ABC_DEF_GHI };
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());
    }
}
