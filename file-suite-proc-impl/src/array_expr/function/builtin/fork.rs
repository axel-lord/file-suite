//! [ForkArgs] impl.

use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Parse},
    punctuated::Punctuated,
};

use crate::array_expr::{
    function::{Call, FunctionCallable, FunctionChain, ToCallable},
    storage::Storage,
    value_array::ValueArray,
};

/// [Call] implementor fo [ForkArgs].
#[derive(Debug, Clone)]
pub struct ForkCallable {
    /// Function chains.
    chains: Vec<Vec<FunctionCallable>>,
}

impl Call for ForkCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
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
pub struct ForkArgs {
    /// Parsed chains.
    chains: Punctuated<FunctionChain, Token![,]>,
}

impl ToCallable for ForkArgs {
    type Call = ForkCallable;

    fn to_callable(&self) -> Self::Call {
        let chains = self
            .chains
            .iter()
            .map(|chain| chain.to_call_chain())
            .collect();
        ForkCallable { chains }
    }
}

impl Parse for ForkArgs {
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

impl ToTokens for ForkArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { chains } = self;
        chains.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    fn fork_join() {
        assert_arr_expr!(
            {
                A B C ->
                    .fork {
                        .join(space).ty(str),
                        .join.case(pascal).ty(ident),
                        ,
                    }
            },
            {
                "A B C" Abc A B C
            }
        );
    }
}
