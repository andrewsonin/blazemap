use std::hash::Hash;

use read_write_api::{RwApi, UpgradableReadApi, UpgradableReadGuard};

use crate::orig_type_id_map::OrigTypeIdMap;

/// Provides interface for `blazemap` key-wrapper types
/// defined by the [`register_blazemap_id`](crate::register_blazemap_id) macro.
pub trait IdWrapper: Copy
{
    /// Original key type.
    type OrigType: 'static + Clone + Eq + Hash;

    #[doc(hidden)]
    type OrigTypeIdMap: 'static + OrigTypeIdMap<Self::OrigType>;

    #[doc(hidden)]
    type OrigTypeIdMapApi: RwApi<Target=&'static mut Self::OrigTypeIdMap>;

    /// Creates a new instance of [`Self`] based on the [`Self::OrigType`] instance.
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

    #[doc(hidden)]
    fn get_index(self) -> usize;

    #[doc(hidden)]
    unsafe fn from_index_unchecked(index: usize) -> Self;

    #[doc(hidden)]
    fn static_info() -> Self::OrigTypeIdMapApi;
}