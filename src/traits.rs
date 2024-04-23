use std::{
    borrow::Borrow,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, Range},
};

/// Provides an interface for `blazemap` id types defined by type-generating
/// macros.
pub trait BlazeMapId: Copy {
    /// Original key type.
    type OrigType: 'static + Clone + Eq + Hash;
    /// Type of the container that holds all the information necessary
    /// for `Self` to be a [`BlazeMapId`] type.
    #[doc(hidden)]
    type TypeInfoContainer: TypeInfoContainer<OrigType = Self::OrigType>;

    /// Returns the offset corresponding to the given identifier.
    #[doc(hidden)]
    fn get_offset(self) -> usize;

    /// Creates an identifier corresponding to the provided offset.
    #[doc(hidden)]
    unsafe fn from_offset_unchecked(offset: usize) -> Self;
}

/// Provides an interface for `blazemap` key-wrapper id types
/// defined by the [`define_key_wrapper`](crate::define_key_wrapper)
/// and [`define_key_wrapper_bounded`](crate::define_key_wrapper_bounded)
/// macros.
pub trait BlazeMapIdWrapper: BlazeMapId {
    /// Creates a new instance of [`Self`] based on the
    /// [`Self::OrigType`](BlazeMapId::OrigType) instance.
    unsafe fn new(type_info_container: &Self::TypeInfoContainer, key: Self::OrigType) -> Self;
}

/// Provides an interface for statically registered `blazemap` id types.
pub trait BlazeMapIdStatic: BlazeMapId {
    /// Creates an iterator over all identifiers registered.
    #[inline]
    #[must_use]
    fn all_instances_iter() -> AllInstancesIter<Self> {
        let num_elems = Self::static_container()
            .capacity_info_provider()
            .offset_capacity();
        AllInstancesIter {
            range: 0..num_elems,
            phantom: PhantomData,
        }
    }

    /// Returns the static container
    /// that holds all the necessary static information for the [`BlazeMapId`]
    /// type.
    #[doc(hidden)]
    fn static_container() -> &'static Self::TypeInfoContainer;
}

/// Implements an interface for [`BlazeMapId`] key-wrapper static containers.
#[doc(hidden)]
pub trait WrapKey<I: BlazeMapId> {
    /// Creates an instance of [`BlazeMapId`] type that is unique to the given
    /// key.
    fn wrap_key(&self, key: I::OrigType) -> I;
}

pub trait TypeInfoContainer: 'static {
    /// Original key type.
    type OrigType;

    /// Returns the provider of the current total number of registered unique
    /// `Self` identifiers. Note that this provider isn't sequentially
    /// consistent.
    #[doc(hidden)]
    fn capacity_info_provider(&self) -> impl Deref<Target = impl CapacityInfoProvider>;

    /// Returns a provider that may unsafely return
    /// the registered key corresponding to the offset specified.
    #[doc(hidden)]
    fn key_by_offset_provider(
        &self,
    ) -> impl Deref<Target = impl KeyByOffsetProvider<Self::OrigType>>;
}

/// Provides the current total number of registered unique [`BlazeMapId`]
/// identifiers. Note that there is no guarantee of sequential consistency.
#[doc(hidden)]
pub trait CapacityInfoProvider {
    /// Returns the current total number of registered unique [`BlazeMapId`]
    /// identifiers.
    fn offset_capacity(&self) -> usize;
}

/// May unsafely return the registered key corresponding to the offset
/// specified.
#[doc(hidden)]
pub trait KeyByOffsetProvider<K> {
    /// Returns the registered key corresponding to the offset specified.
    unsafe fn key_by_offset_unchecked(&self, offset: usize) -> impl Borrow<K>;
}

/// Iterator over consecutive `blazemap` identifiers.
pub struct AllInstancesIter<T> {
    pub(crate) range: Range<usize>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T> Clone for AllInstancesIter<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T> Debug for AllInstancesIter<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.range)
    }
}

impl<T> PartialEq for AllInstancesIter<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range
    }
}

impl<T> Eq for AllInstancesIter<T> {}

impl<T> Hash for AllInstancesIter<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.range.hash(state);
    }
}

impl<T> Iterator for AllInstancesIter<T>
where
    T: BlazeMapId,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next_offset = self.range.next()?;
        Some(unsafe { T::from_offset_unchecked(next_offset) })
    }
}

impl<T> DoubleEndedIterator for AllInstancesIter<T>
where
    T: BlazeMapId,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_back_offset = self.range.next_back()?;
        Some(unsafe { T::from_offset_unchecked(next_back_offset) })
    }
}

impl<T> ExactSizeIterator for AllInstancesIter<T>
where
    T: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}
