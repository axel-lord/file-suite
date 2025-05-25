//! [ValueArray] impl.

use ::std::{
    iter, mem,
    ops::{Deref, DerefMut},
    option, slice, vec,
};

use ::proc_macro2::Span;

use crate::array_expr::value::Value;

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

    /// Push a value onto the array. When used multiple times
    /// [make_vec][ValueArray::make_vec] then pushing to the result may
    /// be more efficient.
    pub fn push(&mut self, value: Value) {
        if self.is_empty() {
            self.inner = ValueArrayInner::Single(value);
        } else {
            self.make_vec().push(value);
        }
    }

    /// Join a ValueArray by a string slice.
    pub fn join_by_str(self, sep: &str) -> Self {
        if self.len() <= 1 {
            return self;
        }

        let mut value = Value::default();
        if let Some(last) = self.last() {
            value.ty = last.ty;
        }

        for span in self.iter().filter_map(Value::span) {
            value.push_span(span);
        }

        *value.make_string() = self.join(sep);

        Self::from_value(value)
    }

    /// Split a ValueArray by a string slice.
    pub fn split_by_str(&self, pat: &str) -> Self {
        if self.is_empty() {
            return ValueArray::new();
        }

        let mut vec = Vec::with_capacity(self.len());

        for value in self {
            for content in value.split(pat) {
                vec.push(
                    Value::new(content.into())
                        .with_span_of(value)
                        .with_ty_of(value),
                );
            }
        }

        Self::from_vec(vec)
    }

    /// Get combined (if any, or possible) span of values.
    pub fn span(&self) -> Option<Span> {
        let mut span = None;

        for value in self {
            let Some(new_span) = value.span() else {
                continue;
            };
            let Some(current_span) = span else {
                span = Some(new_span);
                continue;
            };
            let Some(joined) = current_span.join(new_span) else {
                continue;
            };
            span = Some(joined);
        }

        span
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

impl Extend<Value> for ValueArray {
    fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
        let mut iter = iter.into_iter();

        if let Some(value) = iter.next() {
            self.push(value);
        }

        if let Some(value) = iter.next() {
            let vec = self.make_vec();
            vec.push(value);
            vec.extend(iter);
        };
    }
}
