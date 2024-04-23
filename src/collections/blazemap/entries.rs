use crate::prelude::BlazeMapId;

#[derive(Debug)]
/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed
/// from the [`entry`] method on
/// [`BlazeMap`](crate::collections::blazemap::BlazeMap).
///
/// [`entry`]: crate::collections::blazemap::BlazeMap::entry
pub enum Entry<'a, K, V>
where
    K: BlazeMapId,
{
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

#[derive(Debug)]
/// A view into an occupied entry in a
/// [`BlazeMap`](crate::collections::blazemap::BlazeMap). It is part of the
/// [`Entry`] enum.
pub struct OccupiedEntry<'a, K, V>
where
    K: BlazeMapId,
{
    pub(in crate::collections::blazemap) key: K,

    pub(in crate::collections::blazemap) len: &'a mut usize,

    pub(in crate::collections::blazemap) value: &'a mut Option<V>,
}

#[derive(Debug)]
/// A view into a vacant entry in a
/// [`BlazeMap`](crate::collections::blazemap::BlazeMap). It is part of the
/// [`Entry`] enum.
pub struct VacantEntry<'a, K, V>
where
    K: BlazeMapId,
{
    pub(in crate::collections::blazemap) key: K,

    pub(in crate::collections::blazemap) len: &'a mut usize,

    pub(in crate::collections::blazemap) inner: VacantEntryInner<'a, V>,
}

#[derive(Debug)]
pub(in crate::collections::blazemap) enum VacantEntryInner<'a, V> {
    ShouldBeInserted(&'a mut Option<V>),
    ShouldBeEnlarged(&'a mut Vec<Option<V>>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: BlazeMapId,
{
    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in
    /// the entry.
    #[inline]
    pub fn or_insert_with(self, default: impl FnOnce() -> V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Returns this entry’s key.
    #[inline]
    pub fn key(&self) -> K {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access
    /// to an occupied entry before any potential inserts into the map.
    #[inline]
    #[must_use]
    pub fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: BlazeMapId,
    V: Default,
{
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    #[inline]
    pub fn or_default(self) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: BlazeMapId,
{
    /// Gets the key in the entry.
    #[inline]
    pub fn key(&self) -> K {
        self.key
    }

    /// Take the ownership of the key and value from the map.
    #[inline]
    pub fn remove_entry(self) -> (K, V) {
        let Self { key, len, value } = self;
        *len -= 1;
        let value = unsafe { value.take().unwrap_unchecked() };
        (key, value)
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    pub fn get(&self) -> &V {
        unsafe { self.value.as_ref().unwrap_unchecked() }
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the [`OccupiedEntry`]
    /// which may outlive the destruction of the [`Entry`] value, see
    /// [`into_mut`].
    ///
    /// [`into_mut`]: Self::into_mut
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.value.as_mut().unwrap_unchecked() }
    }

    /// Converts the [`OccupiedEntry`] into a mutable reference
    /// to the value in the entry with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the [`OccupiedEntry`], see
    /// [`get_mut`].
    ///
    /// [`get_mut`]: Self::get_mut
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        unsafe { self.value.as_mut().unwrap_unchecked() }
    }

    /// Sets the value of the entry, and returns the entry’s old value.
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.get_mut(), value)
    }

    /// Takes the value out of the entry, and returns it.
    #[inline]
    pub fn remove(self) -> V {
        let Self { len, value, .. } = self;
        *len -= 1;
        unsafe { value.take().unwrap_unchecked() }
    }
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: BlazeMapId,
{
    /// Gets the key that would be used when inserting a value through the
    /// [`VacantEntry`].
    #[inline]
    pub fn key(&self) -> K {
        self.key
    }

    /// Sets the value of the entry with the [`VacantEntry`]’s key,
    /// and returns a mutable reference to it.
    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        let Self { key, len, inner } = self;
        *len += 1;
        let reference = match inner {
            VacantEntryInner::ShouldBeInserted(reference) => reference,
            VacantEntryInner::ShouldBeEnlarged(vec) => {
                let offset = key.get_offset();
                let new_len = offset + 1; // It's safe to don't use `checked_add`, since `vec` will panic at `isize::MAX`
                vec.resize_with(new_len, || None);
                unsafe { vec.get_unchecked_mut(offset) }
            }
        };
        *reference = Some(value);
        unsafe { reference.as_mut().unwrap_unchecked() }
    }
}
