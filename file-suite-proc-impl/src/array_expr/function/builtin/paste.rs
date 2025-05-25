//! [paste] impl.

use ::proc_macro2::TokenStream;

use crate::{
    ArrayExprPaste,
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value::Value,
        value_array::ValueArray,
    },
    util::{fold_tokens::fold_token_stream, group_help::Delimited},
};

function_struct!(
    /// Same as array_expr_paste except with access to current variables.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    paste {
        /// Content to check or array expressions.
        content: Delimited<TokenStream>,
    }
);

impl ToCallable for paste {
    type Call = PasteCallable;

    fn to_callable(&self) -> Self::Call {
        PasteCallable {
            content: self.content.inner.clone(),
        }
    }
}

/// [Call] implementor for [paste].
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
