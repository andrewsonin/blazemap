/// Creates a new type that is compatible as a key-wrapper for `blazemap` collections.
///
/// # Example
///
/// ```rust
/// use blazemap::prelude::{BlazeMap, register_blazemap_key};
///
/// register_blazemap_key! {
///     pub struct BlazeMapKeyExample(&'static str);
///     DERIVE AS FOR ORIGINAL TYPE: {  // Optional section
///         Debug,
///         Display,
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
macro_rules! register_blazemap_key {
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
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                let Self(lhs) = self;
                let Self(rhs) = other;
                let guard = <Self as $crate::prelude::KeyWrapper>::static_info().read();
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
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let Self(lhs) = self;
                let Self(rhs) = other;
                let guard = <Self as $crate::prelude::KeyWrapper>::static_info().read();
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
                let Self(index) = self;
                let mut f = f.debug_struct(::std::stringify!($new_type));
                let guard = <Self as $crate::prelude::KeyWrapper>::static_info().read();
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
                let Self(index) = self;
                let guard = <Self as $crate::prelude::KeyWrapper>::static_info().read();
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
                let original_key: <Self as $crate::prelude::KeyWrapper>::OrigType
                    = $crate::external::serde::Deserialize::deserialize(deserializer)?;
                Ok(<Self as $crate::prelude::KeyWrapper>::new(original_key))
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
                unsafe {
                    <Self as $crate::prelude::KeyWrapper>::static_info()
                        .read()
                        .get_key_unchecked(index.into_inner())
                        .serialize(serializer)
                }
            }
        }
    };
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty)
    ) => {
        $(#[$attrs])*
        #[derive(Clone, Copy, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        struct $new_type($crate::utils::PrivateIndex);

        impl $new_type
        {
            #[inline]
            pub fn new(value: $orig_type) -> Self {
                <Self as $crate::prelude::KeyWrapper>::new(value)
            }
        }

        impl $crate::prelude::KeyWrapper for $new_type
        {
            type OrigType = $orig_type;

            #[inline]
            fn get_index(&self) -> usize {
                let Self(index) = self;
                index.into_inner()
            }

            #[inline(always)]
            unsafe fn from_index_unchecked(index: usize) -> Self {
                Self($crate::utils::PrivateIndex::new(index))
            }

            #[inline]
            fn static_info() -> &'static $crate::external::parking_lot::RwLock<$crate::utils::StaticInfo<$orig_type>>
            {
                use $crate::external::once_cell::sync::Lazy;
                use $crate::external::parking_lot::RwLock;
                use $crate::utils::StaticInfo;
                static MAP: Lazy<RwLock<StaticInfo<$orig_type>>> = Lazy::new(
                    || RwLock::new(StaticInfo::new())
                );
                &MAP
            }
        }
    };
    (
        $(#[$attrs:meta])*
        $vis:vis
        struct $new_type:ident($orig_type:ty);
        DERIVE AS FOR ORIGINAL TYPE: {$($to_derive:ident),+ $(,)?}
    ) => {
        register_blazemap_key! {
            $(#[$attrs])*
            $vis
            struct $new_type($orig_type)
        }
        $(register_blazemap_key! {@DERIVE $to_derive $new_type})*
    }
}