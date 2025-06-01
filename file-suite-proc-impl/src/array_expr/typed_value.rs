//! [TypedValue] impl

use ::file_suite_proc_lib::{
    Lookahead, ToArg,
    to_arg::{PunctuatedToArg, SliceToArg},
};
use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::{ToTokens, TokenStreamExt};
use ::syn::{
    Ident, LitBool, LitInt, LitStr,
    ext::IdentExt,
    parse::{Lookahead1, Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    array_expr::{value::Value, value_array::ValueArray},
    util::lookahead_parse::LookaheadParse,
};

/// A typed [Value] which may be converted to tokens.
#[derive(Debug, Clone)]
pub enum TypedValue {
    /// Value is an [identifier][Ident].
    Ident(Ident),
    /// Value is a [string literal][LitStr].
    LitStr(LitStr),
    /// Value is an [integer literal][LitInt].
    LitInt(isize, Span),
    /// Value is a [boolean literal][LitBool].
    LitBool(LitBool),
    /// Value is a token stream (cannot be parsed, but is created in some contexts).
    Tokens(TokenStream),
}

impl ToArg for TypedValue {
    type Arg = String;

    fn to_arg(&self) -> Self::Arg {
        self.to_value().into()
    }
}

impl TypedValue {
    /// Convert to a [Value]
    ///
    /// # Panics
    /// If of the Expr, Item or Stmt variants.
    pub fn to_value(&self) -> Value {
        match self {
            TypedValue::Ident(ident) => Value::from(ident),
            TypedValue::LitStr(lit_str) => Value::from(lit_str),
            TypedValue::LitBool(lit_bool) => Value::from(lit_bool),
            TypedValue::LitInt(value, span) => {
                let mut value = Value::new_int(*value);
                value.set_span(*span);
                value
            }
            TypedValue::Tokens(token_stream) => Value::new_tokens(token_stream.clone()),
        }
    }
}

impl Lookahead for TypedValue {
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(LitStr) || lookahead.peek(LitInt) || lookahead.peek(Ident)
    }

    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>>
    where
        Self: Parse,
    {
        Ok(Some(if lookahead.peek(Ident) {
            Self::Ident(input.call(Ident::parse_any)?)
        } else if lookahead.peek(LitStr) {
            Self::LitStr(input.parse()?)
        } else if lookahead.peek(LitInt) {
            let lit_int = input.parse::<LitInt>()?;
            Self::LitInt(lit_int.base10_parse()?, lit_int.span())
        } else {
            return Ok(None);
        }))
    }
}

impl Parse for TypedValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        match <Self as Lookahead>::lookahead_parse(input, &lookahead)? {
            None => Err(lookahead.error()),
            Some(value) => Ok(value),
        }
    }
}

impl LookaheadParse for TypedValue {}

impl ToTokens for TypedValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TypedValue::Ident(ident) => ident.to_tokens(tokens),
            TypedValue::LitStr(lit_str) => lit_str.to_tokens(tokens),
            TypedValue::LitInt(value, span) => {
                let mut literal = Literal::isize_unsuffixed(*value);
                literal.set_span(*span);
                tokens.append(literal);
            }
            TypedValue::LitBool(lit_bool) => lit_bool.to_tokens(tokens),
            TypedValue::Tokens(token_stream) => token_stream.to_tokens(tokens),
        }
    }
}

impl PunctuatedToArg for TypedValue {
    type Arg = ValueArray;

    fn punctuated_to_arg<P>(punctuated: &Punctuated<Self, P>) -> Self::Arg {
        punctuated.iter().map(TypedValue::to_value).collect()
    }
}

impl SliceToArg for TypedValue {
    type Arg = ValueArray;

    fn slice_to_arg(slice: &[Self]) -> Self::Arg {
        slice.iter().map(TypedValue::to_value).collect()
    }
}

/// A punctuated list of typed values.
#[derive(Debug, Clone)]
pub struct TypedValues<P>(Punctuated<TypedValue, P>);

impl<P> ToArg for TypedValues<P> {
    type Arg = ValueArray;

    fn to_arg(&self) -> Self::Arg {
        self.0.to_arg()
    }
}

impl<P> Lookahead for TypedValues<P> {
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        TypedValue::lookahead_peek(lookahead)
    }
}

impl<P> Parse for TypedValues<P>
where
    P: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}

impl<P> LookaheadParse for TypedValues<P> where P: LookaheadParse {}

impl<P> ToTokens for TypedValues<P>
where
    P: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(values) = self;
        values.to_tokens(tokens);
    }
}
