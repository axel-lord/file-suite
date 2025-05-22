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
fn insert<'a>(
    values: &'a mut Vec<StoredValue>,
    key: Cow<'_, str>,
    read_only: bool,
) -> Option<&'a mut ValueArray> {
    let index = values.iter().position(|value| value.key == key.as_ref());
    let index = match index {
        Some(index) => index,
        None => {
            let index = values.len();
            let value = StoredValue {
                key: key.into_owned(),
                read_only: false,
                ..Default::default()
            };
            values.push(value);
            index
        }
    };

    let value = values.get_mut(index).filter(|value| !value.read_only)?;
    value.read_only = read_only;

    Some(&mut value.values)
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
    pub fn insert_global<'this>(
        &'this mut self,
        key: Cow<'_, str>,
        read_only: bool,
    ) -> Option<&'this mut ValueArray> {
        insert(&mut self.globals, key, read_only)
    }

    /// Insert a value into the storage if possible.
    pub fn insert<'this>(
        &'this mut self,
        key: Cow<'_, str>,
        read_only: bool,
    ) -> Option<&'this mut ValueArray> {
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
