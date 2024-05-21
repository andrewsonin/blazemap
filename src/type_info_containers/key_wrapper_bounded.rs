#[cfg(feature = "loom")]
use crate::sync::RwLockReadGuard;
use crate::{
    prelude::BlazeMapId,
    sync::{AtomicUsize, Ordering, RwLock},
    traits::{CapacityInfoProvider, KeyByOffsetProvider, TypeInfoContainer, WrapKey},
};
use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    ops::Deref,
};
#[cfg(not(feature = "loom"))]
use std::{
    cell::UnsafeCell,
    mem::{needs_drop, MaybeUninit},
};

/// Global, statically initialized container with correspondence mapping
/// between blazemap index wrappers and original keys.
///
/// Being an analogue of
/// [`KeyWrapperStaticContainer`](crate::type_info_containers::key_wrapper::StaticContainer)
/// for the case when the user could statically guarantee
/// that the number of unique keys doesn't exceed `CAP`, it's optimized for read
/// operations so that they don't create any multi-thread contention.
#[cfg(not(feature = "loom"))]
#[doc(hidden)]
#[derive(Debug)]
pub struct StaticContainer<K, const CAP: usize> {
    offset_to_orig: Vec<UnsafeCell<MaybeUninit<K>>>,
    orig_to_offset: RwLock<HashMap<K, usize>>,
    next_offset: AtomicUsize,
}

/// Loom-testable version of the above container.
/// Note that it cannot be static
/// due to the [`loom` inability](https://github.com/tokio-rs/loom/issues/290)
/// to test statically initialized code.
#[cfg(feature = "loom")]
#[doc(hidden)]
#[derive(Debug)]
pub struct StaticContainer<K, const CAP: usize> {
    offset_to_orig: Vec<RwLock<Option<K>>>,
    orig_to_offset: RwLock<HashMap<K, usize>>,
    next_offset: AtomicUsize,
}

#[cfg(not(feature = "loom"))]
impl<K, const CAP: usize> Default for StaticContainer<K, CAP> {
    #[inline]
    fn default() -> Self {
        Self {
            offset_to_orig: std::iter::repeat_with(|| UnsafeCell::new(MaybeUninit::uninit()))
                .take(CAP)
                .collect(),
            orig_to_offset: RwLock::new(HashMap::with_capacity(CAP)),
            next_offset: AtomicUsize::new(0),
        }
    }
}

#[cfg(feature = "loom")]
impl<K, const CAP: usize> Default for StaticContainer<K, CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, const CAP: usize> StaticContainer<K, CAP> {
    /// Creates a new instance of [`StaticContainer`].
    #[inline]
    #[must_use]
    #[cfg(not(feature = "loom"))]
    pub fn new() -> Self {
        Self::default()
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
            offset_to_orig: std::iter::repeat_with(|| RwLock::new(None))
                .take(CAP)
                .collect(),
            orig_to_offset: RwLock::new(HashMap::with_capacity(CAP)),
            next_offset: AtomicUsize::new(0),
        }
    }

    #[inline]
    #[doc(hidden)]
    #[cfg(not(feature = "loom"))]
    pub unsafe fn key_by_offset_unchecked(&self, offset: usize) -> &K {
        (*self.offset_to_orig.get_unchecked(offset).get()).assume_init_ref()
    }

    #[inline]
    #[doc(hidden)]
    #[cfg(feature = "loom")]
    pub unsafe fn key_by_offset_unchecked(&self, offset: usize) -> RwLockReadGuard<'_, Option<K>> {
        self.offset_to_orig.get(offset).unwrap().read().unwrap()
    }
}

impl<K, I, const CAP: usize> WrapKey<I> for StaticContainer<K, CAP>
where
    K: Clone + Eq + Hash,
    I: BlazeMapId<OrigType = K>,
{
    #[inline]
    fn wrap_key(&self, key: K) -> I {
        #[cfg(not(feature = "loom"))]
        let offset = self.orig_to_offset.read().get(&key).copied();
        #[cfg(feature = "loom")]
        let offset = self.orig_to_offset.read().unwrap().get(&key).copied();
        unsafe {
            if let Some(offset) = offset {
                I::from_offset_unchecked(offset)
            } else {
                #[cfg(not(feature = "loom"))]
                let mut guard = self.orig_to_offset.write();
                #[cfg(feature = "loom")]
                let mut guard = self.orig_to_offset.write().unwrap();
                let offset = match guard.entry(key) {
                    Entry::Vacant(entry) => {
                        let offset = self.next_offset.load(Ordering::Relaxed);
                        let cell = self
                            .offset_to_orig
                            .get(offset)
                            .unwrap_or_else(|| panic!("capacity {CAP} overflow"));
                        #[cfg(not(feature = "loom"))]
                        (*cell.get()).write(entry.key().clone());
                        #[cfg(feature = "loom")]
                        {
                            let mut guard = cell.try_write().unwrap();
                            let value = &mut *guard;
                            assert!(value.is_none(), "value is already set");
                            *value = Some(entry.key().clone());
                        }
                        entry.insert(offset);
                        self.next_offset.store(offset + 1, Ordering::Release);
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

impl<K, const CAP: usize> Drop for StaticContainer<K, CAP> {
    #[inline]
    fn drop(&mut self) {
        #[cfg(not(feature = "loom"))]
        if !needs_drop::<K>() {
            return;
        }
        #[cfg(not(feature = "loom"))]
        let num_init = *self.next_offset.get_mut();
        #[cfg(feature = "loom")]
        let num_init = self.next_offset.load(Ordering::Acquire);
        self.offset_to_orig.as_mut_slice()[..num_init]
            .iter_mut()
            .for_each(|cell| {
                #[cfg(not(feature = "loom"))]
                unsafe {
                    cell.get_mut().assume_init_drop();
                };
                #[cfg(feature = "loom")]
                let _ = cell.try_write().unwrap().take();
            });
    }
}

unsafe impl<K: Sync + Send, const CAP: usize> Sync for StaticContainer<K, CAP> {}

impl<K: 'static, const CAP: usize> TypeInfoContainer for StaticContainer<K, CAP> {
    type OrigType = K;

    #[inline]
    fn capacity_info_provider(&self) -> impl Deref<Target = impl CapacityInfoProvider> {
        self
    }

    #[inline]
    fn key_by_offset_provider(
        &self,
    ) -> impl Deref<Target = impl KeyByOffsetProvider<Self::OrigType>> {
        self
    }
}

impl<K, const CAP: usize> CapacityInfoProvider for StaticContainer<K, CAP> {
    #[inline]
    fn offset_capacity(&self) -> usize {
        self.next_offset.load(Ordering::Acquire)
    }
}

#[cfg(feature = "loom")]
struct BorrowGuard<'a, K>(RwLockReadGuard<'a, Option<K>>);

#[cfg(feature = "loom")]
impl<K> Borrow<K> for BorrowGuard<'_, K> {
    fn borrow(&self) -> &K {
        self.0.as_ref().unwrap()
    }
}

impl<K, const CAP: usize> KeyByOffsetProvider<K> for StaticContainer<K, CAP> {
    #[inline]
    unsafe fn key_by_offset_unchecked(&self, offset: usize) -> impl Borrow<K> {
        let result = StaticContainer::key_by_offset_unchecked(self, offset);
        #[cfg(feature = "loom")]
        let result = BorrowGuard(result);
        result
    }
}
