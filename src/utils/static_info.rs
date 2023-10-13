use std::collections::HashMap;
use std::hash::Hash;

use once_cell::sync::Lazy;

/// Global, statically initialized structure that contains correspondence mapping
/// between blazemap index wrappers and original keys.
#[doc(hidden)]
#[derive(Debug)]
pub struct StaticInfo<K> {
    index_to_orig: Vec<K>,
    orig_to_index: Lazy<HashMap<K, usize>>,
}

impl<K> StaticInfo<K>
{
    /// Creates a new instance of [`StaticInfo`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            index_to_orig: vec![],
            orig_to_index: Lazy::new(Default::default),
        }
    }
}

impl<K> StaticInfo<K>
    where
        K: Clone + Eq + Hash
{
    #[inline(always)]
    pub(in crate)
    fn num_elems(&self) -> usize {
        self.index_to_orig.len()
    }

    #[inline]
    pub(in crate)
    fn get_index(&self, key: &K) -> Option<usize> {
        self.orig_to_index.get(key).copied()
    }

    #[inline]
    pub
    unsafe
    fn get_key_unchecked(&self, index: usize) -> &K {
        self.index_to_orig.get_unchecked(index)
    }

    #[inline]
    pub(in crate)
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