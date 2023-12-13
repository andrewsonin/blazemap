/// Creates a new type that is compatible as a key-wrapper for `blazemap` collections.
/// Use this macro if you want to wrap your clumsy types in `blazemap` collection keys.
///
/// This macro supports optional inference of standard traits using the following syntax:
///
/// * `Derive(as for Original Type)` — derives traits as for the original type
///   for which `blazemap_key` is being registered. Each call to methods on these traits
///   requires an additional `.read` call on the internal synchronization primitive,
///   so — all other things being equal — their calls may be less optimal
///   than the corresponding calls on instances of the original key's type.
///   This method supports inference of the following traits:
///   * `Default`
///   * `PartialOrd` (mutually exclusive with `Ord`)
///   * `Ord` (also derives `PartialOrd`, so mutually exclusive with `PartialOrd`)
///   * `Debug`
///   * `Display`
///   * `Serialize` (with `serde` feature only)
///   * `Deserialize` (with `serde` feature only)
/// * `Derive(as for Serial Number)` — derives traits in the same way as for
///   the serial number assigned when registering an instance of the original type
///   the first time [`IdWrapper::new`](crate::prelude::BlazeMapIdWrapper::new) was called.
///   Because methods inferred by this option do not require additional
///   locking on synchronization primitives,
///   they do not incur any additional overhead compared to methods inferred for plain `usize`.
///   This method supports inference of the following traits:
///   * `PartialOrd` (mutually exclusive with `Ord`)
///   * `Ord` (also derives `PartialOrd`, so mutually exclusive with `PartialOrd`)
///
/// # Example
///
/// ```rust
/// use blazemap::prelude::{BlazeMap, register_blazemap_id_wrapper};
///
/// register_blazemap_id_wrapper! {
///     pub struct BlazeMapKeyExample(&'static str);
///     Derive(as for Original Type): {  // Optional section
///         Debug,
///         Display,
///     };
///     Derive(as for Serial Number): {  // Optional section
///         Ord,
///     }
/// }
///
/// let key_1 = BlazeMapKeyExample::new("first");
/// let key_2 = BlazeMapKeyExample::new("second");
/// let key_3 = BlazeMapKeyExample::new("third");
///
/// let mut map = BlazeMap::new();
/// map.insert(key_2, "2");
/// map.insert(key_1, "1");
/// map.insert(key_3, "3");
///
/// assert_eq!(format!("{map:?}"), r#"{"first": "1", "second": "2", "third": "3"}"#)
/// ```
#[macro_export]
macro_rules! register_blazemap_id_wrapper {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty)
        $(; Derive(as for Original Type): {$($to_derive_orig:ident),+ $(,)?} )?
        $(; Derive(as for Serial Number): {$(  $to_derive_sn:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::blazemap_id_wrapper_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type($orig_type)
        }
        $($($crate::blazemap_derive_key_inner!   {@DERIVE $to_derive_orig $new_type})*)?
        $($($crate::blazemap_derive_assigned_sn! {@DERIVE   $to_derive_sn $new_type})*)?
    };
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty)
        $(; Derive(as for Serial Number): {$(  $to_derive_sn:ident),+ $(,)?} )?
        $(; Derive(as for Original Type): {$($to_derive_orig:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::blazemap_id_wrapper_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type($orig_type)
        }
        $($($crate::blazemap_derive_key_inner!   {@DERIVE $to_derive_orig $new_type})*)?
        $($($crate::blazemap_derive_assigned_sn! {@DERIVE   $to_derive_sn $new_type})*)?
    }
}

