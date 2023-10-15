use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
    Deserializer,
    ser::SerializeMap,
    Serialize,
    Serializer,
};

pub use crate::collections::blazemap::{
    entry::{Entry, OccupiedEntry, VacantEntry},
    iter::{Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut},
};
use crate::collections::blazemap::entry::VacantEntryInner;
use crate::id_wrapper::IdWrapper;

mod entry;
mod iter;

/// A [`Vec`]-based analogue of a [`HashMap`](std::collections::HashMap).
#[derive(Clone, PartialEq, Eq)]
pub struct BlazeMap<K, V>
{
    inner: Vec<Option<V>>,
    len: usize,
    phantom: PhantomData<K>,
}

impl<K, V> BlazeMap<K, V>
{
    /// Creates a new instance of the [`BlazeMap`].
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: vec![],
            len: 0,
            phantom: Default::default(),
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.len = 0
    }

    /// Shrinks the capacity of the map as much as possible.
    /// It will drop down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let elems_to_crop = self.inner.iter()
            .rev()
            .position(Option::is_some)
            .unwrap_or(0);
        self.inner.truncate(self.inner.len() - elems_to_crop);
        self.inner.shrink_to_fit()
    }

    /// An iterator visiting all key-value pairs. The iterator element type is `(K, &'a V)`.
    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            inner: self.inner.as_ptr(),
            current_position: 0,
            len: self.len,
            phantom: Default::default(),
        }
    }

    /// An iterator visiting all key-value pairs, with mutable references to the values.
    /// The iterator element type is `(K, &'a mut V)`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            inner: self.inner.as_mut_ptr(),
            current_position: 0,
            len: self.len,
            phantom: Default::default(),
        }
    }

    /// An iterator visiting all keys. The iterator element type is `K`.
    #[inline]
    pub fn keys(&self) -> Keys<K, V> {
        Keys {
            inner: self.iter()
        }
    }

    /// An iterator visiting all values. The iterator element type is `&'a V`.
    #[inline]
    pub fn values(&self) -> Values<K, V> {
        Values {
            inner: self.iter()
        }
    }

    /// An iterator visiting all values mutably. The iterator element type is `&'a mut V`.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut {
            inner: self.iter_mut()
        }
    }

    /// Clears the map, returning all key-value pairs as an iterator.
    /// Keeps the allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed,
    /// it drops the remaining key-value pairs.
    /// The returned iterator keeps a mutable borrow on the map to optimize its implementation.
    #[inline]
    pub fn drain(&mut self) -> Drain<K, V> {
        Drain {
            inner: self.iter_mut()
        }
    }
}

impl<K, V> BlazeMap<K, V>
    where
        K: IdWrapper
{
    /// Creates a new instance of the [`BlazeMap`]
    /// with capacity equal to the current total number of unique `K` instances.
    #[inline]
    pub fn with_current_key_wrapper_capacity() -> Self {
        let current_capacity = K::static_info()
            .read()
            .num_elems();
        Self {
            inner: Vec::with_capacity(current_capacity),
            len: 0,
            phantom: Default::default(),
        }
    }

    /// Returns `true` if the map contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        self.inner
            .get(key.get_index())
            .and_then(Option::as_ref)
            .is_some()
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: K) -> Option<&V> {
        self.inner
            .get(key.get_index())
            .and_then(Option::as_ref)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner
            .get_mut(key.get_index())
            .and_then(Option::as_mut)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    /// The key is not updated, though.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
            Entry::Occupied(mut entry) => {
                Some(entry.insert(value))
            }
        }
    }

    /// Removes a key from the map,
    /// returning the value at the key if the key was previously in the map.
    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        if let Entry::Occupied(entry) = self.entry(key) {
            Some(entry.remove())
        } else {
            None
        }
    }

    /// Gets the given key’s corresponding entry in the map for in-place manipulation.
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        let index = key.get_index();
        if index < self.inner.len() {
            let value = unsafe { self.inner.get_unchecked_mut(index) };
            if value.is_some() {
                let occupied = OccupiedEntry {
                    key,
                    len: &mut self.len,
                    value,
                };
                Entry::Occupied(occupied)
            } else {
                let vacant = VacantEntry {
                    key,
                    len: &mut self.len,
                    inner: VacantEntryInner::ShouldBeInserted(value),
                };
                Entry::Vacant(vacant)
            }
        } else {
            let vacant = VacantEntry {
                key,
                len: &mut self.len,
                inner: VacantEntryInner::ShouldBeEnlarged(&mut self.inner),
            };
            Entry::Vacant(vacant)
        }
    }

    /// Creates a consuming iterator visiting all the keys.
    /// The map cannot be used after calling this. The iterator element type is `K`.
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys {
            inner: self.into_iter()
        }
    }

    /// Creates a consuming iterator visiting all the values.
    /// The map cannot be used after calling this. The iterator element type is `V`.
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues {
            inner: self.into_iter()
        }
    }
}

