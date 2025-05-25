//! [FunctionChain] impl.

use ::std::borrow::Cow;

use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Lookahead1, Parse, ParseStream},
};

use crate::{
    array_expr::{
        function::{Call, Function, FunctionCallable, ToCallable},
        storage::Storage,
        value_array::ValueArray,
    },
    util::lookahead_parse::{LookaheadParse, lookahead_parse},
};

/// A function chain.
#[derive(Debug, Clone, Default)]
pub struct FunctionChain {
    /// chain of functions and leading punctuation.
    pub functions: Vec<(Option<Token![.]>, Function)>,
}

impl FunctionChain {
    /// Parse a function chain with a custom termination condition.
    ///
    /// # Note
    /// If termination condition never returns true
    /// this funtion may loop forever.
    ///
    /// # Errors
    /// On incorrect syntax.
    pub fn parse_terminated(
        input: ParseStream,
        should_terminate: fn(&Lookahead1) -> bool,
    ) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        let mut chain = Vec::new();

        if should_terminate(&lookahead) {
            return Ok(Self { functions: chain });
        } else if let dot @ Some(..) = lookahead_parse(input, &lookahead)? {
            chain.push((dot, input.call(Function::parse)?));
        } else if let Some(func) = lookahead_parse(input, &lookahead)? {
            chain.push((None, func));
        } else {
            return Err(lookahead.error());
        };

        loop {
            let lookahead = input.lookahead1();

            if should_terminate(&lookahead) {
                break;
            } else if let dot @ Some(..) = lookahead_parse(input, &lookahead)? {
                chain.push((dot, input.call(Function::parse)?));
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(Self { functions: chain })
    }

    /// Get a callable chain from self.
    pub fn to_call_chain(&self) -> Vec<FunctionCallable> {
        self.functions
            .iter()
            .map(|(_, f)| f.to_callable())
            .collect()
    }

    /// Call a function chain on a value array.
    ///
    /// # Errors
    /// If any of the functions in the chain errors.
    pub fn call_chain(
        chain: &[FunctionCallable],
        mut array: ValueArray,
        storage: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        for func in chain {
            array = func.call(array, storage)?;
        }
        Ok(array)
    }
}

impl Parse for FunctionChain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_terminated(input, |lookahead| lookahead.peek(End))
    }
}

impl ToTokens for FunctionChain {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { functions } = self;
        for (dot, func) in functions {
            dot.to_tokens(tokens);
            func.to_tokens(tokens);
        }
    }
}
