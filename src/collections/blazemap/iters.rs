use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::collections::blazemap::BlazeMap;
use crate::prelude::{BlazeMapId, BlazeMapIdStatic};
use crate::traits::{KeyByOffsetProvider, TypeInfoContainer};

/// An iterator over the entries of a [`BlazeMap`].
///
/// This `struct` is created by the [`iter`] method on [`BlazeMap`]. See its
/// documentation for more.
///
/// [`iter`]: BlazeMap::iter
pub struct Iter<'a, K, V> {
    pub(in crate::collections::blazemap) inner: *const Option<V>,

    pub(in crate::collections::blazemap) current_position: usize,

    pub(in crate::collections::blazemap) len: usize,

    pub(in crate::collections::blazemap) phantom: PhantomData<(K, &'a V)>,
}

/// A mutable iterator over the entries of a [`BlazeMap`].
///
/// This `struct` is created by the [`iter_mut`] method on [`BlazeMap`]. See its
/// documentation for more.
///
/// [`iter_mut`]: BlazeMap::iter_mut
pub struct IterMut<'a, K, V> {
    pub(in crate::collections::blazemap) inner: *mut Option<V>,

    pub(in crate::collections::blazemap) current_position: usize,

    pub(in crate::collections::blazemap) len: usize,

    pub(in crate::collections::blazemap) phantom: PhantomData<(K, &'a mut V)>,
}

/// An iterator over the keys of a [`BlazeMap`].
///
/// This `struct` is created by the [`keys`] method on [`BlazeMap`]. See its
/// documentation for more.
///
/// [`keys`]: BlazeMap::keys
pub struct Keys<'a, K, V> {
    pub(in crate::collections::blazemap) inner: Iter<'a, K, V>,
}

/// An iterator over the values of a [`BlazeMap`].
///
/// This `struct` is created by the [`values`] method on [`BlazeMap`]. See its
/// documentation for more.
///
/// [`values`]: BlazeMap::values
pub struct Values<'a, K, V> {
    pub(in crate::collections::blazemap) inner: Iter<'a, K, V>,
}

/// A mutable iterator over the values of a [`BlazeMap`].
///
/// This `struct` is created by the [`values_mut`] method on [`BlazeMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: BlazeMap::values_mut
pub struct ValuesMut<'a, K, V> {
    pub(in crate::collections::blazemap) inner: IterMut<'a, K, V>,
}

/// An owning iterator over the entries of a [`BlazeMap`].
///
/// This `struct` is created by the [`into_iter`] method on [`BlazeMap`]
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
pub struct IntoIter<K, V> {
    pub(in crate::collections::blazemap) inner: BlazeMap<K, V>,
}

/// An owning iterator over the keys of a [`BlazeMap`].
///
/// This `struct` is created by the [`into_keys`] method on [`BlazeMap`].
/// See its documentation for more.
///
/// [`into_keys`]: BlazeMap::into_keys
pub struct IntoKeys<K, V> {
    pub(in crate::collections::blazemap) inner: IntoIter<K, V>,
}

/// An owning iterator over the values of a [`BlazeMap`].
///
/// This `struct` is created by the [`into_values`] method on [`BlazeMap`].
/// See its documentation for more.
///
/// [`into_values`]: BlazeMap::into_values
pub struct IntoValues<K, V> {
    pub(in crate::collections::blazemap) inner: IntoIter<K, V>,
}

