/// Creates a new type that acts as an `usize`-based replacement for the old
/// type that can be used as a key for `blazemap` collections.
/// Being an analogue of [`define_key_wrapper`](crate::define_key_wrapper)
/// for the case when the user could statically guarantee
/// that the number of unique keys doesn't exceed `MAX_CAP`, it's optimized for
/// read operations so that they don't create any multi-thread contention.
///
/// This macro supports optional inference of standard traits using the
/// following syntax:
///
/// * `Derive(as for Original Type)` — derives traits as for the original type
///   for which `blazemap_key` is being registered. This method supports
///   inference of the following traits:
///   * `Default`
///   * `PartialOrd` (mutually exclusive with `Ord`)
///   * `Ord` (also derives `PartialOrd`, so mutually exclusive with
///     `PartialOrd`)
///   * `Debug`
///   * `Display`
///   * `Serialize` (with `serde` feature only)
///   * `Deserialize` (with `serde` feature only)
/// * `Derive(as for usize)` — derives traits in the same way as for the serial
///   number assigned when registering an instance of the original type the
///   first time
///   [`BlazeMapIdWrapper::new`](crate::prelude::BlazeMapIdWrapper::new) was
///   called. Methods inferred by this option do not incur any additional
///   overhead compared to methods inferred for plain `usize`. This method
///   supports inference of the following traits:
///   * `PartialOrd` (mutually exclusive with `Ord`)
///   * `Ord` (also derives `PartialOrd`, so mutually exclusive with
///     `PartialOrd`)
///
/// # Example
///
/// ```rust
/// use blazemap::{prelude::BlazeMap, define_key_wrapper_bounded};
///
/// define_key_wrapper_bounded! {
///     pub struct Key(&'static str);
///     MAX_CAP = 40_000;
///     Derive(as for Original Type): {  // Optional section
///         Debug,
///         Display,
///     };
///     Derive(as for usize): {          // Optional section
///         Ord,
///     }
/// }
///
/// let key_1 = Key::new("first");
/// let key_2 = Key::new("second");
/// let key_3 = Key::new("third");
///
/// let mut map = BlazeMap::new();
/// map.insert(key_2, "2");
/// map.insert(key_1, "1");
/// map.insert(key_3, "3");
///
/// assert_eq!(format!("{map:?}"), r#"{"first": "1", "second": "2", "third": "3"}"#)
/// ```
#[macro_export]
macro_rules! define_key_wrapper_bounded {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty);
        MAX_CAP = $capacity:literal
        $(; Derive(as for Original Type): {$($to_derive_orig:ident),+ $(,)?} )?
        $(; Derive(as for usize):         {$(  $to_derive_sn:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::key_wrapper_bounded_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type($orig_type);
            MAX_CAP = $capacity
        }
        $($($crate::key_wrapper_derive!     {@DERIVE $to_derive_orig $new_type})*)?
        $($($crate::assigned_offset_derive! {@DERIVE   $to_derive_sn $new_type})*)?
    };
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty);
        MAX_CAP = $capacity:literal
        $(; Derive(as for usize):         {$(  $to_derive_sn:ident),+ $(,)?} )?
        $(; Derive(as for Original Type): {$($to_derive_orig:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::key_wrapper_bounded_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type($orig_type);
            MAX_CAP = $capacity
        }
        $($($crate::key_wrapper_derive!     {@DERIVE $to_derive_orig $new_type})*)?
        $($($crate::assigned_offset_derive! {@DERIVE   $to_derive_sn $new_type})*)?
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! key_wrapper_bounded_inner {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty);
        MAX_CAP = $capacity:literal
    ) => {
        $(#[$attrs])*
        #[derive(Clone, Copy, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $vis struct $new_type($crate::utils::OffsetProvider<usize>);

        #[cfg(not(loom))]
        impl $new_type
        {
            #[inline]
            $vis fn new(value: $orig_type) -> Self {
                use $crate::traits::BlazeMapIdStatic;
                unsafe { <Self as $crate::prelude::BlazeMapIdWrapper>::new(Self::static_container(), value) }
            }

            #[doc = ::std::concat!(
                "Returns the original key corresponding to the [`",
                ::std::stringify!($new_type),
                "`] instance."
            )]
            #[inline]
            #[must_use]
            #[allow(dead_code)]
            $vis fn key(self) -> &'static $orig_type {
                let static_container = <Self as $crate::traits::BlazeMapIdStatic>::static_container();
                unsafe { static_container.key_by_offset_unchecked(self.0.into_offset()) }
            }
        }

        impl $crate::prelude::BlazeMapId for $new_type
        {
            type OrigType = $orig_type;
            type TypeInfoContainer = $crate::type_info_containers::key_wrapper_bounded::StaticContainer<$orig_type, $capacity>;

            #[inline]
            fn get_offset(self) -> usize {
                self.0.into_offset()
            }

            #[inline]
            unsafe fn from_offset_unchecked(offset: usize) -> Self {
                Self($crate::utils::OffsetProvider::<usize>::new(offset))
            }
        }

        #[cfg(not(loom))]
        impl $crate::traits::BlazeMapIdStatic for $new_type
        {
            #[inline]
            fn static_container() -> &'static Self::TypeInfoContainer
            {
                use $crate::type_info_containers::key_wrapper_bounded::StaticContainer;
                use $crate::external::once_cell::sync::Lazy;
                static MAP: Lazy<StaticContainer<$orig_type, $capacity>> = Lazy::new(StaticContainer::new);
                &MAP
            }
        }

        impl $crate::prelude::BlazeMapIdWrapper for $new_type
        {
            #[inline]
            unsafe fn new(type_info_container: &Self::TypeInfoContainer, key: $orig_type) -> Self {
                use $crate::traits::WrapKey;
                type_info_container.wrap_key(key)
            }
        }
    }
}
