/// Creates a new type that acts as an `usize`-based replacement for the old
/// type that can be used as a key for `blazemap` collections.
///
/// This macro supports optional inference of standard traits using the
/// following syntax:
///
/// * `Derive(as for Original Type)` — derives traits as for the original type
///   for which `blazemap_key` is being registered. Each call to methods on
///   these traits requires an additional `.read` call on the internal
///   synchronization primitive, so — all other things being equal — their calls
///   may be less optimal than the corresponding calls on instances of the
///   original key's type. This method supports inference of the following
///   traits:
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
///   called. Because methods inferred by this option do not require additional
///   locking on synchronization primitives, they do not incur any additional
///   overhead compared to methods inferred for plain `usize`. This method
///   supports inference of the following traits:
///   * `PartialOrd` (mutually exclusive with `Ord`)
///   * `Ord` (also derives `PartialOrd`, so mutually exclusive with
///     `PartialOrd`)
///
/// # Example
///
/// ```rust
/// use blazemap::{prelude::Map, define_key_wrapper};
///
/// define_key_wrapper! {
///     pub struct Key(&'static str);
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
/// let mut map = Map::new();
/// map.insert(key_2, "2");
/// map.insert(key_1, "1");
/// map.insert(key_3, "3");
///
/// assert_eq!(format!("{map:?}"), r#"{"first": "1", "second": "2", "third": "3"}"#)
/// ```
#[macro_export]
macro_rules! define_key_wrapper {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty)
        $(; Derive(as for Original Type): {$($to_derive_orig:ident),+ $(,)?} )?
        $(; Derive(as for usize):         {$(  $to_derive_sn:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::key_wrapper_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type($orig_type)
        }
        $($($crate::key_wrapper_derive!     {@DERIVE $to_derive_orig $new_type})*)?
        $($($crate::assigned_offset_derive! {@DERIVE   $to_derive_sn $new_type})*)?
    };
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty)
        $(; Derive(as for usize):         {$(  $to_derive_sn:ident),+ $(,)?} )?
        $(; Derive(as for Original Type): {$($to_derive_orig:ident),+ $(,)?} )?
        $(;)?
    ) => {
        $crate::key_wrapper_inner! {
            $(#[$attrs])*
            $vis
            struct $new_type($orig_type)
        }
        $($($crate::key_wrapper_derive!     {@DERIVE $to_derive_orig $new_type})*)?
        $($($crate::assigned_offset_derive! {@DERIVE   $to_derive_sn $new_type})*)?
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! key_wrapper_inner {
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty)
    ) => {
        $(#[$attrs])*
        #[derive(Clone, Copy, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $vis struct $new_type($crate::utils::OffsetProvider<usize>);

        #[cfg(not(feature = "loom"))]
        impl $new_type
        {
            #[doc = ::std::concat!("Creates a new instance of [`", ::std::stringify!($new_type), "`].")]
            #[inline]
            $vis fn new(value: $orig_type) -> Self {
                use $crate::traits::BlazeMapIdStatic;
                unsafe { <Self as $crate::prelude::BlazeMapIdWrapper>::new(Self::static_container(), value) }
            }
        }

        impl $crate::prelude::BlazeMapId for $new_type
        {
            type OrigType = $orig_type;
            type TypeInfoContainer = $crate::sync::RwLock<$crate::type_info_containers::key_wrapper::StaticContainer<$orig_type>>;

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
                use $crate::sync::RwLock;
                use $crate::type_info_containers::key_wrapper::StaticContainer;

                static MAP: RwLock<StaticContainer<$orig_type>> = RwLock::new(StaticContainer::new());
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

#[doc(hidden)]
#[macro_export]
macro_rules! key_wrapper_derive {
    (@DERIVE Default $new_type:ident) => {
        impl Default for $new_type {
            #[inline]
            fn default() -> Self {
                Self::new(Default::default())
            }
        }
    };
    (@DERIVE PartialOrd $new_type:ident) => {
        impl PartialOrd for $new_type {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                use ::std::borrow::Borrow;
                use $crate::traits::{KeyByOffsetProvider, TypeInfoContainer};
                let Self(lhs) = self;
                let Self(rhs) = other;
                let guard = <Self as $crate::prelude::BlazeMapIdStatic>::static_container()
                    .key_by_offset_provider();
                let (lhs, rhs) = unsafe {
                    (
                        guard.key_by_offset_unchecked(lhs.into_offset()),
                        guard.key_by_offset_unchecked(rhs.into_offset()),
                    )
                };
                lhs.borrow().partial_cmp(rhs.borrow())
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
                use ::std::borrow::Borrow;
                use $crate::traits::{KeyByOffsetProvider, TypeInfoContainer};

                let Self(lhs) = self;
                let Self(rhs) = other;
                let guard = <Self as $crate::prelude::BlazeMapIdStatic>::static_container()
                    .key_by_offset_provider();
                let (lhs, rhs) = unsafe {
                    (
                        guard.key_by_offset_unchecked(lhs.into_offset()),
                        guard.key_by_offset_unchecked(rhs.into_offset()),
                    )
                };
                lhs.borrow().cmp(rhs.borrow())
            }
        }
    };
    (@DERIVE Debug $new_type:ident) => {
        impl ::std::fmt::Debug for $new_type {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                use ::std::borrow::Borrow;
                use $crate::traits::{KeyByOffsetProvider, TypeInfoContainer};

                let mut f = f.debug_struct(::std::stringify!($new_type));
                let offset = self.0.into_offset();
                let guard = <Self as $crate::prelude::BlazeMapIdStatic>::static_container()
                    .key_by_offset_provider();
                let original_key = unsafe { guard.key_by_offset_unchecked(offset) };
                f.field("original_key", original_key.borrow());
                drop(original_key);
                drop(guard);
                f.field("offset", &offset).finish()
            }
        }
    };
    (@DERIVE Display $new_type:ident) => {
        impl ::std::fmt::Display for $new_type {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                use ::std::borrow::Borrow;
                use $crate::traits::{KeyByOffsetProvider, TypeInfoContainer};

                let guard = <Self as $crate::prelude::BlazeMapIdStatic>::static_container()
                    .key_by_offset_provider();
                let original_key = unsafe { guard.key_by_offset_unchecked(self.0.into_offset()) };
                write!(f, "{}", original_key.borrow())
            }
        }
    };
    (@DERIVE Deserialize $new_type:ident) => {
        impl<'de> $crate::external::serde::Deserialize<'de> for $new_type {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: $crate::external::serde::Deserializer<'de>,
            {
                use $crate::traits::BlazeMapIdStatic;
                let original_key: <Self as $crate::prelude::BlazeMapId>::OrigType =
                    $crate::external::serde::Deserialize::deserialize(deserializer)?;
                Ok(unsafe {
                    <Self as $crate::prelude::BlazeMapIdWrapper>::new(
                        Self::static_container(),
                        original_key,
                    )
                })
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
                use ::std::borrow::Borrow;
                use $crate::traits::{KeyByOffsetProvider, TypeInfoContainer};

                unsafe {
                    <Self as $crate::prelude::BlazeMapIdStatic>::static_container()
                        .key_by_offset_provider()
                        .key_by_offset_unchecked(self.0.into_offset())
                        .borrow()
                        .serialize(serializer)
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! assigned_offset_derive {
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
}
