//! [JoinArgs] impl.

use ::file_suite_proc_lib::FromArg;

use crate::{
    array_expr::{
        function::{Call, DefaultArgs, ToCallable},
        storage::Storage,
        typed_value::TypedValue,
        value_array::ValueArray,
    },
    util::kw_kind,
};

/// Join values by a string.
#[derive(Debug, Clone)]
pub struct JoinByCallable {
    /// String to join values by.
    by: String,
}

impl FromArg for JoinByCallable {
    type Factory = TypedValue;

    fn from_arg(by: String) -> Self {
        Self { by }
    }
}

impl DefaultArgs for JoinByCallable {
    fn default_args() -> Self {
        Self { by: String::new() }
    }
}

impl Call for JoinByCallable {
    fn call(&self, array: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        Ok(array.join_by_str(&self.by))
    }
}

kw_kind!(
    /// Keyword specified join.
    JoinArgs;
    /// Enum of possible values for [JoinKw].
    #[expect(non_camel_case_types)]
    JoinKind: Default {
        #[default]
        /// Concat values.
        concat,
        /// Join by dashes '-'.
        kebab,
        /// Join by underscores '_'.
        snake,
        /// Join by double colons '::'.
        path,
        /// Join by spaces ' '.
        space,
        /// Join by dots '.'.
        dot,
    }
);

impl ToCallable for JoinArgs {
    type Call = JoinKind;

    fn to_callable(&self) -> Self::Call {
        self.kind
    }
}

impl DefaultArgs for JoinKind {
    fn default_args() -> Self {
        Self::concat
    }
}

impl Call for JoinKind {
    fn call(&self, input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        Ok(match self {
            JoinKind::concat => input.join_by_str(""),
            JoinKind::kebab => input.join_by_str("-"),
            JoinKind::snake => input.join_by_str("_"),
            JoinKind::path => input.join_by_str("::"),
            JoinKind::space => input.join_by_str(" "),
            JoinKind::dot => input.join_by_str("."),
        })
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    fn join_ints() {
        assert_arr_expr!(
            {1 0 0 0 -> join.ty(int)},
            {1000},
        );
        assert_arr_expr!(
            {
                snake -> global(conv),
                uses snake case -> join(=conv).ty(ident),
            },
            {
                uses_snake_case
            },
        );
    }

    #[test]
    fn join_by() {
        assert_arr_expr!(
            { joined by commas -> join_by(", ").ty(str) },
            { "joined, by, commas" },
        );

        assert_arr_expr!(
            {
                "." -> global(by),
                joined by dots -> join_by(=by).ty(str),
            },
            { "joined.by.dots" }
        );
    }
}