/// A draining iterator over the entries of a [`BlazeMap`].
///
/// This `struct` is created by the [`drain`] method on [`BlazeMap`]. See its
/// documentation for more.
///
/// [`drain`]: BlazeMap::drain
pub struct Drain<'a, K, V> {
    pub(in crate::collections::blazemap) map: &'a mut BlazeMap<K, V>,

    pub(in crate::collections::blazemap) current_position: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: BlazeMapId,
{
    type Item = (K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(K, &'a V)> {
        let Self {
            inner,
            current_position,
            len,
            ..
        } = self;
        if *len == 0 {
            return None;
        }
        unsafe {
            loop {
                match &*inner.add(*current_position) {
                    None => {
                        *current_position += 1;
                        continue;
                    }
                    Some(value) => {
                        let key = K::from_offset_unchecked(*current_position);
                        *current_position += 1;
                        *len -= 1;
                        return Some((key, value));
                    }
                }
            }
        }
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: BlazeMapId,
{
    type Item = (K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(K, &'a mut V)> {
        if self.len == 0 {
            return None;
        }
        unsafe {
            loop {
                match &mut *self.inner.add(self.current_position) {
                    None => {
                        self.current_position += 1;
                        continue;
                    }
                    Some(value) => {
                        let key = K::from_offset_unchecked(self.current_position);
                        self.current_position += 1;
                        self.len -= 1;
                        return Some((key, value));
                    }
                }
            }
        }
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V>
where
    K: BlazeMapId,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<K> {
        let Iter {
            inner,
            current_position,
            len,
            ..
        } = &mut self.inner;
        if *len == 0 {
            return None;
        }
        unsafe {
            loop {
                match &*inner.add(*current_position) {
                    None => {
                        *current_position += 1;
                        continue;
                    }
                    Some(_) => {
                        let key = K::from_offset_unchecked(*current_position);
                        *current_position += 1;
                        *len -= 1;
                        return Some(key);
                    }
                }
            }
        }
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<&'a V> {
        let Iter {
            inner,
            current_position,
            len,
            ..
        } = &mut self.inner;
        if *len == 0 {
            return None;
        }
        loop {
            match unsafe { &*inner.add(*current_position) } {
                None => {
                    *current_position += 1;
                    continue;
                }
                Some(value) => {
                    *current_position += 1;
                    *len -= 1;
                    return Some(value);
                }
            }
        }
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<&'a mut V> {
        let inner = &mut self.inner;
        if inner.len == 0 {
            return None;
        }
        loop {
            match unsafe { &mut *inner.inner.add(inner.current_position) } {
                None => {
                    inner.current_position += 1;
                    continue;
                }
                Some(value) => {
                    inner.current_position += 1;
                    inner.len -= 1;
                    return Some(value);
                }
            }
        }
    }
}

impl<'a, K, V> ExactSizeIterator for ValuesMut<'a, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len
    }
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: BlazeMapId,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        let BlazeMap { inner, len, .. } = &mut self.inner;
        while let Some(back) = inner.pop() {
            if let Some(value) = back {
                let key = unsafe { K::from_offset_unchecked(inner.len()) };
                *len -= 1;
                return Some((key, value));
            }
        }
        None
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len
    }
}

impl<K, V> Iterator for IntoKeys<K, V>
where
    K: BlazeMapId,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<K> {
        let BlazeMap { inner, len, .. } = &mut self.inner.inner;
        while let Some(back) = inner.pop() {
            if back.is_some() {
                let key = unsafe { K::from_offset_unchecked(inner.len()) };
                *len -= 1;
                return Some(key);
            }
        }
        None
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<V> {
        let BlazeMap { inner, len, .. } = &mut self.inner.inner;
        while let Some(back) = inner.pop() {
            if let Some(value) = back {
                *len -= 1;
                return Some(value);
            }
        }
        None
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K, V> Iterator for Drain<'a, K, V>
where
    K: BlazeMapId,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        if self.map.len == 0 {
            return None;
        }
        unsafe {
            loop {
                let value = &mut *self.map.inner.as_mut_ptr().add(self.current_position);
                match value.take() {
                    None => {
                        self.current_position += 1;
                        continue;
                    }
                    Some(value) => {
                        let key = K::from_offset_unchecked(self.current_position);
                        self.map.len -= 1;
                        self.current_position += 1;
                        return Some((key, value));
                    }
                }
            }
        }
    }
}

impl<'a, K, V> ExactSizeIterator for Drain<'a, K, V>
where
    K: BlazeMapId,
{
    #[inline]
    fn len(&self) -> usize {
        self.map.len
    }
}

impl<'a, K, V> Drop for Drain<'a, K, V> {
    #[inline]
    fn drop(&mut self) {
        self.map.clear();
    }
}

unsafe impl<'a, K, V> Send for Iter<'a, K, V>
where
    K: Sync,
    V: Sync,
{
}

unsafe impl<'a, K, V> Sync for Iter<'a, K, V>
where
    K: Sync,
    V: Sync,
{
}

impl<'a, K, V> Unpin for Iter<'a, K, V> {}

impl<'a, K, V> UnwindSafe for Iter<'a, K, V>
where
    K: RefUnwindSafe,
    V: RefUnwindSafe,
{
}

unsafe impl<'a, K, V> Send for IterMut<'a, K, V>
where
    K: Sync,
    V: Send,
{
}

unsafe impl<'a, K, V> Sync for IterMut<'a, K, V>
where
    K: Sync,
    V: Sync,
{
}

impl<'a, K, V> Unpin for IterMut<'a, K, V> {}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            current_position: self.current_position,
            len: self.len,
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V> Debug for Iter<'a, K, V>
where
    K: BlazeMapIdStatic,
    K::OrigType: Debug,
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let key_provider = K::static_container().key_by_offset_provider();
        let mut debug_map = f.debug_map();
        for (key, value) in self.clone() {
            let key = unsafe { key_provider.key_by_offset_unchecked(key.get_offset()) };
            debug_map.entry(key.borrow(), value);
        }
        debug_map.finish()
    }
}

impl<'a, K, V> Debug for IterMut<'a, K, V>
where
    K: BlazeMapIdStatic,
    K::OrigType: Debug,
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            inner,
            current_position,
            len,
            ..
        } = self;
        let iter = Iter::<K, V> {
            inner: *inner,
            current_position: *current_position,
            len: *len,
            phantom: PhantomData,
        };
        iter.fmt(f)
    }
}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, K, V> Clone for Values<'a, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, K, V> Debug for Keys<'a, K, V>
where
    K: BlazeMapIdStatic,
    K::OrigType: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let key_provider = K::static_container().key_by_offset_provider();
        let mut debug_list = f.debug_list();
        for key in self.clone() {
            let key = unsafe { key_provider.key_by_offset_unchecked(key.get_offset()) };
            debug_list.entry(key.borrow());
        }
        debug_list.finish()
    }
}

impl<'a, K, V> Debug for Values<'a, K, V>
where
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Debug for ValuesMut<'a, K, V>
where
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let IterMut {
            inner,
            current_position,
            len,
            ..
        } = self.inner;
        let iter = Values::<K, V> {
            inner: Iter {
                inner,
                current_position,
                len,
                phantom: PhantomData,
            },
        };
        iter.fmt(f)
    }
}

impl<K, V> Debug for IntoIter<K, V>
where
    K: BlazeMapIdStatic,
    K::OrigType: Debug,
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<K, V> Debug for IntoKeys<K, V>
where
    K: BlazeMapIdStatic,
    K::OrigType: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.inner.keys().fmt(f)
    }
}

impl<K, V> Debug for IntoValues<K, V>
where
    K: BlazeMapId,
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.inner.values().fmt(f)
    }
}

impl<'a, K, V> Debug for Drain<'a, K, V>
where
    K: BlazeMapIdStatic,
    K::OrigType: Debug,
    V: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}
