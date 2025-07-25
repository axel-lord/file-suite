//! [ChunksArgs] impl.

use ::std::num::NonZero;

use ::file_suite_proc_lib::{ToArg, spanned_int::SpannedInt};
use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Parse},
};
use proc_macro2::TokenStream;
use syn::parse::ParseStream;

use crate::{
    function::{Arg, Call, Callable, Function, FunctionChain, ParsedArg, ToCallable},
    storage::Storage,
    value_array::ValueArray,
};

/// Specification for how many values are in each chunk (except the last which may be
/// smaller) and what chain to call on them.
/// If a second chain is specified (may be empty) it is called on the remainder instead.
#[derive(Debug, Clone)]
pub struct ChunksArgs {
    /// Size of chunks.
    chunk_size: ParsedArg<SpannedInt<NonZero<usize>>>,
    /// ',' token.
    comma_token: Token![,],
    /// Function chain.
    chain: FunctionChain,
    /// Remainder chain.
    remainder: Option<RemainderChain>,
}

impl ToCallable for ChunksArgs {
    type Call = ChunksCallable;

    fn to_callable(&self) -> Self::Call {
        ChunksCallable {
            size: self.chunk_size.to_arg(),
            chain: self.chain.to_call_chain(),
            remainder: self
                .remainder
                .as_ref()
                .map(|remainder| remainder.chain.to_call_chain()),
        }
    }
}

impl ToTokens for ChunksArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            chunk_size,
            comma_token,
            chain,
            remainder,
        } = self;
        chunk_size.to_tokens(tokens);
        comma_token.to_tokens(tokens);
        chain.to_tokens(tokens);
        remainder.to_tokens(tokens);
    }
}

impl Parse for ChunksArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let chunk_size = input.parse()?;
        let comma_token = input.parse()?;
        let chain = FunctionChain::parse_terminated(input, |lookahead| {
            lookahead.peek(End) || lookahead.peek(Token![,])
        })?;

        let remainder = if input.peek(Token![,]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            chunk_size,
            comma_token,
            chain,
            remainder,
        })
    }
}

/// An optional chain to run the chunk remainder (may never be run).
#[derive(Debug, Clone)]
pub struct RemainderChain {
    /// ',' token.
    comma_token: Token![,],
    /// Function chain to run on remainder.
    chain: FunctionChain,
}

impl ToTokens for RemainderChain {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { comma_token, chain } = self;
        comma_token.to_tokens(tokens);
        chain.to_tokens(tokens);
    }
}

impl Parse for RemainderChain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            comma_token: input.parse()?,
            chain: input.parse()?,
        })
    }
}

/// [Call] implementor for [ChunksCallable].
#[derive(Debug, Clone)]
pub struct ChunksCallable {
    /// Size of chunks (with exceptions for last chunk).
    size: Arg<NonZero<usize>>,
    /// Chain to call on chunks.
    chain: Vec<Callable<Function>>,
    /// Special chain to use on remainder (if none regular chain is used).
    remainder: Option<Vec<Callable<Function>>>,
}

impl Call for ChunksCallable {
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray> {
        let mut array = array.into_iter();
        let mut out_array = ValueArray::new();

        let size = self.size.get(storage)?;

        loop {
            let values = array.by_ref().take(size.get()).collect::<ValueArray>();
            if values.is_empty() {
                break;
            }

            let chain = match &self.remainder {
                Some(remainder) if values.len() != size.get() => remainder,
                _ => &self.chain,
            };

            out_array.extend(
                storage.with_local_layer(|storage| {
                    FunctionChain::call_chain(chain, values, storage)
                })?,
            );
        }

        Ok(out_array)
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::test::assert_arr_expr;

    #[test]
    fn chunks() {
        assert_arr_expr!(
            { A -> repeat(3).enumerate.chunks(2, shift.join).ty(ident) },
            { A1 A2 A3 },
        );

        assert_arr_expr!(
            {
                2 -> global(by),
                A B C D -> chunks(=by, join).ty(ident),
            },
            { AB CD },
        );
    }
}
