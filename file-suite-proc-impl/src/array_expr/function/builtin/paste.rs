//! [PasteArgs] impl.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::{
    ArrayExprPaste,
    array_expr::{
        function::{Call, ToCallable},
        storage::Storage,
        value::Value,
        value_array::ValueArray,
    },
    util::fold_tokens::fold_token_stream,
};

/// Arguments for paste function.
#[derive(Debug, Clone)]
pub struct PasteArgs {
    /// Tokens to search for array expressions in.
    content: TokenStream,
}

impl Parse for PasteArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            content: input.parse()?,
        })
    }
}

impl ToTokens for PasteArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.content.clone());
    }
}

impl ToCallable for PasteArgs {
    type Call = PasteCallable;

    fn to_callable(&self) -> Self::Call {
        PasteCallable {
            content: self.content.clone(),
        }
    }
}

/// [Call] implementor for [PasteArgs].
#[derive(Debug, Clone)]
pub struct PasteCallable {
    /// Tokens to check for array expressions.
    content: TokenStream,
}

impl Call for PasteCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        if !array.is_empty() {
            return Err(
                "paste should not be used with a non-empty array, use clear to clear it if this is intended".into(),
            );
        }
        let tokens = fold_token_stream(&mut ArrayExprPaste { storage }, self.content.clone())?;
        let value = Value::new_tokens(tokens);
        Ok(ValueArray::from_value(value))
    }
}
