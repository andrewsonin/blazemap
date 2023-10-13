//! Implements a [vector-based slab-like map](prelude::BlazeMap)
//! with an interface similar to that of [`HashMap`](std::collections::HashMap),
//! and also provides tools
//! for generating lightweight identifiers that can be type-safely used as keys for this map.

/// Collection types.
pub mod collections;
/// Utilities for testing the codebase with [`loom`](crate::external::loom).
#[cfg(loom)]
pub mod loom;
#[doc(hidden)]
pub mod sync;
#[doc(hidden)]
pub mod traits;
mod type_gen;
#[doc(hidden)]
pub mod type_info_containers;
#[doc(hidden)]
pub mod utils;

/// Crate prelude.
pub mod prelude {
    pub use crate::{
        collections::blazemap::BlazeMap,
        define_key_wrapper, define_key_wrapper_bounded, define_plain_id,
        traits::{AllInstancesIter, BlazeMapId, BlazeMapIdStatic, BlazeMapIdWrapper},
    };
}

/// Public re-exports of external crates used.
pub mod external {
    #[cfg(feature = "serde")]
    pub use serde;

    #[cfg(loom)]
    pub use loom;
    pub use {once_cell, parking_lot};
}
