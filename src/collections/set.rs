mod iters;

use crate::{
    collections::set::iters::{Drain, IntoIter, Iter},
    prelude::{BlazeMapId, BlazeMapIdStatic, BlazeMapIdWrapper},
    traits::{CapacityInfoProvider, TypeInfoContainer},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{Debug, Formatter},
    iter::{once_with, repeat},
    marker::PhantomData,
};

/// A [`Vec`]-based analogue of a [`HashSet`](std::collections::HashSet).
#[derive(Clone, PartialEq, Eq)]
pub struct BlazeSet<K> {
    bitmask: Vec<u8>,
    len: usize,
    phantom: PhantomData<K>,
}

impl<K> BlazeSet<K> {
    /// Creates a new instance of [`BlazeSet`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bitmask: vec![],
            len: 0,
            phantom: PhantomData,
        }
    }

    /// Returns the number of elements in the set.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the set contains no elements.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears the set, removing all keys. Keeps the allocated memory
    /// for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.bitmask.clear();
        self.len = 0;
    }

    /// Shrinks the capacity of the set as much as possible.
    /// It will drop down as much as possible while maintaining the internal
    /// rules and possibly leaving some space in accordance with the resize
    /// policy.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        while let Some(last) = self.bitmask.last() {
            if *last == 0 {
                // `unwrap_unchecked` here is just a compiler hint.
                unsafe { self.bitmask.pop().unwrap_unchecked() };
            } else {
                break;
            }
        }
        self.bitmask.shrink_to_fit();
    }

    /// Clears the set, returning all keys as an iterator.
    /// Keeps the allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed,
    /// it drops the remaining keys.
    /// The returned iterator keeps a mutable borrow on the set to optimize its
    /// implementation.
    #[inline]
    #[must_use]
    pub fn drain(&mut self) -> Drain<'_, K> {
        todo!()
    }
}

impl<K> BlazeSet<K>
where
    K: BlazeMapId,
{
    /// An iterator visiting all keys. The iterator element type is `K`.
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, K> {
        todo!()
    }
}

impl<K> BlazeSet<K>
where
    K: BlazeMapIdStatic,
{
    /// Creates a new instance of [`BlazeSet`]
    /// with capacity equal to the current total number of unique `K` instances.
    #[inline]
    #[must_use]
    pub fn with_current_key_type_capacity() -> Self {
        let current_capacity = K::static_container()
            .capacity_info_provider()
            .offset_capacity();
        let rem = current_capacity % 8;
        let cap = if rem != 0 {
            current_capacity / 8 + 1
        } else {
            current_capacity / 8
        };
        Self {
            bitmask: vec![0; cap],
            len: 0,
            phantom: PhantomData,
        }
    }
}

impl<K> BlazeSet<K>
where
    K: BlazeMapId,
{
    /// Returns `true` if the set contains the specified key.
    #[inline]
    #[must_use]
    pub fn contains(&self, key: K) -> bool {
        let offset = key.get_offset();
        let position = offset / 8;
        if let Some(cell) = self.bitmask.get(position) {
            let bit = position % 8;
            cell & (1 << bit) != 0
        } else {
            false
        }
    }

    /// Inserts a key into the set.
    ///
    /// If the set did not have this key present, `false` is returned.
    ///
    /// If the set did have this key present, `true` is returned.
    #[inline]
    pub fn insert(&mut self, key: K) -> bool {
        let offset = key.get_offset();
        let position = offset / 8;
        if let Some(cell) = self.bitmask.get_mut(position) {
            let bit = position % 8;
            let mask = 1 << bit;
            let was_here = *cell & mask != 0;
            *cell |= mask;
            if !was_here {
                self.len += 1;
            }
            was_here
        } else {
            let new = repeat(0)
                .take(position - self.bitmask.len())
                .chain(once_with(|| {
                    let bit = position % 8;
                    1 << bit
                }));
            self.bitmask.extend(new);
            self.len += 1;
            false
        }
    }

    /// Removes a key from the set,
    /// returning the `true` if the key was previously in the set.
    #[inline]
    pub fn remove(&mut self, key: K) -> bool {
        let offset = key.get_offset();
        let position = offset / 8;
        if let Some(cell) = self.bitmask.get_mut(position) {
            let bit = position % 8;
            let mask = 1 << bit;
            let was_here = *cell & mask != 0;
            *cell &= 0b11111111 ^ mask;
            if was_here {
                self.len -= 1;
            }
            was_here
        } else {
            false
        }
    }
}

impl<K> IntoIterator for BlazeSet<K>
where
    K: BlazeMapId,
{
    type Item = K;
    type IntoIter = IntoIter<K>;

    #[inline]
    fn into_iter(self) -> IntoIter<K> {
        todo!()
    }
}

impl<'a, K> IntoIterator for &'a BlazeSet<K>
where
    K: BlazeMapId,
{
    type Item = K;
    type IntoIter = Iter<'a, K>;

    #[inline]
    fn into_iter(self) -> Iter<'a, K> {
        todo!()
    }
}

impl<'a, K> IntoIterator for &'a mut BlazeSet<K>
where
    K: BlazeMapId,
{
    type Item = K;
    type IntoIter = Iter<'a, K>;

    #[inline]
    fn into_iter(self) -> Iter<'a, K> {
        (self as &BlazeSet<K>).into_iter()
    }
}

impl<K> FromIterator<K> for BlazeSet<K>
where
    K: BlazeMapIdStatic,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        todo!()
    }
}

impl<K> Default for BlazeSet<K>
where
    K: BlazeMapId,
{
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl<K> Debug for BlazeSet<K>
where
    K: BlazeMapIdStatic,
    <K as BlazeMapId>::OrigType: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<K> Serialize for BlazeSet<K>
where
    K: BlazeMapIdStatic,
    <K as BlazeMapId>::OrigType: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<'de, K> Deserialize<'de> for BlazeSet<K>
where
    K: BlazeMapIdWrapper + BlazeMapIdStatic,
    <K as BlazeMapId>::OrigType: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}
