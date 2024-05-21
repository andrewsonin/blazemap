use crate::{
    prelude::AllInstancesIter,
    traits::{BlazeMapId, TypeInfoContainer},
};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};
use std::{cmp::Ordering, marker::PhantomData};

/// Provides `PartialOrd`, `Ord` and `Serialize` traits, which are derived as
/// for an original type, for [`BlazeMapId`]s in the
/// [`loom`](crate::external::loom) context.
#[derive(Debug, Copy, Clone)]
pub struct TestableId<'a, I, C> {
    id: I,
    type_info_container: &'a C,
}

impl<'a, I, C> TestableId<'a, I, C>
where
    I: BlazeMapId<TypeInfoContainer = C>,
    C: TypeInfoContainer,
{
    /// Creates a new instance of [`TestableId`].
    ///
    /// # Safety
    /// Mustn't be used outside of loom tests,
    /// since there is no guarantee that one [`BlazeMapId`]
    /// doesn't interact with different containers of the same type.
    #[inline]
    pub fn new(id: I, type_info_container: &'a C) -> Self {
        Self {
            id,
            type_info_container,
        }
    }

    /// Creates an iterator over all identifiers registered.
    #[inline]
    #[must_use]
    pub fn all_instances_iter(&self) -> AllInstancesIter<I> {
        use crate::traits::CapacityInfoProvider;
        let num_elems = self
            .type_info_container
            .capacity_info_provider()
            .offset_capacity();
        AllInstancesIter {
            range: 0..num_elems,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, C> PartialEq for TestableId<'a, I, C>
where
    I: BlazeMapId<TypeInfoContainer = C> + PartialEq,
    C: TypeInfoContainer,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        assert!(std::ptr::eq(
            self.type_info_container,
            other.type_info_container,
        ));
        self.id.eq(&other.id)
    }
}

impl<'a, I, C> Eq for TestableId<'a, I, C>
where
    I: BlazeMapId<TypeInfoContainer = C> + Eq,
    C: TypeInfoContainer,
{
}

impl<'a, I, C> PartialOrd for TestableId<'a, I, C>
where
    I: BlazeMapId<TypeInfoContainer = C> + PartialEq,
    C: TypeInfoContainer,
    C::OrigType: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use crate::traits::KeyByOffsetProvider;
        use std::borrow::Borrow;
        assert!(std::ptr::eq(
            self.type_info_container,
            other.type_info_container,
        ));
        let guard = self.type_info_container.key_by_offset_provider();
        let (lhs, rhs) = unsafe {
            (
                guard.key_by_offset_unchecked(self.id.get_offset()),
                guard.key_by_offset_unchecked(other.id.get_offset()),
            )
        };
        lhs.borrow().partial_cmp(rhs.borrow())
    }
}

impl<'a, I, C> Ord for TestableId<'a, I, C>
where
    I: BlazeMapId<TypeInfoContainer = C> + Eq,
    C: TypeInfoContainer,
    C::OrigType: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        use crate::traits::KeyByOffsetProvider;
        use std::borrow::Borrow;
        assert!(std::ptr::eq(
            self.type_info_container,
            other.type_info_container,
        ));
        let guard = self.type_info_container.key_by_offset_provider();
        let (lhs, rhs) = unsafe {
            (
                guard.key_by_offset_unchecked(self.id.get_offset()),
                guard.key_by_offset_unchecked(other.id.get_offset()),
            )
        };
        lhs.borrow().cmp(rhs.borrow())
    }
}

#[cfg(feature = "serde")]
impl<'a, I, C> Serialize for TestableId<'a, I, C>
where
    I: BlazeMapId<TypeInfoContainer = C>,
    C: TypeInfoContainer,
    C::OrigType: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use crate::traits::KeyByOffsetProvider;
        use ::std::borrow::Borrow;

        unsafe {
            self.type_info_container
                .key_by_offset_provider()
                .key_by_offset_unchecked(self.id.get_offset())
                .borrow()
                .serialize(serializer)
        }
    }
}
