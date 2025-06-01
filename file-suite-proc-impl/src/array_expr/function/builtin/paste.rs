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
        let tokens = ::fold_tokens::fold_tokens(
            &mut ArrayExprPaste { storage },
            self.content.clone().into(),
        )?;
        let value = Value::new_tokens(tokens);
        Ok(ValueArray::from_value(value))
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use ::quote::quote;

    use crate::array_expr;

    #[test]
    fn paste() {
        let expr = quote! {
            T ->
                .repeat(3)
                .enumerate
                .chunks{ 2, shift.join }
                .ty(ident)
                .stairs {
                    .local(t)
                    .paste {
                        Value(++!(=t));
                    }
                }
        };
        let expected = quote! {
            Value(T1);
            Value(T1 T2);
            Value(T1 T2 T3);
        };
        let result = array_expr(expr).unwrap();
        assert_eq!(result.to_string(), expected.to_string());
    }
}
