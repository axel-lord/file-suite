//! [case] impl.

use std::borrow::Cow;

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{group_help::GroupSingle, kw_kind, lookahead_parse::ParseWrap},
};

kw_kind!(
    /// Casing to apply.
    Spec;
    /// Enum containing possible values for [Spec]
    #[expect(non_camel_case_types)]
    CaseKind {
        /// Convert to PascalCase.
        pascal,
        /// Convert to camelCase.
        camel,
        /// Convert to UPPERCASE.
        upper,
        /// Convert to LOWERCASE.
        lower,
    }
);

function_struct!(
    /// Apply case to input.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    case {
        /// Specification for which case to apply.
        spec: GroupSingle<ParseWrap<Spec>>,
    }
);

impl ToCallable for case {
    type Call = CaseKind;

    fn to_callable(&self) -> Self::Call {
        self.spec.content.0.kind
    }
}

impl Call for CaseKind {
    fn call(
        &self,
        mut input: ValueArray,
        _: &mut Storage,
    ) -> Result<ValueArray, Cow<'static, str>> {
        /// Get capitalized version of a string slice.
        fn capitalize(value: &str) -> String {
            let mut chars = value.chars();
            let mut capitalized = String::with_capacity(value.len());

            if let Some(first) = chars.next() {
                capitalized.extend(first.to_uppercase());
            }

            for chr in chars {
                capitalized.extend(chr.to_lowercase());
            }

            capitalized
        }

        match self {
            CaseKind::pascal => {
                for value in &mut input {
                    let mut capitalized = capitalize(value);
                    capitalized.shrink_to_fit();
                    *value.make_string() = capitalized;
                }

                Ok(input)
            }
            CaseKind::camel => {
                let mut values = input.iter_mut();

                if let Some(first) = values.next() {
                    *first.make_string() = first.to_lowercase();
                }

                for value in values {
                    *value.make_string() = capitalize(value);
                }

                Ok(input)
            }
            CaseKind::upper => {
                for value in &mut input {
                    *value.make_string() = value.to_uppercase();
                }

                Ok(input)
            }
            CaseKind::lower => {
                for value in &mut input {
                    *value.make_string() = value.to_lowercase();
                }

                Ok(input)
            }
        }
    }
}
