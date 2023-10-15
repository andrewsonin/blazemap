use std::hash::Hash;

use parking_lot::{RwLock, RwLockUpgradableReadGuard};

use crate::utils::StaticInfo;

/// Provides interface for `blazemap` key-wrapper types
/// defined by the [`register_blazemap_id`](crate::register_blazemap_id) macro.
pub trait IdWrapper: Copy
{
    /// Original key type.
    type OrigType: 'static + Clone + Eq + Hash;

    /// Creates a new instance of [`Self`] based on the [`Self::OrigType`] instance.
    #[inline]
    fn new(key: Self::OrigType) -> Self {
        unsafe {
            let guard = Self::static_info().upgradable_read();
            if let Some(index) = guard.get_index(&key) {
                Self::from_index_unchecked(index)
            } else {
                let mut guard = RwLockUpgradableReadGuard::upgrade(guard);
                let index = guard.insert_new_key_unchecked(key);
                Self::from_index_unchecked(index)
            }
        }
    }

    #[doc(hidden)]
    fn get_index(self) -> usize;

    #[doc(hidden)]
    unsafe fn from_index_unchecked(index: usize) -> Self;

    #[doc(hidden)]
    fn static_info() -> &'static RwLock<StaticInfo<Self::OrigType>>;
}