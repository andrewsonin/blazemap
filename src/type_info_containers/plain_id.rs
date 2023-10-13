use crate::sync::{AtomicUsize, Ordering};
use std::borrow::Borrow;
use std::ops::Deref;

use crate::traits::{CapacityInfoProvider, KeyByOffsetProvider, TypeInfoContainer};

/// Global, statically initialized offset generator.
#[doc(hidden)]
#[derive(Debug)]
pub struct StaticContainer {
    next_offset: AtomicUsize,
}

impl StaticContainer {
    /// Creates a new instance of [`StaticContainer`].
    #[inline]
    #[must_use]
    #[cfg(not(loom))]
    pub const fn new() -> Self {
        Self {
            next_offset: AtomicUsize::new(0),
        }
    }

    /// Creates a new instance of [`StaticContainer`].
    ///
    /// # Safety
    /// Mustn't be used outside of loom tests,
    /// since there is no guarantee that one [`BlazeMapId`](crate::prelude::BlazeMapId)
    /// doesn't interact with different containers of the same type.
    #[inline]
    #[must_use]
    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            next_offset: AtomicUsize::new(0),
        }
    }

    /// Returns the next identifier.
    #[inline]
    pub fn next_id(&self) -> usize {
        self.next_offset
            .fetch_update(Ordering::Release, Ordering::Acquire, |next_id| {
                next_id.checked_add(1)
            })
            .expect("usize overflow")
    }
}

impl TypeInfoContainer for StaticContainer {
    type OrigType = usize;

    #[inline]
    fn capacity_info_provider(&self) -> impl Deref<Target = impl CapacityInfoProvider> {
        self
    }

    #[inline]
    fn key_by_offset_provider(
        &self,
    ) -> impl Deref<Target = impl KeyByOffsetProvider<Self::OrigType>> {
        &KeyByOffsetProviderTrivial
    }
}

impl CapacityInfoProvider for StaticContainer {
    #[inline]
    fn offset_capacity(&self) -> usize {
        self.next_offset.load(Ordering::Acquire)
    }
}

/// Zero-sized type that trivially implements [`KeyByOffsetProvider`].
#[doc(hidden)]
#[repr(transparent)]
#[derive(Debug)]
pub struct KeyByOffsetProviderTrivial;

impl KeyByOffsetProvider<usize> for KeyByOffsetProviderTrivial {
    #[inline]
    unsafe fn key_by_offset_unchecked(&self, offset: usize) -> impl Borrow<usize> {
        offset
    }
}
