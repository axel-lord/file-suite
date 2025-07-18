//! [Function] impl.

use ::std::fmt::Debug;

pub mod builtin {
    //! Builtin funtions.

    pub mod alias;
    pub mod block;
    pub mod case;
    pub mod chain;
    pub mod chunks;
    pub mod clear;
    pub mod count;
    pub mod enumerate;
    pub mod for_each;
    pub mod fork;
    pub mod get;
    pub mod intersperse;
    pub mod join;
    pub mod nth;
    pub mod paste;
    pub mod rand;
    pub mod repeat;
    pub mod rev;
    pub mod set;
    pub mod shift;
    pub mod skip;
    pub mod split;
    pub mod stairs;
    pub mod take;
    pub mod trim;
    pub mod ty;
}

use ::file_suite_proc_lib::lookahead_keywords;

use crate::{
    function::{
        builtin::{
            alias::AliasCallable,
            block::BlockArgs,
            case::CaseKind,
            chain::ChainArgs,
            chunks::ChunksArgs,
            clear::ClearCallable,
            count::CountCallable,
            enumerate::EnumerateArgs,
            for_each::ForEachCallable,
            fork::ForkCallable,
            get::GetCallable,
            intersperse::IntersperseCallable,
            join::{JoinByCallable, JoinKind},
            nth::NthCallable,
            paste::PasteArgs,
            rand::RandCallable,
            repeat::RepeatCallable,
            rev::RevCallable,
            set::{Global, Local, SetArgs},
            shift::ShiftCallable,
            skip::SkipCallable,
            split::{SplitByCallable, SplitKind},
            stairs::StairsCallable,
            take::TakeCallable,
            trim::TrimCallable,
        },
        macros::function_enum,
    },
    value::TyKind,
};

pub use self::{
    arg::{Arg, ParsedArg},
    call::{Call, DefaultArgs, ToCallable},
    chain::FunctionChain,
    deferred_args::DeferredArgs,
    delimited::Delimited,
    empty_args::EmptyArgs,
    keyword_function::KwFn,
    optional_delimited::OptionalDelimited,
    single_arg::SingleArg,
    use_alias::UseAlias,
};

mod arg;
mod call;
mod chain;
mod deferred_args;
mod delimited;
mod empty_args;
mod keyword_function;
mod macros;
mod optional_delimited;
mod single_arg;
mod use_alias;

/// Get the [Call] implementation for [ToCallable] implementors
pub type Callable<T> = <T as ToCallable>::Call;

lookahead_keywords!(
    #[doc(hidden)]
    kw {
        alias,
        case,
        chunks,
        clear,
        count,
        split,
        join,
        ty,
        enumerate,
        rev,
        trim,
        shift,
        fork,
        repeat,
        stairs,
        paste,
        global,
        local,
        chain,
        block,
        join_by,
        split_by,
        take,
        skip,
        intersperse,
        get,
        nth,
        rand,
        for_each,
    }
);

function_enum!(
    /// Enum collecting [Call] implementors.
    #[derive(Debug, Clone)]
    Function {
        /// Split array values according to input keyword.
        Split(KwFn<kw::split, Delimited<SingleArg<SplitKind>>>),
        /// Split array values by input.
        SplitBy(KwFn<kw::split_by, Delimited<SingleArg<SplitByCallable>>>),
        /// Join array according to input keyword.
        Join(KwFn<kw::join, OptionalDelimited<SingleArg<JoinKind>>>),
        /// Join an array by a value.
        JoinBy(KwFn<kw::join_by, OptionalDelimited<SingleArg<JoinByCallable>>>),
        /// Case array according to specification.
        Case(KwFn<kw::case, Delimited<SingleArg<CaseKind>>>),
        /// Convert type of array.
        Type(KwFn<kw::ty, Delimited<SingleArg<TyKind>>>),
        /// Enumerate array.
        Enumerate(KwFn<kw::enumerate, OptionalDelimited<EnumerateArgs>>),
        /// Reverse array.
        Rev(KwFn<kw::rev, EmptyArgs<RevCallable>>),
        /// Trim array array.
        Trim(KwFn<kw::trim, EmptyArgs<TrimCallable>>),
        /// Take an amount of values from array.
        Take(KwFn<kw::take, Delimited<SingleArg<TakeCallable>>>),
        /// Skip an amount of values of array.
        Skip(KwFn<kw::skip, Delimited<SingleArg<SkipCallable>>>),
        /// Shift/Rotate array.
        Shift(KwFn<kw::shift, OptionalDelimited<SingleArg<ShiftCallable>>>),
        /// Fork array.
        Fork(KwFn<kw::fork, Delimited<DeferredArgs<ForkCallable>>>),
        /// Repeat array.
        Repeat(KwFn<kw::repeat, Delimited<SingleArg<RepeatCallable>>>),
        /// Intersperse array elements with input.
        Intersperse(KwFn<kw::intersperse, Delimited<SingleArg<IntersperseCallable>>>),
        /// Stair array.
        Stairs(KwFn<kw::stairs, Delimited<DeferredArgs<StairsCallable>>>),
        /// Paste tokens.
        Paste(KwFn<kw::paste, Delimited<PasteArgs>>),
        /// Count array values.
        Count(KwFn<kw::count, EmptyArgs<CountCallable>>),
        /// Chain an array expr after array.
        Chain(KwFn<kw::chain, OptionalDelimited<ChainArgs>>),
        /// Get nth value.
        Nth(KwFn<kw::nth, Delimited<SingleArg<NthCallable>>>),
        /// Chain an array expr after array with local variable access.
        Block(KwFn<kw::block, OptionalDelimited<BlockArgs>>),
        /// Split array into chunks.
        Chunks(KwFn<kw::chunks, Delimited<ChunksArgs>>),
        /// Run a chain for each value in chain.
        ForEach(KwFn<kw::for_each, Delimited<DeferredArgs<ForEachCallable>>>),
        /// Clear array.
        Clear(KwFn<kw::clear, EmptyArgs<ClearCallable>>),
        /// Set a global variable.
        Global(KwFn<kw::global, Delimited<SetArgs<Global>>>),
        /// Get a variable.
        Get(KwFn<kw::get, EmptyArgs<GetCallable>>),
        /// Set a local variable.
        Local(KwFn<kw::local, Delimited<SetArgs<Local>>>),
        /// Convert array to random hexadecimal values.
        Rand(KwFn<kw::rand, EmptyArgs<RandCallable>>),
        /// Set an alias.
        Alias(KwFn<kw::alias, Delimited<DeferredArgs<AliasCallable>>>),
        /// Use an alias.
        UseAlias(UseAlias),
    }
);
