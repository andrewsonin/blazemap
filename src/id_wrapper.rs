use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Range;

use read_write_api::{ReadApi, UpgradableReadApi, UpgradableReadGuard};

use crate::orig_type_id_map::{InsertableStaticInfoApi, StaticInfoApi};

/// Provides interface for `blazemap` id types
/// defined by the [`register_blazemap_id_wrapper`](crate::register_blazemap_id_wrapper)
/// and [`register_blazemap_id`](crate::register_blazemap_id) macros.
pub trait BlazeMapId: Copy
{
    /// Original key type.
    type OrigType: 'static + Clone + Eq + Hash;

    #[doc(hidden)]
    type StaticInfoApi: 'static + StaticInfoApi<Self::OrigType>;

    #[doc(hidden)]
    type StaticInfoApiLock: ReadApi<Target=Self::StaticInfoApi>;

    /// Creates an iterator over all identifiers registered at the time this method was called.
    #[inline]
    fn all_instances_iter() -> AllInstancesIter<Self> {
        let num_elems = Self::static_info().read().num_elems();
        AllInstancesIter {
            range: 0..num_elems,
            phantom: Default::default(),
        }
    }

    #[doc(hidden)]
    fn get_index(self) -> usize;

    #[doc(hidden)]
    unsafe fn from_index_unchecked(index: usize) -> Self;

    #[doc(hidden)]
    fn static_info() -> Self::StaticInfoApiLock;
}

/// Provides interface for constructable `blazemap` key-wrapper types
/// defined by the [`register_blazemap_id_wrapper`](crate::register_blazemap_id_wrapper) macro.
pub trait BlazeMapIdWrapper: BlazeMapId
    where
        Self::StaticInfoApi: InsertableStaticInfoApi<Self::OrigType>,
        Self::StaticInfoApiLock: UpgradableReadApi
{
    /// Creates a new instance of [`Self`] based on the [`Self::OrigType`](BlazeMapId::OrigType) instance.
    #[inline]
    fn new(key: Self::OrigType) -> Self {
        unsafe {
            let mut static_info = Self::static_info();
            let guard = static_info.upgradable_read();
            if let Some(index) = guard.get_index(&key) {
                Self::from_index_unchecked(index)
            } else {
                let mut guard = guard.upgrade();
                let index = guard.insert_new_key_unchecked(key);
                Self::from_index_unchecked(index)
            }
        }
    }
}

/// Iterator over consecutive `blazemap` identifiers.
pub struct AllInstancesIter<T>
{
    range: Range<usize>,
    phantom: PhantomData<T>,
}

impl<T> Clone for AllInstancesIter<T>
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            phantom: Default::default(),
        }
    }
}

impl<T> Debug for AllInstancesIter<T>
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.range)
    }
}

impl<T> PartialEq for AllInstancesIter<T>
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range
    }
}

impl<T> Eq for AllInstancesIter<T> {}

impl<T> Hash for AllInstancesIter<T>
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.range.hash(state)
    }
}

impl<T> Iterator for AllInstancesIter<T>
    where
        T: BlazeMapId
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next_id = self.range.next()?;
        Some(unsafe { T::from_index_unchecked(next_id) })
    }
}

impl<T> DoubleEndedIterator for AllInstancesIter<T>
    where
        T: BlazeMapId
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_back_id = self.range.next_back()?;
        Some(unsafe { T::from_index_unchecked(next_back_id) })
    }
}

impl<T> ExactSizeIterator for AllInstancesIter<T>
    where
        T: BlazeMapId
{
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}