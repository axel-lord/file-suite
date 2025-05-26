//! [Function] impl.

use ::std::fmt::Debug;

pub mod builtin {
    //! Builtin funtions.

    pub mod alias;
    pub mod case;
    pub mod chunks;
    pub mod clear;
    pub mod count;
    pub mod enumerate;
    pub mod fork;
    pub mod join;
    pub mod paste;
    pub mod repeat;
    pub mod rev;
    pub mod set;
    pub mod shift;
    pub mod split;
    pub mod stairs;
    pub mod trim;
    pub mod ty;
    pub mod use_alias;
}

use crate::{
    array_expr::function::{
        builtin::{
            alias::AliasArgs,
            case::CaseArgs,
            chunks::ChunksArgs,
            clear::ClearCallable,
            count::CountCallable,
            enumerate::EnumerateArgs,
            fork::ForkArgs,
            join::JoinArgs,
            paste::PasteArgs,
            repeat::RepeatArgs,
            rev::RevCallable,
            set::{Global, Local, SetArgs},
            shift::ShiftArgs,
            split::SplitArgs,
            stairs::StairsArgs,
            trim::TrimCallable,
            ty::TyArgs,
            use_alias::UseAlias,
        },
        macros::function_enum,
    },
    lookahead_parse_keywords,
    util::group_help::{Delimited, OptionalDelimited},
};

pub use self::{
    call::{Call, ToCallable},
    chain::FunctionChain,
    empty_args::EmptyArgs,
    keyword_function::KwFn,
};

mod macros;

mod call;

mod chain;

mod keyword_function;

mod empty_args;

/// Type used in call chains, result of [ToCallable] on [Function].
pub type FunctionCallable = <Function as ToCallable>::Call;

lookahead_parse_keywords![
    alias, case, chunks, clear, count, split, join, ty, enumerate, rev, trim, shift, fork, repeat,
    stairs, paste, global, local,
];

function_enum!(
    /// Enum collecting [Call] implementors.
    #[derive(Debug, Clone)]
    Function {
        /// Split array according to specification
        Split(KwFn<kw::split, Delimited<SplitArgs>>),
        /// Join array according to specification.
        Join(KwFn<kw::join, OptionalDelimited<JoinArgs>>),
        /// Case array according to specification.
        Case(KwFn<kw::case, Delimited<CaseArgs>>),
        /// Convert type of array.
        Type(KwFn<kw::ty, Delimited<TyArgs>>),
        /// Enumerate array.
        Enumerate(KwFn<kw::enumerate, Option<Delimited<EnumerateArgs>>>),
        /// Reverse array.
        Rev(KwFn<kw::rev, EmptyArgs<RevCallable>>),
        /// Trim array array.
        Trim(KwFn<kw::trim, EmptyArgs<TrimCallable>>),
        /// Shift/Rotate array.
        Shift(KwFn<kw::shift, OptionalDelimited<ShiftArgs>>),
        /// Fork array.
        Fork(KwFn<kw::fork, Delimited<ForkArgs>>),
        /// Repeat array.
        Repeat(KwFn<kw::repeat, Delimited<RepeatArgs>>),
        /// Stair array.
        Stairs(KwFn<kw::stairs, Delimited<StairsArgs>>),
        /// Paste tokens.
        Paste(KwFn<kw::paste, Delimited<PasteArgs>>),
        /// Count array values.
        Count(KwFn<kw::count, EmptyArgs<CountCallable>>),
        /// Split array into chunks.
        Chunks(KwFn<kw::chunks, Delimited<ChunksArgs>>),
        /// Clear array.
        Clear(KwFn<kw::clear, EmptyArgs<ClearCallable>>),
        /// Set a global variable.
        Global(KwFn<kw::global, Delimited<SetArgs<Global>>>),
        /// Set a local variable.
        Local(KwFn<kw::local, Delimited<SetArgs<Local>>>),
        /// Set an alias.
        Alias(KwFn<kw::alias, Delimited<AliasArgs>>),
        /// Use an alias.
        UseAlias(UseAlias),
    }
);
