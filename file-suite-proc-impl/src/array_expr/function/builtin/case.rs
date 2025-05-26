//! [case] impl.

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        storage::Storage,
        value_array::ValueArray,
    },
    util::kw_kind,
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

impl ToCallable for CaseArgs {
    type Call = CaseKind;

    fn to_callable(&self) -> Self::Call {
        self.kind
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
