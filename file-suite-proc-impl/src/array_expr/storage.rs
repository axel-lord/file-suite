//! Variable storage.

use ::std::{borrow::Cow, rc::Rc};

use crate::array_expr::{
    function::{Function, ToCallable},
    value_array::ValueArray,
};

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

/// Stored alias.
#[derive(Debug, Clone)]
struct StoredAlias {
    /// Alias key.
    key: String,
    /// Alias chain.
    chain: Rc<[<Function as ToCallable>::Call]>,
}

/// Storage for variables.
#[derive(Debug)]
pub struct Storage {
    /// Variables.
    variables: Vec<Vec<StoredValue>>,
    /// Chain aliases.
    aliases: Vec<StoredAlias>,
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
    /// Get a new empty storage.
    pub fn initial() -> Self {
        Self {
            variables: Vec::from([Vec::new()]),
            aliases: Vec::new(),
        }
    }

    /// Use storage with a new layer for local variables.
    /// The layer will be removed upon return.
    pub fn with_local_layer<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Storage) -> T,
    {
        self.variables.push(Vec::new());
        let value = f(self);
        self.variables.pop();
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
        insert(
            self.variables.first_mut().unwrap_or_else(|| unreachable!()),
            key,
            read_only,
        )
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
        let Self {
            variables,
            aliases: _,
        } = self;
        let vars = variables.last_mut().unwrap_or_else(|| unreachable!());

        insert(vars, key, read_only)
    }

    /// Get a value from the storage.
    ///
    /// # Errors
    /// If the variable cannot be found a [NoVar][crate::Error::NoVar] error is returned.
    pub fn try_get<'this>(&'this self, key: &'_ str) -> crate::Result<&'this ValueArray> {
        self.variables
            .iter()
            .rev()
            .flatten()
            .find(|value| value.key == key)
            .map(|value| &value.values)
            .ok_or_else(|| crate::Error::NoVar(key.into()))
    }

    /// Get a value from the storage.
    pub fn get<'this>(&'this self, key: &'_ str) -> Option<&'this ValueArray> {
        self.variables
            .iter()
            .rev()
            .flatten()
            .find(|value| value.key == key)
            .map(|value| &value.values)
    }

    /// Set an alias.
    pub fn set_alias(&mut self, key: String, chain: Vec<<Function as ToCallable>::Call>) {
        for alias in &mut self.aliases {
            if alias.key == key {
                alias.chain = chain.into();
                return;
            }
        }
        self.aliases.push(StoredAlias {
            key,
            chain: chain.into(),
        });
    }

    /// Get an alias.
    pub fn get_alias(&self, key: &str) -> Option<Rc<[<Function as ToCallable>::Call]>> {
        self.aliases.iter().find_map(|alias| {
            if alias.key == key {
                Some(alias.chain.clone())
            } else {
                None
            }
        })
    }
}
