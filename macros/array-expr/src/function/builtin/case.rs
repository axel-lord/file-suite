//! [CaseArgs] impl.

use ::file_suite_proc_lib::kw_kind;

use crate::{
    from_values::{FromValues, ensure_single},
    function::Call,
    storage::Storage,
    value_array::ValueArray,
};

kw_kind!(
    /// Casing to apply.
    CaseArgs;
    /// Enum containing possible values for [CaseArgs]
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

impl FromValues for CaseKind {
    fn from_values(values: &[crate::value::Value]) -> crate::Result<Self> {
        Ok(ensure_single(values)?.parse()?)
    }
}

impl Call for CaseKind {
    fn call(&self, mut input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
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

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::test::assert_arr_expr;

    #[test]
    fn case_convert() {
        assert_arr_expr!(
            { "from-kebab-to-camel" -> split(kebab).case(camel).join.ty(ident) },
            { fromKebabToCamel },
        );

        assert_arr_expr!(
            { CamelToSnake -> split(camel).case(lower).join(snake).ty(ident) },
            { _camel_to_snake },
        );

        assert_arr_expr!(
            { pascal -> global(case), pascal "case" -> case(=case).join.ty(ident) },
            { PascalCase },
        );
    }
}
