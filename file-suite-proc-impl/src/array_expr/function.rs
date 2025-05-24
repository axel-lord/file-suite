//! [Function] impl.

use ::std::fmt::Debug;

use ::syn::{
    Token,
    parse::{End, Lookahead1, ParseStream},
};
pub(crate) use macros::{function_enum, function_struct, spec_impl};

pub mod builtin {
    //! Builtin funtions.

    pub mod alias;
    pub mod case;
    pub mod clear;
    pub mod count;
    pub mod enumerate;
    pub mod join;
    pub mod rev;
    pub mod set;
    pub mod split;
    pub mod ty;
    pub mod use_alias;
    pub mod trim {
        //! [trim] impl

        use ::std::borrow::Cow;

        use crate::{
            array_expr::{
                function::{Call, ToCallable, function_struct},
                storage::Storage,
                value_array::ValueArray,
            },
            util::group_help::EmptyGroup,
        };

        function_struct!(
            /// Trim whitespace arround values in array.
            #[derive(Debug, Clone)]
            #[expect(non_camel_case_types)]
            trim {
                /// Optional delimiter.
                [optional] delim: Option<EmptyGroup>,
            }
        );

        impl ToCallable for trim {
            type Call = TrimCallable;

            fn to_callable(&self) -> Self::Call {
                TrimCallable
            }
        }

        /// [Call] implementor for [trim].
        #[derive(Debug, Clone, Copy)]
        pub struct TrimCallable;

        impl Call for TrimCallable {
            fn call(
                &self,
                mut array: ValueArray,
                _storage: &mut Storage,
            ) -> Result<ValueArray, Cow<'static, str>> {
                for value in &mut array {
                    // make_string may not be cheap. And the created string needs an addr.
                    if value.is_empty() {
                        continue;
                    }

                    // We make sure trim works on the same string as is drained and truncated.
                    let string = value.make_string();

                    let trimmed = string.trim();
                    let len = trimmed.len();
                    let start = trimmed.as_ptr().addr() - string.as_str().as_ptr().addr();

                    string.drain(..start);
                    string.truncate(len);
                }
                Ok(array)
            }
        }
    }
}

use crate::{
    array_expr::function::builtin::{
        alias::alias,
        case::case,
        clear::clear,
        count::count,
        enumerate::enumerate,
        join::join,
        rev::rev,
        set::{global, local},
        split::split,
        trim::trim,
        ty::ty,
        use_alias::UseAlias,
    },
    util::lookahead_parse::{LookaheadParse, lookahead_parse},
};

pub use self::call::{Call, ToCallable};

mod macros;

mod call;

function_enum!(
    /// Enum collecting [Call] implementors.
    #[derive(Debug, Clone)]
    Function {
        /// Split array according to specification
        Split(split),
        /// Join array according to specification.
        Join(join),
        /// Case array according to specification.
        Case(case),
        /// Convert type of array.
        Type(ty),
        /// Enumerate array.
        Enumerate(enumerate),
        /// Reverse array.
        Rev(rev),
        /// Trim array elements.
        Trim(trim),
        /// Count array elements.
        Count(count),
        /// Clear array.
        Clear(clear),
        /// Set a global variable.
        Global(global),
        /// Set a local variable.
        Local(local),
        /// Set an alias.
        Alias(alias),
        /// Use an alias.
        UseAlias(UseAlias),
    }
);

impl Function {
    /// Parse a function chain with a custom termination condition.
    ///
    /// # Note
    /// If termination condition never returns true
    /// this funtion may loop forever.
    ///
    /// # Errors
    /// On incorrect syntax.
    pub fn parse_chain_terminated(
        input: ParseStream,
        should_terminate: fn(&Lookahead1) -> bool,
    ) -> ::syn::Result<Vec<(Option<Token![.]>, Self)>> {
        let lookahead = input.lookahead1();
        let mut chain = Vec::new();

        if should_terminate(&lookahead) {
            return Ok(chain);
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

        Ok(chain)
    }

    /// Parse a function chain.
    ///
    /// # Errors
    /// On incorrect syntax.
    #[inline]
    pub fn parse_chain(input: ParseStream) -> ::syn::Result<Vec<(Option<Token![.]>, Self)>> {
        Self::parse_chain_terminated(input, |lookahead| lookahead.peek(End))
    }
}
