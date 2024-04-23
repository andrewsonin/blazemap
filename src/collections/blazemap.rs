use std::{
    borrow::Borrow,
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

#[cfg(feature = "serde")]
use {
    crate::prelude::BlazeMapIdWrapper,
    serde::{
        de::{MapAccess, Visitor},
        ser::SerializeMap,
        Deserialize, Deserializer, Serialize, Serializer,
    },
};

pub use crate::collections::blazemap::{
    entries::{Entry, OccupiedEntry, VacantEntry},
    iters::{Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut},
};
use crate::{
    collections::blazemap::entries::VacantEntryInner,
    traits::{
        BlazeMapId, BlazeMapIdStatic, CapacityInfoProvider, KeyByOffsetProvider, TypeInfoContainer,
    },
};

mod entries;
mod iters;

/// A [`Vec`]-based analogue of a [`HashMap`](std::collections::HashMap).
#[derive(Clone, PartialEq, Eq)]
pub struct BlazeMap<K, V> {
    pub(in crate::collections::blazemap) inner: Vec<Option<V>>,
    pub(in crate::collections::blazemap) len: usize,
    phantom: PhantomData<K>,
}

impl<K, V> BlazeMap<K, V> {
    /// Creates a new instance of the [`BlazeMap`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: vec![],
            len: 0,
            phantom: PhantomData,
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no elements.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.len = 0;
    }

    /// Shrinks the capacity of the map as much as possible.
    /// It will drop down as much as possible while maintaining the internal
    /// rules and possibly leaving some space in accordance with the resize
    /// policy.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        if !self.is_empty() {
            let elems_to_crop = self
                .inner
                .iter()
                .rev()
                .position(Option::is_some)
                .unwrap_or(0);
            self.inner.truncate(self.inner.len() - elems_to_crop);
        }
        self.inner.shrink_to_fit();
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
    }

    /// Clears the map, returning all key-value pairs as an iterator.
    /// Keeps the allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed,
    /// it drops the remaining key-value pairs.
    /// The returned iterator keeps a mutable borrow on the map to optimize its
    /// implementation.
    #[inline]
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        Drain {
            map: self,
            current_position: 0,
        }
    }
}

impl<K, V> BlazeMap<K, V>
where
    K: BlazeMapId,
{
    /// An iterator visiting all key-value pairs. The iterator element type is
    /// `(K, &V)`.
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        Iter {
            inner: self.inner.as_ptr(),
            current_position: 0,
            len: self.len,
            phantom: PhantomData,
        }
    }

    /// An iterator visiting all key-value pairs, with mutable references to the
    /// values. The iterator element type is `(K, &mut V)`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        IterMut {
            inner: self.inner.as_mut_ptr(),
            current_position: 0,
            len: self.len,
            phantom: PhantomData,
        }
    }

    /// An iterator visiting all keys. The iterator element type is `K`.
    #[inline]
    #[must_use]
    pub fn keys(&self) -> Keys<'_, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all values. The iterator element type is `&V`.
    #[inline]
    #[must_use]
    pub fn values(&self) -> Values<'_, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        Values { inner: self.iter() }
    }

    /// An iterator visiting all values mutably. The iterator element type is
    /// `&mut V`.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        ValuesMut {
            inner: self.iter_mut(),
        }
    }
}

impl<K, V> BlazeMap<K, V>
where
    K: BlazeMapIdStatic,
{
    /// Creates a new instance of the [`BlazeMap`]
    /// with capacity equal to the current total number of unique `K` instances.
    #[inline]
    #[must_use]
    pub fn with_current_key_type_capacity() -> Self {
        let current_capacity = K::static_container()
            .capacity_info_provider()
            .offset_capacity();
        Self {
            inner: Vec::with_capacity(current_capacity),
            len: 0,
            phantom: PhantomData,
        }
    }
}

