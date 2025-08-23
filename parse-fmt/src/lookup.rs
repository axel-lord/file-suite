//! Functions to create lookup closures for some common container types.

use ::std::{
    borrow::{Borrow, Cow},
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
    ops::Range,
};

use crate::display_bytes;

/// Create a lookup function combining multiple other lookup functions and returning
/// the first successfull result.
pub fn either<'a, EA, EB>(
    mut a: impl 'a + FnMut(&'a [u8]) -> Result<Cow<'a, [u8]>, EA>,
    mut b: impl 'a + FnMut(&'a [u8]) -> Result<Cow<'a, [u8]>, EB>,
) -> impl 'a + FnMut(&'a [u8]) -> Result<Cow<'a, [u8]>, (EA, EB)> {
    move |key| match a(key) {
        Ok(val) => Ok(val),
        Err(ea) => match b(key) {
            Ok(val) => Ok(val),
            Err(eb) => Err((ea, eb)),
        },
    }
}

/// Create a lookup function using a backing hash map.
///
/// The lookup functions returns the key as the error variant.
pub fn hash_map<'a, K, V, B>(
    map: &'a HashMap<K, V, B>,
) -> impl 'a + Fn(&'a [u8]) -> Result<Cow<'a, [u8]>, &'a [u8]>
where
    K: Borrow<[u8]> + Hash + Eq,
    V: AsRef<[u8]>,
    B: BuildHasher,
{
    move |key| {
        map.get(key)
            .map(|value| Cow::Borrowed(value.as_ref()))
            .ok_or(key)
    }
}

/// Create a lookup function using a backing btree map.
///
/// The lookup functions returns the key as the error variant.
pub fn btree_map<'a, K, V>(
    map: &'a BTreeMap<K, V>,
) -> impl 'a + Fn(&'a [u8]) -> Result<Cow<'a, [u8]>, &'a [u8]>
where
    K: Borrow<[u8]> + Ord,
    V: AsRef<[u8]>,
{
    move |key| {
        map.get(key)
            .map(|value| Cow::Borrowed(value.as_ref()))
            .ok_or(key)
    }
}

/// Create a lookup function using a backing iterable collection.
///
/// The lookup functions returns the key as the error variant.
pub fn seq_map<'a, I, K, V>(map: &'a I) -> impl 'a + Fn(&'a [u8]) -> Result<Cow<'a, [u8]>, &'a [u8]>
where
    &'a I: IntoIterator<Item = &'a (K, V)>,
    K: 'a + Borrow<[u8]>,
    V: 'a + AsRef<[u8]>,
{
    move |key| {
        map.into_iter()
            .find_map(|(k, v)| {
                if k.borrow() == key {
                    Some(Cow::Borrowed(v.as_ref()))
                } else {
                    None
                }
            })
            .ok_or(key)
    }
}

/// Error returned when sequence lookup fails.
#[derive(Debug, ::thiserror::Error, PartialEq, Eq)]
pub enum SeqLookupError<'a> {
    /// Could not parse lookup key as an integer, if not empty.
    #[error("could not parse `{1}` as an integer: {0}")]
    ParseIndex(#[source] ::std::num::ParseIntError, &'a str),
    /// Could not convert lookup key to utf-8.
    #[error("key `{}` is not utf-8: {}", display_bytes(.1), .0)]
    IndexNotUtf8(#[source] ::std::str::Utf8Error, &'a [u8]),
    /// Index was out of range for slice.
    #[error("index {0} is outside valid range for lookup {0:?}")]
    OutOfRange(isize, Range<isize>),
}

impl Hash for SeqLookupError<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            SeqLookupError::ParseIndex(_, key) => key.hash(state),
            SeqLookupError::IndexNotUtf8(_, items) => items.hash(state),
            SeqLookupError::OutOfRange(idx, range) => {
                isize::hash_slice(&[*idx, range.start, range.end], state)
            }
        }
    }
}

/// Create a lookup function using a backing iterable collection, accepting indices and empty
/// groups.
pub fn seq<'a, V>(
    map: &'a [V],
) -> impl 'a + FnMut(&'a [u8]) -> Result<Cow<'a, [u8]>, SeqLookupError<'a>>
where
    V: 'a + AsRef<[u8]>,
{
    let len = map.len() as isize;
    let range = -len..len;
    let mut idx_counter = 0isize;
    move |key| {
        let idx = if key.is_empty() {
            let idx = idx_counter;
            idx_counter += 1;
            idx
        } else {
            let key = str::from_utf8(key).map_err(|err| SeqLookupError::IndexNotUtf8(err, key))?;
            key.parse::<isize>()
                .map_err(|err| SeqLookupError::ParseIndex(err, key))?
        };

        let uidx = if idx < range.start {
            return Err(SeqLookupError::OutOfRange(idx, range.clone()));
        } else if idx < 0 {
            (range.end + idx) as usize
        } else {
            idx as usize
        };

        map.get(uidx)
            .ok_or_else(|| SeqLookupError::OutOfRange(idx, range.clone()))
            .map(|value| Cow::Borrowed(value.as_ref()))
    }
}
