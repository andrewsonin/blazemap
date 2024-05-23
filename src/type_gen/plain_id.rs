/// Creates a new type based on incrementally generated `usize` instances
/// that can be used as a key for `blazemap` collections.
///
/// This macro supports optional inference of standard traits using the
/// following syntax:
///
/// * `Derive` — derives traits in the same way as for the serial number
///   assigned when creating a new instance of the type. Because methods
///   inferred by this option do not require additional locking on
///   synchronization primitives, they do not incur any additional overhead
///   compared to methods inferred for plain `usize`. This method supports
///   inference of the following traits:
///   * `PartialOrd` (mutually exclusive with `Ord`)
///   * `Ord` (also derives `PartialOrd`, so mutually exclusive with
///     `PartialOrd`)
///   * `Serialize` (with `serde` feature only)
///
/// # Example
///
/// ```rust
/// use blazemap::{prelude::Map, define_plain_id};
///
/// define_plain_id! {
///     pub struct Id;
///     Derive: {       // Optional section
///         Ord
///     };
/// }
///
/// let key_1 = Id::new();
/// let key_2 = Id::new();
/// let key_3 = Id::new();
///
/// let mut map = Map::new();
/// map.insert(key_2, "2");
/// map.insert(key_1, "1");
/// map.insert(key_3, "3");
///
/// assert_eq!(format!("{map:?}"), r#"{0: "1", 1: "2", 2: "3"}"#)
/// ```
#[macro_export]
macro_rules! define_plain_id {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident
        $(; Derive: {$($to_derive_sn:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::plain_id_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type
        }
        $($($crate::plain_id_derive! {@DERIVE $to_derive_sn $new_type})*)?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! plain_id_inner {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident
    ) => {
        $(#[$attrs])*
        #[derive(Clone, Copy, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $vis struct $new_type($crate::utils::OffsetProvider<usize>);

        impl $new_type
        {
            #[doc = ::std::concat!("Creates a new instance of [`", ::std::stringify!($new_type), "`].")]
            #[inline]
            #[cfg(not(feature = "loom"))]
            $vis fn new() -> Self {
                let next_id = <Self as $crate::prelude::BlazeMapIdStatic>::static_container().next_id();
                Self(unsafe { $crate::utils::OffsetProvider::<usize>::new(next_id) })
            }

            #[doc = ::std::concat!("Creates a new instance of [`", ::std::stringify!($new_type), "`].")]
            #[inline]
            #[cfg(feature = "loom")]
            $vis fn new(type_info_container: &<Self as $crate::prelude::BlazeMapId>::TypeInfoContainer) -> Self {
                let next_id = type_info_container.next_id();
                Self(unsafe { $crate::utils::OffsetProvider::<usize>::new(next_id) })
            }
        }

        impl $crate::prelude::BlazeMapId for $new_type
        {
            type OrigType = usize;
            type TypeInfoContainer = $crate::type_info_containers::plain_id::StaticContainer;

            #[inline]
            fn get_offset(self) -> usize {
                self.0.into_offset()
            }

            #[inline]
            unsafe fn from_offset_unchecked(offset: usize) -> Self {
                Self($crate::utils::OffsetProvider::<usize>::new(offset))
            }
        }

        #[cfg(not(feature = "loom"))]
        impl $crate::traits::BlazeMapIdStatic for $new_type
        {
            #[inline]
            fn static_container() -> &'static Self::TypeInfoContainer
            {
                use $crate::type_info_containers::plain_id::StaticContainer;
                static INFO: StaticContainer = StaticContainer::new();
                &INFO
            }
        }

        impl ::std::fmt::Debug for $new_type
        {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result
            {
                f.debug_tuple(::std::stringify!($new_type))
                    .field(&self.0.into_offset())
                    .finish()
            }
        }

        impl ::std::fmt::Display for $new_type
        {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result
            {
                write!(f, "{}", self.0.into_offset())
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! plain_id_derive {
    (@DERIVE PartialOrd $new_type:ident) => {
        impl PartialOrd for $new_type {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                let Self(lhs) = self;
                let Self(rhs) = other;
                lhs.into_offset().partial_cmp(&rhs.into_offset())
            }
        }
    };
    (@DERIVE Ord $new_type:ident) => {
        impl PartialOrd for $new_type {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $new_type {
            #[inline]
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let Self(lhs) = self;
                let Self(rhs) = other;
                lhs.into_offset().cmp(&rhs.into_offset())
            }
        }
    };
    (@DERIVE Serialize $new_type:ident) => {
        impl $crate::external::serde::Serialize for $new_type {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: $crate::external::serde::Serializer,
            {
                self.0.into_offset().serialize(serializer)
            }
        }
    };
}
