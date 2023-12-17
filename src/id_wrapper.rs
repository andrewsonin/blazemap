use std::hash::Hash;

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