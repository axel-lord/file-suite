//! [chunk] impl.

use ::std::{borrow::Cow, num::NonZero};

use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{End, Parse},
};
use proc_macro2::TokenStream;
use syn::parse::ParseStream;

use crate::{
    array_expr::{
        function::{Call, FunctionCallable, FunctionChain, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{group_help::GroupSingle, lookahead_parse::LookaheadParse, spanned_int::SpannedInt},
};

function_struct!(
    /// Split array into chunks and run input chain on them.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    chunk {
        /// Chunk specification.
        spec: GroupSingle<Spec>,
    }
);

impl ToCallable for chunk {
    type Call = ChunkCallable;

    fn to_callable(&self) -> Self::Call {
        let content = &self.spec.content;
        ChunkCallable {
            size: content.chunk_size.value,
            chain: content.chain.to_call_chain(),
            remainder: content
                .remainder
                .as_ref()
                .map(|remainder| remainder.chain.to_call_chain()),
        }
    }
}

/// [Call] implementor for [chunk].
#[derive(Debug, Clone)]
pub struct ChunkCallable {
    /// Size of chunks (with exceptions for last chunk).
    size: NonZero<usize>,
    /// Chain to call on chunks.
    chain: Vec<FunctionCallable>,
    /// Special chain to use on remainder (if none regular chain is used).
    remainder: Option<Vec<FunctionCallable>>,
}

impl Call for ChunkCallable {
    fn call(
        &self,
        array: ValueArray,
        storage: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        let mut array = array.into_iter();
        let mut out_array = ValueArray::new();

        loop {
            let values = array.by_ref().take(self.size.get()).collect::<ValueArray>();
            if values.is_empty() {
                break;
            }

            let chain = match &self.remainder {
                Some(remainder) if values.len() != self.size.get() => remainder,
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

/// Specification for how many values are in each chunk (except the last which may be
/// smaller) and what chain to call on them.
/// If a second chain is specified (may be empty) it is called on the remainder instead.
#[derive(Debug, Clone)]
pub struct Spec {
    /// Size of chunks.
    chunk_size: SpannedInt<NonZero<usize>>,
    /// ',' token.
    comma_token: Token![,],
    /// Function chain.
    chain: FunctionChain,
    /// Remainder chain.
    remainder: Option<RemainderChain>,
}

impl ToTokens for Spec {
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

impl Parse for Spec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let chunk_size = input.call(SpannedInt::parse)?;
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
