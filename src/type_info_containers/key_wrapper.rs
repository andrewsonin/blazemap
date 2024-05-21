use crate::{
    prelude::BlazeMapId,
    sync::RwLock,
    traits::{CapacityInfoProvider, KeyByOffsetProvider, TypeInfoContainer, WrapKey},
};
#[cfg(not(feature = "loom"))]
use once_cell::sync::Lazy;
use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    ops::Deref,
};

/// Global, statically initialized container with correspondence mapping
/// between blazemap offset wrappers and original keys.
#[cfg(not(feature = "loom"))]
#[doc(hidden)]
#[derive(Debug)]
pub struct StaticContainer<K> {
    offset_to_orig: Vec<K>,
    orig_to_offset: Lazy<HashMap<K, usize>>,
}

/// Loom-testable version of the above container.
/// Note that it cannot be static
/// due to the [`loom` inability](https://github.com/tokio-rs/loom/issues/290)
/// to test statically initialized code.
#[cfg(feature = "loom")]
#[doc(hidden)]
#[derive(Debug)]
pub struct StaticContainer<K> {
    offset_to_orig: Vec<K>,
    orig_to_offset: HashMap<K, usize>,
}

impl<K> Default for StaticContainer<K> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K> StaticContainer<K> {
    /// Creates a new instance of [`StaticContainer`].
    #[inline]
    #[must_use]
    #[cfg(not(feature = "loom"))]
    pub const fn new() -> Self {
        Self {
            offset_to_orig: vec![],
            orig_to_offset: Lazy::new(Default::default),
        }
    }

    /// Creates a new instance of [`StaticContainer`].
    ///
    /// # Safety
    /// Mustn't be used outside of loom tests,
    /// since there is no guarantee that one
    /// [`BlazeMapId`](crate::prelude::BlazeMapId) doesn't interact with
    /// different containers of the same type.
    #[inline]
    #[must_use]
    #[cfg(feature = "loom")]
    pub fn new() -> Self {
        Self {
            offset_to_orig: vec![],
            orig_to_offset: HashMap::new(),
        }
    }
}

impl<K, I> WrapKey<I> for RwLock<StaticContainer<K>>
where
    K: Clone + Eq + Hash,
    I: BlazeMapId<OrigType = K>,
{
    #[inline]
    fn wrap_key(&self, key: K) -> I {
        #[cfg(not(feature = "loom"))]
        let offset = self.read().orig_to_offset.get(&key).copied();
        #[cfg(feature = "loom")]
        let offset = self.read().unwrap().orig_to_offset.get(&key).copied();
        unsafe {
            if let Some(offset) = offset {
                I::from_offset_unchecked(offset)
            } else {
                #[cfg(not(feature = "loom"))]
                let mut guard = self.write();
                #[cfg(feature = "loom")]
                let mut guard = self.write().unwrap();
                let container = &mut *guard;
                let offset = match container.orig_to_offset.entry(key) {
                    Entry::Vacant(entry) => {
                        let offset = container.offset_to_orig.len();
                        container.offset_to_orig.push(entry.key().clone());
                        entry.insert(offset);
                        offset
                    }
                    Entry::Occupied(entry) => *entry.get(),
                };
                drop(guard);
                I::from_offset_unchecked(offset)
            }
        }
    }
}

impl<K> TypeInfoContainer for RwLock<StaticContainer<K>>
where
    K: 'static,
{
    type OrigType = K;

    #[inline]
    fn capacity_info_provider(&self) -> impl Deref<Target = impl CapacityInfoProvider> {
        #[cfg(not(feature = "loom"))]
        let result = self.read();
        #[cfg(feature = "loom")]
        let result = self.read().unwrap();
        result
    }

    #[inline]
    fn key_by_offset_provider(
        &self,
    ) -> impl Deref<Target = impl KeyByOffsetProvider<Self::OrigType>> {
        #[cfg(not(feature = "loom"))]
        let result = self.read();
        #[cfg(feature = "loom")]
        let result = self.read().unwrap();
        result
    }
}

impl<K> CapacityInfoProvider for StaticContainer<K> {
    #[inline]
    fn offset_capacity(&self) -> usize {
        self.offset_to_orig.len()
    }
}

impl<K> KeyByOffsetProvider<K> for StaticContainer<K> {
    #[inline]
    unsafe fn key_by_offset_unchecked(&self, offset: usize) -> impl Borrow<K> {
        #[cfg(not(feature = "loom"))]
        let result = self.offset_to_orig.get_unchecked(offset);
        #[cfg(feature = "loom")]
        let result = self.offset_to_orig.get(offset).unwrap();
        result
    }
}
