//! [UseAlias] impl.

use ::file_suite_proc_lib::{Lookahead, lookahead::ParseBufferExt};
use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{Parse, ParseStream},
};

use crate::{
    array_expr::{
        function::{Call, FunctionChain, ToCallable},
        storage::Storage,
        typed_value::TypedValue,
        value_array::ValueArray,
    },
    util::group_help::EmptyDelimited,
};

/// Use an alias.
#[derive(Debug, Clone)]
pub struct UseAlias {
    /// '=' token signals an alias is to be used.
    eq_token: Token![=],
    /// Alias to use.
    alias_key: TypedValue,
    /// Optional empty delim group for chain parity.
    delim: Option<EmptyDelimited>,
}

impl ToCallable for UseAlias {
    type Call = UseAliasCallable;

    fn to_callable(&self) -> Self::Call {
        UseAliasCallable {
            alias_key: String::from(self.alias_key.to_value()),
        }
    }
}

/// [Call] implementor for [UseAlias].
#[derive(Debug, Clone)]
pub struct UseAliasCallable {
    /// Key for alias to use.
    alias_key: String,
}

impl Call for UseAliasCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        let alias = storage
            .get_alias(&self.alias_key)
            .ok_or_else(|| format!("could net get chain alias for key '{}'", self.alias_key))?;

        storage.with_local_layer(|storage| FunctionChain::call_chain(&alias, array, storage))
    }
}

impl Lookahead for UseAlias {
    fn lookahead_peek(lookahead: &syn::parse::Lookahead1) -> bool {
        lookahead.peek(Token![=])
    }
}

impl Parse for UseAlias {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            eq_token: input.parse()?,
            alias_key: input.parse()?,
            delim: input.optional_parse()?,
        })
    }
}

impl ToTokens for UseAlias {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            eq_token,
            alias_key,
            delim,
        } = self;
        eq_token.to_tokens(tokens);
        alias_key.to_tokens(tokens);
        delim.to_tokens(tokens);
    }
}
