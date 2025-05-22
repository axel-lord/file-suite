//! Variable storage.

use ::std::borrow::Cow;

use crate::array_expr::value_array::ValueArray;

/// Stored value.
#[derive(Debug, Clone)]
struct Stored {
    /// Value key.
    key: String,
    /// Value is read only.
    read_only: bool,
    /// Values.
    values: ValueArray,
}

/// Storage for variables.
#[derive(Debug, Clone, Default)]
pub struct Storage {
    /// Values of storage.
    values: Vec<Stored>,
}

impl Storage {
    /// Insert a value into the storage if possible.
    pub fn insert(&mut self, key: Cow<str>, read_only: bool) -> Option<&mut ValueArray> {
        let idx = self.values.iter().position(
            |Stored {
                 key: stored_key, ..
             }| { stored_key == key.as_ref() },
        );

        let idx = if let Some(idx) = idx {
            idx
        } else {
            self.values.push(Stored {
                key: key.into_owned(),
                read_only,
                values: ValueArray::new(),
            });
            self.values.len() - 1
        };

        let value = self.values.get_mut(idx).filter(|s| !s.read_only)?;
        value.read_only = read_only;

        Some(&mut value.values)
    }

    /// Get a value from the storage.
    pub fn get(&self, key: &str) -> Option<&ValueArray> {
        self.values
            .iter()
            .find(|val| val.key == key)
            .map(|val| &val.values)
    }
}
