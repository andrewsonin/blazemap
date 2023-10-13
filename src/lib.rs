mod key_wrapper;
#[doc(hidden)]
pub mod utils;
/// Collection types.
pub mod collections;
mod macros;

/// Crate prelude.
pub mod prelude {
    pub use crate::{
        collections::blazemap::BlazeMap,
        key_wrapper::KeyWrapper,
        register_blazemap_key,
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
    use crate::register_blazemap_key;

    #[cfg(feature = "serde")]
    mod serde_compatible
    {
        use crate::register_blazemap_key;

        register_blazemap_key! {
            pub struct BlazeMapKeyExample(usize);
            DERIVE AS FOR ORIGINAL TYPE: {
                Default,
                Debug,
                Display,
                Ord,
                Serialize,
                Deserialize
            }
        }
    }

    register_blazemap_key! {
        pub struct BlazeMapKeyExample(usize);
        DERIVE AS FOR ORIGINAL TYPE: {
            Default,
            Debug,
            Display,
            Ord
        }
    }
}