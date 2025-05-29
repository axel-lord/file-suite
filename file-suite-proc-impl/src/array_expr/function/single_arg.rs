//! Utility to simplify single argument functions.

use ::std::{fmt::Debug, marker::PhantomData};

use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::{
    array_expr::{
        from_values::FromValues,
        function::{Call, DefaultArgs, ParsedArg, ToArg, ToCallable},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{lookahead_parse::LookaheadParse, neverlike::NoPhantomData},
};

/// Single argument parse implementor.
pub struct SingleArg<V, C> {
    /// Parsed argument, which may be a variable access.
    arg: ParsedArg<V>,
    /// Allow C to exist.
    _p: PhantomData<fn() -> C>,
}

impl<V, C> ToTokens for SingleArg<V, C>
where
    V: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { arg, _p: _ } = self;
        arg.to_tokens(tokens);
    }
}

impl<V, C> Parse for SingleArg<V, C>
where
    V: LookaheadParse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            arg: input.call(ParsedArg::parse)?,
            _p: PhantomData,
        })
    }
}

impl<V, C> Clone for SingleArg<V, C>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            arg: self.arg.clone(),
            _p: PhantomData,
        }
    }
}

impl<V, C> Debug for SingleArg<V, C>
where
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleArg")
            .field("arg", &self.arg)
            .field("_p", &self._p)
            .finish()
    }
}

impl<V, C> ToCallable for SingleArg<V, C>
where
    C: Call + From<V::Arg>,
    V: ToArg,
    V::Arg: FromValues,
{
    type Call = SingleArgCallable<V::Arg, C>;

    fn to_callable(&self) -> Self::Call {
        match &self.arg {
            ParsedArg::Variable { eq_token: _, var } => {
                SingleArgCallable::Variable(var.to_value().into())
            }
            ParsedArg::Value(value) => SingleArgCallable::Callable(C::from(value.to_arg())),
        }
    }
}

/// [Call] implementor for [SingleArg].
pub enum SingleArgCallable<V, C> {
    /// Get arg from a variable.
    Variable(String),
    /// Arg is already gotten.
    Callable(C),
    /// Allow for V to exist.
    _P(NoPhantomData<fn() -> V>),
}

impl<V, C> DefaultArgs for SingleArgCallable<V, C>
where
    C: DefaultArgs,
{
    fn default_args() -> Self {
        Self::Callable(C::default_args())
    }
}

impl<V, C> Call for SingleArgCallable<V, C>
where
    V: FromValues,
    C: Call + From<V>,
{
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        match self {
            SingleArgCallable::Variable(key) => storage
                .get(key)
                .ok_or_else(|| {
                    crate::Error::from(format!("could not get variable with key '{key}'"))
                })
                .and_then(|values| V::from_values(values))
                .map(C::from)?
                .call(array, storage),
            SingleArgCallable::Callable(callable) => callable.call(array, storage),
            SingleArgCallable::_P(no_phantom_data) => no_phantom_data.unwrap(),
        }
    }
}

impl<V, C> Clone for SingleArgCallable<V, C>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Variable(key) => Self::Variable(key.clone()),
            Self::Callable(val) => Self::Callable(val.clone()),
            Self::_P(no_phantom_data) => no_phantom_data.unwrap(),
        }
    }
}

impl<V, C> Debug for SingleArgCallable<V, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(key) => f.debug_tuple("Variable").field(key).finish(),
            Self::Callable(value) => f.debug_tuple("Callable").field(value).finish(),
            Self::_P(no_phantom_data) => no_phantom_data.unwrap(),
        }
    }
}
