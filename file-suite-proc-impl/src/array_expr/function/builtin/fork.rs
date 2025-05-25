//! [fork] impl.

use ::std::borrow::Cow;

use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Parse},
    punctuated::Punctuated,
};

use crate::{
    array_expr::{
        function::{Call, FunctionCallable, FunctionChain, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::group_help::GroupSingle,
};

function_struct!(
    /// Fork array duplicating it and running all input function chains on
    /// their own copy, then joining them in order of the function chains.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    fork {
        /// Fork function chains.
        forks: GroupSingle<Forks>,
    }
);

impl ToCallable for fork {
    type Call = ForkCallable;

    fn to_callable(&self) -> Self::Call {
        ForkCallable {
            chains: self
                .forks
                .content
                .chains
                .iter()
                .map(|chain| chain.to_call_chain())
                .collect(),
        }
    }
}

/// [Call] implementor fo [fork].
#[derive(Debug, Clone)]
pub struct ForkCallable {
    /// Function chains.
    chains: Vec<Vec<FunctionCallable>>,
}

impl Call for ForkCallable {
    fn call(
        &self,
        array: ValueArray,
        storage: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        let mut output_array = ValueArray::new();

        for chain in &self.chains {
            output_array.extend(storage.with_local_layer(|storage| {
                FunctionChain::call_chain(chain, array.clone(), storage)
            })?);
        }

        Ok(output_array)
    }
}

/// Function chains to fork to.
#[derive(Debug, Clone)]
pub struct Forks {
    /// Parsed chains.
    chains: Punctuated<FunctionChain, Token![,]>,
}

impl Parse for Forks {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let chains = Punctuated::parse_terminated_with(input, |input| {
            FunctionChain::parse_terminated(input, |lookahead| {
                lookahead.peek(Token![,]) || lookahead.peek(End)
            })
        })?;

        if chains.is_empty() {
            return Err(input.error("fork expects at least one function chain"));
        }

        Ok(Self { chains })
    }
}

impl ToTokens for Forks {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { chains } = self;
        chains.to_tokens(tokens);
    }
}
