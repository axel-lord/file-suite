//! [UseAlias] impl.

use ::std::borrow::Cow;

use ::quote::ToTokens;
use ::syn::{Token, parse::ParseStream};

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        storage::Storage,
        typed_value::TypedValue,
        value_array::ValueArray,
    },
    util::{
        group_help::EmptyDelimited,
        lookahead_parse::{LookaheadParse, lookahead_parse, optional_parse},
    },
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
    fn call(
        &self,
        mut array: ValueArray,
        storage: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        let Some(alias) = storage.get_alias(&self.alias_key) else {
            return Err(Cow::Owned(format!(
                "could net get chain alias for key '{}'",
                self.alias_key
            )));
        };

        for func in alias.as_ref() {
            array = func.call(array, storage)?;
        }

        Ok(array)
    }
}

impl LookaheadParse for UseAlias {
    fn lookahead_parse(
        input: ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        let Some(eq_token) = lookahead_parse(input, lookahead)? else {
            return Ok(None);
        };

        let alias_key = input.call(TypedValue::parse)?;
        let delim = optional_parse(input)?;

        Ok(Some(Self {
            eq_token,
            alias_key,
            delim,
        }))
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
