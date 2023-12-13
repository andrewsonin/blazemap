use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::Lazy;

use crate::orig_type_id_map::{InsertableStaticInfoApi, StaticInfoApi};

/// Global, statically initialized structure that contains correspondence mapping
/// between blazemap index wrappers and original keys.
#[doc(hidden)]
#[derive(Debug)]
pub struct IdWrapperStaticInfo<K> {
    index_to_orig: Vec<K>,
    orig_to_index: Lazy<HashMap<K, usize>>,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct TrivialIdStaticInfo {
    next_id: AtomicUsize,
}

impl<K> IdWrapperStaticInfo<K>
{
    /// Creates a new instance of [`IdWrapperStaticInfo`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            index_to_orig: vec![],
            orig_to_index: Lazy::new(Default::default),
        }
    }
}

impl TrivialIdStaticInfo
{
    /// Creates a new instance of [`TrivialIdStaticInfo`].
    #[inline]
    pub const fn new(first_id: usize) -> Self {
        Self {
            next_id: AtomicUsize::new(first_id),
        }
    }

    /// Returns the next identifier.
    #[inline]
    pub fn next_id(&self) -> usize {
        self.next_id.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |next_id| next_id.checked_add(1),
        )
            .expect("usize overflow")
    }
}

impl<K> StaticInfoApi<K> for IdWrapperStaticInfo<K>
    where
        K: Clone + Eq + Hash
{
    type KeyUnchecked<'a> = &'a K
        where Self: 'a;

    #[inline(always)]
    fn num_elems(&self) -> usize {
        self.index_to_orig.len()
    }

    #[inline]
    unsafe
    fn get_key_unchecked(&self, index: usize) -> &K {
        self.index_to_orig.get_unchecked(index)
    }
}

impl<K> InsertableStaticInfoApi<K> for IdWrapperStaticInfo<K>
    where
        K: Clone + Eq + Hash
{
    #[inline]
    fn get_index(&self, key: &K) -> Option<usize> {
        self.orig_to_index.get(key).copied()
    }

    #[inline]
    unsafe
    fn insert_new_key_unchecked(&mut self, key: K) -> usize
    {
        let next_id = self.num_elems();
        let Self {
            index_to_orig,
            orig_to_index
        } = self;
        index_to_orig.push(key.clone());
        orig_to_index.insert(key, next_id);
        next_id
    }
}

impl StaticInfoApi<usize> for TrivialIdStaticInfo
{
    type KeyUnchecked<'a> = usize
        where Self: 'a;

    #[inline(always)]
    fn num_elems(&self) -> usize {
        self.next_id.load(Ordering::Relaxed)
    }

    #[inline(always)]
    unsafe fn get_key_unchecked(&self, index: usize) -> usize {
        index
    }
}