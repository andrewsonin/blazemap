//! Provides a [wrapper](register_blazemap_id) for replacing a small number
//! of clumsy objects with identifiers,
//! and also implements a [vector-based slab-like map](prelude::BlazeMap)
//! with an interface similar to that of [`HashMap`](std::collections::HashMap).

mod id_wrapper;
#[doc(hidden)]
pub mod utils;
/// Collection types.
pub mod collections;
mod macros;
#[doc(hidden)]
pub mod orig_type_id_map;

/// Crate prelude.
pub mod prelude {
    pub use crate::{
        collections::blazemap::BlazeMap,
        id_wrapper::IdWrapper,
        register_blazemap_id,
    };
}

/// Public re-exports of external crates used.
pub mod external {
    #[cfg(feature = "serde")]
    pub use serde;

    pub use {once_cell, parking_lot};
}

#[cfg(test)]
mod tests
{
    use crate::register_blazemap_id;

    #[cfg(feature = "serde")]
    mod serde_compatible
    {
        use crate::register_blazemap_id;

        register_blazemap_id! {
            pub struct BlazeMapKeyExample(usize);
            Derive(as for Original Type): {
                Default,
                Debug,
                Display,
                Ord,
                Serialize,
                Deserialize
            }
        }
    }

    register_blazemap_id! {
        pub struct BlazeMapKeyExample(usize);
        Derive(as for Original Type): {
            Default,
            Debug,
            Display,
            Ord
        }
    }
}