impl<K, V> IntoIterator for BlazeMap<K, V>
    where
        K: IdWrapper
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            inner: self
        }
    }
}

impl<'a, K, V> IntoIterator for &'a BlazeMap<K, V>
    where
        K: IdWrapper
{
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut BlazeMap<K, V>
    where
        K: IdWrapper
{
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }
}

impl<K, V> FromIterator<(K, V)> for BlazeMap<K, V>
    where
        K: IdWrapper
{
    #[inline]
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> Self {
        let mut result = BlazeMap::with_current_key_wrapper_capacity();
        iter.into_iter()
            .for_each(|(key, value)| { result.insert(key, value); });
        result
    }
}

impl<K, V> Default for BlazeMap<K, V>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! blaze_map_orig_key_blocking_iter {
    ($self:ident, $iter:ident, $guard:ident) => {
        let $guard = K::static_info().read();
        let $iter = $self.inner.iter()
            .enumerate()
            .filter_map(|(idx, value)| Some((idx, value.as_ref()?)))
            .map(
                |(idx, value)| {
                    let key = unsafe { $guard.get_key_unchecked(idx) };
                    (key, value)
                }
            );
    }
}

impl<K, V> Debug for BlazeMap<K, V>
    where
        K: IdWrapper,
        <K as IdWrapper>::OrigType: Debug,
        V: Debug
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        blaze_map_orig_key_blocking_iter!(self, iter, guard);
        f.debug_map().entries(iter).finish()
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for BlazeMap<K, V>
    where
        K: IdWrapper,
        <K as IdWrapper>::OrigType: Serialize,
        V: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        blaze_map_orig_key_blocking_iter!(self, iter, guard);
        let mut serializer = serializer.serialize_map(Some(self.len))?;
        for (key, value) in iter {
            serializer.serialize_entry(key, value)?;
        }
        serializer.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for BlazeMap<K, V>
    where
        K: IdWrapper,
        <K as IdWrapper>::OrigType: Deserialize<'de>,
        V: Deserialize<'de>
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
    {
        deserializer.deserialize_map(BlazeMapDeserializer(Default::default()))
    }
}

#[cfg(feature = "serde")]
struct BlazeMapDeserializer<K, V>(PhantomData<(K, V)>)
    where
        K: IdWrapper;

#[cfg(feature = "serde")]
impl<'de, K, V> Visitor<'de> for BlazeMapDeserializer<K, V>
    where
        K: IdWrapper,
        <K as IdWrapper>::OrigType: Deserialize<'de>,
        V: Deserialize<'de>
{
    type Value = BlazeMap<K, V>;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "BlazeMap-compatible map")
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>
    {
        let mut result = BlazeMap::with_current_key_wrapper_capacity();

        while let Some((key, value)) = map.next_entry::<K::OrigType, V>()? {
            let key = K::new(key);
            result.insert(key, value);
        }
        result.shrink_to_fit();
        Ok(result)
    }
}