/// Creates a new type based on `usize` that is compatible as a key-wrapper for `blazemap` collections.
///
/// This macro supports optional inference of standard traits using the following syntax:
///
/// * `Derive` — derives traits in the same way as for
///   the serial number assigned when creating a new instance of the type.
///   Because methods inferred by this option do not require additional
///   locking on synchronization primitives,
///   they do not incur any additional overhead compared to methods inferred for plain `usize`.
///   This method supports inference of the following traits:
///   * `PartialOrd` (mutually exclusive with `Ord`)
///   * `Ord` (also derives `PartialOrd`, so mutually exclusive with `PartialOrd`)
///   * `Serialize` (with `serde` feature only)
///
/// # Example
///
/// ```rust
/// use blazemap::prelude::{BlazeMap, register_blazemap_id};
///
/// register_blazemap_id! {
///     pub struct BlazeMapIdExample(start from: 1);  // "(start from: number)" is optional
///     Derive: {                                     // Derive section is also optional
///         Ord
///     };
/// }
///
/// let key_1 = BlazeMapIdExample::new();
/// let key_2 = BlazeMapIdExample::new();
/// let key_3 = BlazeMapIdExample::new();
///
/// let mut map = BlazeMap::new();
/// map.insert(key_2, "2");
/// map.insert(key_1, "1");
/// map.insert(key_3, "3");
///
/// assert_eq!(format!("{map:?}"), r#"{1: "1", 2: "2", 3: "3"}"#)
/// ```
#[macro_export]
macro_rules! register_blazemap_id {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident(start from: $first_id:literal)
        $(; Derive: {$($to_derive_sn:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::blazemap_id_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type($first_id)
        }
        $($($crate::blazemap_id_inner_derive! {@DERIVE $to_derive_sn $new_type})*)?
    };
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident
        $(; Derive: {$($to_derive_sn:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::blazemap_id_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type(0)
        }
        $($($crate::blazemap_id_inner_derive! {@DERIVE $to_derive_sn $new_type})*)?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! blazemap_id_wrapper_inner {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty)
    ) => {
        $(#[$attrs])*
        #[derive(Clone, Copy, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $vis struct $new_type($crate::utils::PrivateIndex);

        impl $new_type
        {
            #[inline]
            pub fn new(value: $orig_type) -> Self {
                <Self as $crate::prelude::BlazeMapIdWrapper>::new(value)
            }
        }

        impl $crate::prelude::BlazeMapId for $new_type
        {
            type OrigType = $orig_type;
            type StaticInfoApi = $crate::utils::IdWrapperStaticInfo<$orig_type>;
            type StaticInfoApiLock = &'static $crate::external::parking_lot::RwLock<$crate::utils::IdWrapperStaticInfo<$orig_type>>;

            #[inline]
            fn get_index(self) -> usize {
                let Self(index) = self;
                index.into_inner()
            }

            #[inline(always)]
            unsafe fn from_index_unchecked(index: usize) -> Self {
                Self($crate::utils::PrivateIndex::new(index))
            }

            #[inline]
            fn static_info() -> &'static $crate::external::parking_lot::RwLock<$crate::utils::IdWrapperStaticInfo<$orig_type>>
            {
                use $crate::external::once_cell::sync::Lazy;
                use $crate::external::parking_lot::RwLock;
                use $crate::utils::IdWrapperStaticInfo;

                static MAP: Lazy<RwLock<IdWrapperStaticInfo<$orig_type>>> = Lazy::new(
                    || RwLock::new(IdWrapperStaticInfo::new())
                );
                &MAP
            }
        }

        impl $crate::prelude::BlazeMapIdWrapper for $new_type
        {}
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! blazemap_id_inner {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($first_id:literal)
    ) => {
        $(#[$attrs])*
        #[derive(Clone, Copy, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $vis struct $new_type($crate::utils::PrivateIndex);

        impl $new_type
        {
            #[inline]
            #[allow(dead_code)]
            pub fn new() -> Self {
                use $crate::prelude::BlazeMapId;

                let next_id = Self::static_info().0.next_id();
                Self(unsafe { $crate::utils::PrivateIndex::new(next_id) })
            }
        }

        impl $crate::prelude::BlazeMapId for $new_type
        {
            type OrigType = usize;
            type StaticInfoApi = $crate::utils::TrivialIdStaticInfo;
            type StaticInfoApiLock = &'static $crate::external::read_write_api::RwApiWrapperOwned<$crate::utils::TrivialIdStaticInfo>;

            #[inline]
            fn get_index(self) -> usize {
                let Self(index) = self;
                index.into_inner()
            }

            #[inline(always)]
            unsafe fn from_index_unchecked(index: usize) -> Self {
                Self($crate::utils::PrivateIndex::new(index))
            }

            #[inline]
            fn static_info() -> &'static $crate::external::read_write_api::RwApiWrapperOwned<$crate::utils::TrivialIdStaticInfo>
            {
                use $crate::utils::TrivialIdStaticInfo;
                use $crate::external::read_write_api::RwApiWrapperOwned;

                static INFO: RwApiWrapperOwned<TrivialIdStaticInfo> = RwApiWrapperOwned(TrivialIdStaticInfo::new($first_id));
                &INFO
            }
        }

        impl ::std::fmt::Debug for $new_type
        {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
            {
                let Self(index) = self;
                f.debug_tuple(::std::stringify!($new_type))
                    .field(&index.into_inner())
                    .finish()
            }
        }

        impl ::std::fmt::Display for $new_type
        {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
            {
                let Self(index) = self;
                write!(f, "{}", index.into_inner())
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! blazemap_derive_key_inner {
    (@DERIVE Default $new_type:ident) => {
        impl Default for $new_type
        {
            #[inline]
            fn default() -> Self {
                Self::new(Default::default())
            }
        }
    };
    (@DERIVE PartialOrd $new_type:ident) => {
        impl PartialOrd for $new_type
        {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering>
            {
                use $crate::orig_type_id_map::StaticInfoApi;

                let Self(lhs) = self;
                let Self(rhs) = other;
                let guard = <Self as $crate::prelude::BlazeMapId>::static_info().read();
                let (lhs, rhs) = unsafe {
                    (
                        guard.get_key_unchecked(lhs.into_inner()),
                        guard.get_key_unchecked(rhs.into_inner()),
                    )
                };
                lhs.partial_cmp(rhs)
            }
        }
    };
    (@DERIVE Ord $new_type:ident) => {
        impl PartialOrd for $new_type
        {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $new_type
        {
            #[inline]
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering
            {
                use $crate::orig_type_id_map::StaticInfoApi;

                let Self(lhs) = self;
                let Self(rhs) = other;
                let guard = <Self as $crate::prelude::BlazeMapId>::static_info().read();
                let (lhs, rhs) = unsafe {
                    (
                        guard.get_key_unchecked(lhs.into_inner()),
                        guard.get_key_unchecked(rhs.into_inner()),
                    )
                };
                lhs.cmp(rhs)
            }
        }
    };
    (@DERIVE Debug $new_type:ident) => {
        impl ::std::fmt::Debug for $new_type
        {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
            {
                use $crate::orig_type_id_map::StaticInfoApi;

                let Self(index) = self;
                let mut f = f.debug_struct(::std::stringify!($new_type));
                let guard = <Self as $crate::prelude::BlazeMapId>::static_info().read();
                let original_key = unsafe { guard.get_key_unchecked(index.into_inner()) };
                f.field("original_key", original_key);
                drop(guard);
                f
                    .field("index", &index.into_inner())
                    .finish()
            }
        }
    };
    (@DERIVE Display $new_type:ident) => {
        impl ::std::fmt::Display for $new_type
        {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
            {
                use $crate::orig_type_id_map::StaticInfoApi;

                let Self(index) = self;
                let guard = <Self as $crate::prelude::BlazeMapId>::static_info().read();
                let original_key = unsafe { guard.get_key_unchecked(index.into_inner()) };
                write!(f, "{original_key}")
            }
        }
    };
    (@DERIVE Deserialize $new_type:ident) => {
        impl<'de> $crate::external::serde::Deserialize<'de> for $new_type
        {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: $crate::external::serde::Deserializer<'de>
            {
                let original_key: <Self as $crate::prelude::BlazeMapId>::OrigType
                    = $crate::external::serde::Deserialize::deserialize(deserializer)?;
                Ok(<Self as $crate::prelude::BlazeMapIdWrapper>::new(original_key))
            }
        }
    };
    (@DERIVE Serialize $new_type:ident) => {
        impl $crate::external::serde::Serialize for $new_type
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: $crate::external::serde::Serializer
            {
                use $crate::orig_type_id_map::StaticInfoApi;

                let Self(index) = self;
                unsafe {
                    <Self as $crate::prelude::BlazeMapId>::static_info()
                        .read()
                        .get_key_unchecked(index.into_inner())
                        .serialize(serializer)
                }
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! blazemap_derive_assigned_sn {
    (@DERIVE PartialOrd $new_type:ident) => {
        impl PartialOrd for $new_type
        {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                let Self(lhs) = self;
                let Self(rhs) = other;
                lhs.into_inner().partial_cmp(&rhs.into_inner())
            }
        }
    };
    (@DERIVE Ord $new_type:ident) => {
        impl PartialOrd for $new_type
        {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $new_type
        {
            #[inline]
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let Self(lhs) = self;
                let Self(rhs) = other;
                lhs.into_inner().cmp(&rhs.into_inner())
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! blazemap_id_inner_derive {
    (@DERIVE PartialOrd $new_type:ident) => {
        impl PartialOrd for $new_type
        {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                let Self(lhs) = self;
                let Self(rhs) = other;
                lhs.into_inner().partial_cmp(&rhs.into_inner())
            }
        }
    };
    (@DERIVE Ord $new_type:ident) => {
        impl PartialOrd for $new_type
        {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $new_type
        {
            #[inline]
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let Self(lhs) = self;
                let Self(rhs) = other;
                lhs.into_inner().cmp(&rhs.into_inner())
            }
        }
    };
    (@DERIVE Serialize $new_type:ident) => {
        impl $crate::external::serde::Serialize for $new_type
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: $crate::external::serde::Serializer
            {
                let Self(index) = self;
                index.into_inner().serialize(serializer)
            }
        }
    }
}