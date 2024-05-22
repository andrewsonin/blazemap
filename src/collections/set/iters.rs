use crate::prelude::BlazeSet;
use std::marker::PhantomData;

/// An iterator over the entries of a [`BlazeSet`].
///
/// This `struct` is created by the [`iter`] method on [`BlazeSet`]. See its
/// documentation for more.
///
/// [`iter`]: BlazeSet::iter
pub struct Iter<'a, K> {
    phantom: PhantomData<&'a K>,
}

/// An owning iterator over the entries of a [`BlazeSet`].
///
/// This `struct` is created by the [`into_iter`] method on [`BlazeSet`]
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
pub struct IntoIter<K> {
    inner: BlazeSet<K>,
}

/// A draining iterator over the entries of a [`BlazeSet`].
///
/// This `struct` is created by the [`drain`] method on [`BlazeSet`]. See its
/// documentation for more.
///
/// [`drain`]: BlazeSet::drain
pub struct Drain<'a, K> {
    set: &'a mut BlazeSet<K>,
}

impl<K> Iterator for Iter<'_, K> {
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<K> Iterator for IntoIter<K> {
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<K> Iterator for Drain<'_, K> {
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