impl<K, V> BlazeMap<K, V>
where
    K: BlazeMapId,
{
    /// Returns `true` if the map contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        self.inner
            .get(key.get_offset())
            .and_then(Option::as_ref)
            .is_some()
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: K) -> Option<&V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        self.inner.get(key.get_offset()).and_then(Option::as_ref)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        self.inner
            .get_mut(key.get_offset())
            .and_then(Option::as_mut)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        let result = match self.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
            Entry::Occupied(mut entry) => Some(entry.insert(value)),
        };
        result
    }

    /// Removes a key from the map,
    /// returning the value at the key if the key was previously in the map.
    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        let result = if let Entry::Occupied(entry) = self.entry(key) {
            Some(entry.remove())
        } else {
            None
        };
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        result
    }

    /// Gets the given key’s corresponding entry in the map for in-place
    /// manipulation.
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        let offset = key.get_offset();
        if offset < self.inner.len() {
            let value = unsafe { self.inner.get_unchecked_mut(offset) };
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
    /// The map cannot be used after calling this. The iterator element type is
    /// `K`.
    #[inline]
    #[must_use]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        IntoKeys {
            inner: self.into_iter(),
        }
    }

    /// Creates a consuming iterator visiting all the values.
    /// The map cannot be used after calling this. The iterator element type is
    /// `V`.
    #[inline]
    #[must_use]
    pub fn into_values(self) -> IntoValues<K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        IntoValues {
            inner: self.into_iter(),
        }
    }
}

impl<K, V> IntoIterator for BlazeMap<K, V>
where
    K: BlazeMapId,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> IntoIter<K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        IntoIter { inner: self }
    }
}

impl<'a, K, V> IntoIterator for &'a BlazeMap<K, V>
where
    K: BlazeMapId,
{
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Iter<'a, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut BlazeMap<K, V>
where
    K: BlazeMapId,
{
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> IterMut<'a, K, V> {
        debug_assert_eq!(
            self.inner.iter().filter_map(Option::as_ref).count(),
            self.len
        );
        self.iter_mut()
    }
}

impl<K, V> FromIterator<(K, V)> for BlazeMap<K, V>
where
    K: BlazeMapIdStatic,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut result = BlazeMap::with_current_key_type_capacity();
        iter.into_iter().for_each(|(key, value)| {
            result.insert(key, value);
        });
        debug_assert_eq!(
            result.inner.iter().filter_map(Option::as_ref).count(),
            result.len
        );
        result
    }
}

impl<K, V> Default for BlazeMap<K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! blaze_map_orig_key_blocking_iter {
    ($self:ident, $iter:ident, $guard:ident) => {
        let $guard = K::static_container().key_by_offset_provider();
        let $iter = $self
            .inner
            .iter()
            .enumerate()
            .filter_map(|(idx, value)| Some((idx, value.as_ref()?)))
            .map(|(idx, value)| {
                let key = unsafe { $guard.key_by_offset_unchecked(idx) };
                (key, value)
            });
    };
}

impl<K, V> Debug for BlazeMap<K, V>
where
    K: BlazeMapIdStatic,
    <K as BlazeMapId>::OrigType: Debug,
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        blaze_map_orig_key_blocking_iter!(self, iter, guard);
        let mut debug_map = f.debug_map();
        for (key, value) in iter {
            debug_map.entry(key.borrow(), value);
        }
        debug_map.finish()
    }
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for BlazeMap<K, V>
where
    K: BlazeMapIdStatic,
    <K as BlazeMapId>::OrigType: Serialize,
    V: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        blaze_map_orig_key_blocking_iter!(self, iter, guard);
        let mut serializer = serializer.serialize_map(Some(self.len))?;
        for (key, value) in iter {
            serializer.serialize_entry(key.borrow(), value)?;
        }
        serializer.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for BlazeMap<K, V>
where
    K: BlazeMapIdWrapper + BlazeMapIdStatic,
    <K as BlazeMapId>::OrigType: Deserialize<'de>,
    V: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(BlazeMapDeserializer(PhantomData))
    }
}

#[cfg(feature = "serde")]
struct BlazeMapDeserializer<K, V>(PhantomData<(K, V)>);

#[cfg(feature = "serde")]
impl<'de, K, V> Visitor<'de> for BlazeMapDeserializer<K, V>
where
    K: BlazeMapIdWrapper + BlazeMapIdStatic,
    <K as BlazeMapId>::OrigType: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = BlazeMap<K, V>;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "BlazeMap-compatible map")
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = BlazeMap::with_current_key_type_capacity();

        while let Some((key, value)) = map.next_entry::<K::OrigType, V>()? {
            let key = unsafe { K::new(K::static_container(), key) };
            result.insert(key, value);
        }
        result.shrink_to_fit();
        debug_assert_eq!(
            result.inner.iter().filter_map(Option::as_ref).count(),
            result.len
        );
        Ok(result)
    }
}
