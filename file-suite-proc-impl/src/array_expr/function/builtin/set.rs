//! [Global] and [Local], [SetArgs] impl.

use ::std::borrow::Cow;

use ::file_suite_proc_lib::{lookahead::ParseBufferExt, neverlike::NoPhantomData};
use ::quote::ToTokens;
use ::syn::{
    MacroDelimiter,
    parse::{End, Parse},
    punctuated::Punctuated,
};

use crate::{
    array_expr::{
        ArrayExpr, Node,
        function::{Call, ToCallable},
        storage::Storage,
        typed_value::TypedValue,
        value_array::ValueArray,
    },
    util::{delimited::MacroDelimExt, group_help::Delimited},
};

#[derive(Debug, Clone)]
/// Set global variables.
pub enum Global {}

#[derive(Debug, Clone)]
/// Set local variables.
pub enum Local {}

impl Call for SetCallable<Local> {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        self.set_variables(array, storage, |storage, key| {
            storage
                .insert(key, false)
                .map_err(|key| format!("read-only local with key '{key}' already exists").into())
        })?;
        Ok(ValueArray::new())
    }
}

impl Call for SetCallable<Global> {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        self.set_variables(array, storage, |storage, key| {
            storage
                .insert_global(key, false)
                .map_err(|key| format!("read-only global with key '{key}' already exists").into())
        })?;
        Ok(ValueArray::new())
    }
}

/// Whether to set variables specified by array or input, with the other as values.
#[derive(Debug, Clone)]
pub enum SetCallable<T> {
    /// Use array as keys.
    SetArray {
        /// Values to set variables to.
        key_expr: ArrayExpr,
    },
    /// Use input as keys.
    SetInput {
        /// Keys of variables to set.
        keys: ValueArray,
    },
    /// Phantom variant.
    _P {
        /// Allow for T.
        _p: NoPhantomData<fn() -> T>,
    },
}

impl<T> SetCallable<T> {
    /// Set variables according to variant.
    ///
    /// # Errors
    /// If the values cannot be computed, in the case of SetArray.
    /// Or if the variables to be set are read-only.
    fn set_variables<I>(&self, array: ValueArray, storage: &mut Storage, insert: I) -> crate::Result
    where
        I: for<'a, 'k> Fn(&'a mut Storage, Cow<'k, str>) -> crate::Result<&'a mut ValueArray>,
    {
        match self {
            SetCallable::SetArray { key_expr } => {
                let values = storage
                    .with_local_layer(|storage| key_expr.compute_with_storage(storage))
                    .map_err(|err| err.to_string())?;

                for key in array.into_iter().map(String::from) {
                    let var = insert(storage, Cow::Owned(key))
                        .map_err(|key| format!("cannot set read-only variable '{key}'"))?;

                    *var = values.clone();
                }
            }
            SetCallable::SetInput { keys } => {
                for key in keys {
                    let var = insert(storage, Cow::Borrowed(key))
                        .map_err(|key| format!("cannot set read-only variable '{key}'"))?;

                    *var = array.clone();
                }
            }
            SetCallable::_P { _p } => _p.unwrap(),
        };
        Ok(())
    }
}

/// What variable to set, or what to set variables to.
#[derive(Debug, Clone)]
pub enum SetArgs<T> {
    /// Use array as variable keys. Setting them to input, which is an array expression.
    SetArray {
        /// Value to set variables to.
        expr: Option<Delimited<Node>>,
    },
    /// Use input which is a list of values as variable keys. setting them to array.
    SetInput {
        /// Variable keys to set.
        keys: Punctuated<TypedValue, ::syn::token::Comma>,
    },
    /// Phantom variant.
    _P {
        /// Allow T.
        _p: NoPhantomData<fn() -> T>,
    },
}

impl<T> ToCallable for SetArgs<T>
where
    SetCallable<T>: Call,
{
    type Call = SetCallable<T>;

    fn to_callable(&self) -> Self::Call {
        match self {
            SetArgs::SetArray { expr } => SetCallable::<T>::SetArray {
                key_expr: expr
                    .as_ref()
                    .map(|expr| expr.inner.to_array_expr())
                    .unwrap_or_default(),
            },
            SetArgs::SetInput { keys } => SetCallable::<T>::SetInput {
                keys: keys.iter().map(|key| key.to_value()).collect(),
            },
            SetArgs::_P { _p } => _p.unwrap(),
        }
    }
}

impl<T> Parse for SetArgs<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        Ok(if lookahead.peek(End) {
            Self::SetArray { expr: None }
        } else if MacroDelimiter::lookahead_peek(&lookahead) {
            Self::SetArray {
                expr: Some(input.parse()?),
            }
        } else if let Some(keys) = input.lookahead_parse_terminated(&lookahead)? {
            Self::SetInput { keys }
        } else {
            return Err(lookahead.error())?;
        })
    }
}

impl<T> ToTokens for SetArgs<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            SetArgs::SetArray { expr } => expr.to_tokens(tokens),
            SetArgs::SetInput { keys } => keys.to_tokens(tokens),
            SetArgs::_P { _p } => _p.unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    fn set_global() {
        assert_arr_expr!(
            {
                -> .chain(-> .chain(-> .chain(-> .chain( value -> global(var) )))),
                =var,
            },
            { value }
        );
    }

    #[test]
    fn set_local() {
        assert_arr_expr!(
            {
                1234 ->
                    .local(N)
                    .chain( 5678 -> .local(N) )
                    .chain( 9 -> .global(N) )
                    .chain(=N)
            },
            { 1234 },
        );
    }
}
