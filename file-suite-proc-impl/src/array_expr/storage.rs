//! Variable storage.

use ::std::borrow::Cow;

use crate::array_expr::value_array::ValueArray;

/// Stored value.
#[derive(Debug, Clone, Default)]
struct StoredValue {
    /// Value key.
    key: String,
    /// Value is read only.
    read_only: bool,
    /// Values.
    values: ValueArray,
}

/// Storage for variables.
#[derive(Debug, Default)]
pub struct Storage {
    /// Values of storage.
    globals: Vec<StoredValue>,
    /// Local values.
    locals: Vec<Vec<StoredValue>>,
}

/// Insert a value into a vec.
///
/// # Errors
/// If the variable is read-only the key is returned.
fn insert<'a, 'k>(
    values: &'a mut Vec<StoredValue>,
    key: Cow<'k, str>,
    read_only: bool,
) -> Result<&'a mut ValueArray, Cow<'k, str>> {
    let index = values.iter().position(|value| value.key == key.as_ref());
    match index {
        Some(index) => {
            let Some(value) = values.get_mut(index).filter(|value| !value.read_only) else {
                return Err(key);
            };
            value.read_only = read_only;
            Ok(&mut value.values)
        }
        None => {
            let index = values.len();
            let value = StoredValue {
                key: key.into_owned(),
                read_only: false,
                ..Default::default()
            };
            values.push(value);
            let value = &mut values[index];
            value.read_only = read_only;
            Ok(&mut value.values)
        }
    }
}

impl Storage {
    /// Use storage with a new layer for local variables.
    /// The layer will be removed upon return.
    pub fn with_local_layer<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Storage) -> T,
    {
        self.locals.push(Vec::new());
        let value = f(self);
        self.locals.pop();
        value
    }

    /// Insert into the furthest backing storage.
    ///
    /// # Errors
    /// If the variable is read-only the key is returned.
    pub fn insert_global<'this, 'key>(
        &'this mut self,
        key: Cow<'key, str>,
        read_only: bool,
    ) -> Result<&'this mut ValueArray, Cow<'key, str>> {
        insert(&mut self.globals, key, read_only)
    }

    /// Insert a value into the storage if possible.
    ///
    /// # Errors
    /// If the variable is read-only the key is returned.
    pub fn insert<'this, 'key>(
        &'this mut self,
        key: Cow<'key, str>,
        read_only: bool,
    ) -> Result<&'this mut ValueArray, Cow<'key, str>> {
        let Self { globals, locals } = self;
        let vars = locals.last_mut().unwrap_or(globals);

        insert(vars, key, read_only)
    }

    /// Get a value from the storage.
    pub fn get<'this>(&'this self, key: &'_ str) -> Option<&'this ValueArray> {
        self.locals
            .iter()
            .rev()
            .flatten()
            .chain(&self.globals)
            .find(|value| value.key == key)
            .map(|value| &value.values)
    }
}
