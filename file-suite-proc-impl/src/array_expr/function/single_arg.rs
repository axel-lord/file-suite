//! Utility to simplify single argument functions.

use ::std::{fmt::Debug, marker::PhantomData};

use ::file_suite_proc_lib::{FromArg, Lookahead, ToArg};
use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::array_expr::{
    from_values::FromValues,
    function::{Call, DefaultArgs, ParsedArg, ToCallable},
    storage::Storage,
    value_array::ValueArray,
};

/// Single argument parse implementor.
pub struct SingleArg<C>
where
    C: FromArg,
{
    /// Parsed argument, which may be a variable access.
    arg: ParsedArg<C::Factory>,
    /// Allow C to exist.
    _p: PhantomData<fn() -> C>,
}

impl<C> ToCallable for SingleArg<C>
where
    C: FromArg + Call,
    <C::Factory as ToArg>::Arg: FromValues,
{
    type Call = SingleArgCallable<C>;

    fn to_callable(&self) -> Self::Call {
        match &self.arg {
            ParsedArg::Variable { eq_token: _, var } => {
                SingleArgCallable::Variable(var.to_value().into())
            }
            ParsedArg::Value(value) => SingleArgCallable::Callable(C::from_arg(value.to_arg())),
        }
    }
}

impl<C> Clone for SingleArg<C>
where
    C: FromArg,
    C::Factory: Clone,
{
    fn clone(&self) -> Self {
        Self {
            arg: self.arg.clone(),
            _p: PhantomData,
        }
    }
}

impl<C> Debug for SingleArg<C>
where
    C: FromArg,
    C::Factory: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleArg")
            .field("arg", &self.arg)
            .field("_p", &self._p)
            .finish()
    }
}

impl<C> Parse for SingleArg<C>
where
    C: FromArg,
    C::Factory: Lookahead + Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            arg: input.parse()?,
            _p: PhantomData,
        })
    }
}

impl<C> ToTokens for SingleArg<C>
where
    C: FromArg,
    C::Factory: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { arg, _p: _ } = self;
        arg.to_tokens(tokens);
    }
}

/// [Call] implementor for [SingleArg].
pub enum SingleArgCallable<C>
where
    C: Call,
{
    /// Get argument from variable.
    Variable(String),
    /// Already has argument.
    Callable(C),
}

impl<C> Clone for SingleArgCallable<C>
where
    C: Call + Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Variable(arg0) => Self::Variable(arg0.clone()),
            Self::Callable(arg0) => Self::Callable(arg0.clone()),
        }
    }
}

impl<C> Debug for SingleArgCallable<C>
where
    C: Call + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(arg0) => f.debug_tuple("Variable").field(arg0).finish(),
            Self::Callable(arg0) => f.debug_tuple("Callable").field(arg0).finish(),
        }
    }
}

impl<C> DefaultArgs for SingleArgCallable<C>
where
    C: Call + DefaultArgs,
{
    fn default_args() -> Self {
        Self::Callable(C::default_args())
    }
}

impl<C> Call for SingleArgCallable<C>
where
    C: Call + FromArg,
    <C::Factory as ToArg>::Arg: FromValues,
{
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        match self {
            SingleArgCallable::Variable(key) => storage
                .try_get(key)
                .and_then(|values| <C::Factory as ToArg>::Arg::from_values(values))
                .map(C::from_arg)?
                .call(array, storage),
            SingleArgCallable::Callable(callable) => callable.call(array, storage),
        }
    }
}
