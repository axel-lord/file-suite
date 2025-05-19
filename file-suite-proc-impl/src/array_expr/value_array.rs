//! [ValueArray] impl.

use ::std::{
    iter, mem,
    ops::{Deref, DerefMut},
    option, slice, vec,
};

use crate::value::Value;

/// An array of values.
#[derive(Debug, Clone, Default)]
#[repr(transparent)]
pub struct ValueArray {
    /// Internal state, hidden.
    inner: ValueArrayInner,
}

/// Implementation detail to hide variants.
#[derive(Debug, Clone, Default)]
enum ValueArrayInner {
    /// Value array is empty.
    #[default]
    Empty,
    /// Value array has a single element.
    Single(Value),
    /// Value array is a vector of any amount of elements (includes none and one element).
    Vec(Vec<Value>),
}

/// Owned iterator for [ValueArray].
pub type IntoIter = iter::Chain<option::IntoIter<Value>, vec::IntoIter<Value>>;

impl ValueArray {
    /// Create a new instance.
    const fn new_inner(inner: ValueArrayInner) -> Self {
        Self { inner }
    }

    /// Create a new empty instance.
    pub const fn new() -> Self {
        Self::new_inner(ValueArrayInner::Empty)
    }

    /// Create a new instance from a single value.
    pub const fn from_value(value: Value) -> Self {
        Self::new_inner(ValueArrayInner::Single(value))
    }

    /// Creat a new instance from a vec of values.
    pub const fn from_vec(value: Vec<Value>) -> Self {
        Self::new_inner(ValueArrayInner::Vec(value))
    }

    /// Converts internals to a [Vec] and returns a mutable
    /// reference to it.
    pub fn make_vec(&mut self) -> &mut Vec<Value> {
        self.inner = ValueArrayInner::Vec(match mem::take(&mut self.inner) {
            ValueArrayInner::Empty => Vec::new(),
            ValueArrayInner::Single(value) => Vec::from([value]),
            ValueArrayInner::Vec(values) => values,
        });

        if let ValueArrayInner::Vec(vec) = &mut self.inner {
            vec
        } else {
            unreachable!()
        }
    }
}

impl From<Value> for ValueArray {
    fn from(value: Value) -> Self {
        Self::from_value(value)
    }
}

impl From<Vec<Value>> for ValueArray {
    fn from(value: Vec<Value>) -> Self {
        Self::from_vec(value)
    }
}

impl From<ValueArray> for Vec<Value> {
    fn from(value: ValueArray) -> Self {
        match value.inner {
            ValueArrayInner::Empty => Vec::new(),
            ValueArrayInner::Single(value) => Vec::from([value]),
            ValueArrayInner::Vec(values) => values,
        }
    }
}

impl AsRef<[Value]> for ValueArray {
    fn as_ref(&self) -> &[Value] {
        match &self.inner {
            ValueArrayInner::Empty => &[],
            ValueArrayInner::Single(value) => ::std::slice::from_ref(value),
            ValueArrayInner::Vec(values) => values,
        }
    }
}

impl AsMut<[Value]> for ValueArray {
    fn as_mut(&mut self) -> &mut [Value] {
        match &mut self.inner {
            ValueArrayInner::Empty => &mut [],
            ValueArrayInner::Single(value) => ::std::slice::from_mut(value),
            ValueArrayInner::Vec(values) => values,
        }
    }
}

impl Deref for ValueArray {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for ValueArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl IntoIterator for ValueArray {
    type Item = Value;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self.inner {
            ValueArrayInner::Empty => None.into_iter().chain(Vec::new()),
            ValueArrayInner::Single(value) => Some(value).into_iter().chain(Vec::new()),
            ValueArrayInner::Vec(values) => None.into_iter().chain(values),
        }
    }
}

impl<'i> IntoIterator for &'i ValueArray {
    type Item = &'i Value;

    type IntoIter = slice::Iter<'i, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'i> IntoIterator for &'i mut ValueArray {
    type Item = &'i mut Value;

    type IntoIter = slice::IterMut<'i, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl FromIterator<Value> for ValueArray {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        let mut iter = iter.into_iter();

        let Some(first) = iter.next() else {
            return Self::new();
        };

        let Some(second) = iter.next() else {
            return Self::from_value(first);
        };

        let mut vec = vec![first, second];
        vec.extend(iter);

        Self::from_vec(vec)
    }
}
