//! [global] and [local] impl.

use ::std::borrow::Cow;

use ::quote::ToTokens;
use ::syn::{
    MacroDelimiter, Token,
    parse::{End, Parse},
    punctuated::Punctuated,
};

use crate::{
    array_expr::{
        ArrayExpr, Node,
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        typed_value::TypedValue,
        value_array::ValueArray,
    },
    util::{
        delimited::MacroDelimExt,
        group_help::GroupSingle,
        lookahead_parse::{LookaheadParse, lookahead_parse_terminated},
    },
};

function_struct!(
    /// Set a global variable.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    global {
        /// Specification for how to, and with what content set global.
        spec: GroupSingle<Spec>,
    }

    /// Set a local variable.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    local {
        /// Specification for how to, and with what content set local.
        spec: GroupSingle<Spec>,
    }
);

impl ToCallable for global {
    type Call = GlobalCallable;

    fn to_callable(&self) -> Self::Call {
        GlobalCallable(Behaviour::from(&self.spec.content))
    }
}

impl ToCallable for local {
    type Call = LocalCallable;

    fn to_callable(&self) -> Self::Call {
        LocalCallable(Behaviour::from(&self.spec.content))
    }
}

/// [Call] implementor fo [local].
#[derive(Debug, Clone)]
pub struct LocalCallable(Behaviour);

impl Call for LocalCallable {
    fn call(
        &self,
        array: ValueArray,
        storage: &mut Storage,
    ) -> Result<ValueArray, std::borrow::Cow<'static, str>> {
        self.0
            .set_variables(array, storage, |storage, key| storage.insert(key, false))?;
        Ok(ValueArray::new())
    }
}

/// [Call] implementor fo [global].
#[derive(Debug, Clone)]
pub struct GlobalCallable(Behaviour);

impl Call for GlobalCallable {
    fn call(
        &self,
        array: ValueArray,
        storage: &mut Storage,
    ) -> Result<ValueArray, std::borrow::Cow<'static, str>> {
        self.0.set_variables(array, storage, |storage, key| {
            storage.insert_global(key, false)
        })?;
        Ok(ValueArray::new())
    }
}

/// Whether to set variables specified by array or input, with the other as values.
#[derive(Debug, Clone)]
pub enum Behaviour {
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
}

impl Behaviour {
    /// Set variables according to variant.
    ///
    /// # Errors
    /// If the values cannot be computed, in the case of SetArray.
    /// Or if the variables to be set are read-only.
    fn set_variables<I>(
        &self,
        array: ValueArray,
        storage: &mut Storage,
        insert: I,
    ) -> Result<(), std::borrow::Cow<'static, str>>
    where
        I: for<'a, 'k> Fn(
            &'a mut Storage,
            Cow<'k, str>,
        ) -> Result<&'a mut ValueArray, Cow<'k, str>>,
    {
        match self {
            Behaviour::SetArray { key_expr } => {
                let values = storage
                    .with_local_layer(|storage| key_expr.compute_with_storage(storage))
                    .map_err(|err| err.to_string())?;

                for key in array.into_iter().map(String::from) {
                    let var = insert(storage, Cow::Owned(key)).map_err(|key| {
                        Cow::Owned(format!("cannot set read-only variable '{key}'"))
                    })?;

                    *var = values.clone();
                }
            }
            Behaviour::SetInput { keys } => {
                for key in keys {
                    let var = insert(storage, Cow::Borrowed(key)).map_err(|key| {
                        Cow::Owned(format!("cannot set read-only variable '{key}'"))
                    })?;

                    *var = array.clone();
                }
            }
        };
        Ok(())
    }
}

impl From<&Spec> for Behaviour {
    fn from(value: &Spec) -> Self {
        match value {
            Spec::SetArray { expr } => Self::SetArray {
                key_expr: expr
                    .as_ref()
                    .map(|expr| expr.content.to_array_expr())
                    .unwrap_or_default(),
            },
            Spec::SetInput { keys } => Self::SetInput {
                keys: keys.iter().map(|key| key.to_value()).collect(),
            },
        }
    }
}

/// What variable to set, or what to set variables to.
#[derive(Debug, Clone)]
pub enum Spec {
    /// Use array as variable keys. Setting them to input, which is an array expression.
    SetArray {
        /// Value to set variables to.
        expr: Option<GroupSingle<Node>>,
    },
    /// Use input which is a list of values as variable keys. setting them to array.
    SetInput {
        /// Variable keys to set.
        keys: Punctuated<TypedValue, Token![,]>,
    },
}

impl Parse for Spec {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        Ok(if lookahead.peek(End) {
            Self::SetArray { expr: None }
        } else if MacroDelimiter::lookahead_peek(&lookahead) {
            Self::SetArray {
                expr: Some(input.call(LookaheadParse::parse)?),
            }
        } else if let Some(keys) = lookahead_parse_terminated(input, &lookahead)? {
            Self::SetInput { keys }
        } else {
            return Err(lookahead.error())?;
        })
    }
}

impl ToTokens for Spec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Spec::SetArray { expr } => expr.to_tokens(tokens),
            Spec::SetInput { keys } => keys.to_tokens(tokens),
        }
    }
}
