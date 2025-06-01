//! [FunctionChain] impl.

use ::file_suite_proc_lib::{Lookahead, ToArg};
use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Lookahead1, Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    array_expr::{
        function::{Call, Function, FunctionCallable, ToCallable},
        storage::Storage,
        value_array::ValueArray,
    },
    util::lookahead_parse::{LookaheadParse, lookahead_parse},
};

// /// A function chain.
// #[derive(Debug, Clone, Default)]
// pub struct FunctionChain {
//     /// Leading.
//     dot: Option<Token![.]>,
//     /// Functions of chain.
//     functions: Punctuated<Function, Token![.]>,
// }

/// A function chain.
#[derive(Debug, Clone, Default)]
pub struct FunctionChain {
    /// Leading dot of function chain, if any.
    pub leading_dot: Option<Token![.]>,
    /// Functions of function chain.
    pub functions: Punctuated<Function, Token![.]>,
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

        if should_terminate(&lookahead) {
            return Ok(Self::default());
        }

        let mut functions = Punctuated::new();
        let mut leading_dot = None;

        if let dot @ Some(..) = lookahead_parse(input, &lookahead)? {
            leading_dot = dot;
            functions.push_value(input.parse()?);
        } else if let Some(first) = lookahead_parse(input, &lookahead)? {
            functions.push_value(first);
        } else {
            return Err(lookahead.error());
        }

        let mut chains = Self {
            leading_dot,
            functions,
        };

        chains.parse_additional(input, should_terminate)?;

        Ok(chains)
    }

    /// Parse additional functions from input.
    ///
    /// # Errors
    /// If any invalid values are encountered.
    pub fn parse_additional(
        &mut self,
        input: ParseStream,
        should_terminate: fn(&Lookahead1) -> bool,
    ) -> ::syn::Result<()> {
        loop {
            let lookahead = input.lookahead1();
            if should_terminate(&lookahead) {
                return Ok(());
            }

            self.functions.push_punct(input.parse()?);
            self.functions.push_value(input.parse()?);
        }
    }

    /// Get a callable chain from self.
    pub fn to_call_chain(&self) -> Vec<FunctionCallable> {
        self.functions.iter().map(|f| f.to_callable()).collect()
    }

    /// Call a function chain on a value array.
    ///
    /// # Errors
    /// If any of the functions in the chain errors.
    pub fn call_chain(
        chain: &[FunctionCallable],
        mut array: ValueArray,
        storage: &mut Storage,
    ) -> crate::Result<ValueArray> {
        for func in chain {
            array = func.call(array, storage)?;
        }
        Ok(array)
    }
}

impl ToArg for FunctionChain {
    type Arg = Vec<FunctionCallable>;

    fn to_arg(&self) -> Self::Arg {
        self.to_call_chain()
    }
}

impl Parse for FunctionChain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_terminated(input, |lookahead| lookahead.peek(End))
    }
}

impl Lookahead for FunctionChain {
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(End)
            || lookahead.peek(Token![=])
            || <Function as Lookahead>::lookahead_peek(lookahead)
    }
}

impl LookaheadParse for FunctionChain {}

impl ToTokens for FunctionChain {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            functions,
            leading_dot,
        } = self;
        leading_dot.to_tokens(tokens);
        functions.to_tokens(tokens);
    }
}

/// A list set of funvtion chains.
#[derive(Debug, Clone)]
pub struct FunctionChains(Punctuated<FunctionChain, Token![,]>);

impl ToArg for FunctionChains {
    type Arg = Vec<Vec<FunctionCallable>>;

    fn to_arg(&self) -> Self::Arg {
        self.0.iter().map(|chain| chain.to_call_chain()).collect()
    }
}

impl Parse for FunctionChains {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let chains = Punctuated::parse_terminated_with(input, |input| {
            FunctionChain::parse_terminated(input, |lookahead| {
                lookahead.peek(Token![,]) || lookahead.peek(End)
            })
        })?;

        if chains.is_empty() {
            return Err(input.error("at least one chain expected"));
        }

        Ok(Self(chains))
    }
}

impl ToTokens for FunctionChains {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self(chains) = self;
        chains.to_tokens(tokens);
    }
}
