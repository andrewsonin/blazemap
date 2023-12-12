#[doc(hidden)]
pub trait OrigTypeIdMap<K>
{
    fn num_elems(&self) -> usize;

    fn get_index(&self, key: &K) -> Option<usize>;

    unsafe fn get_key_unchecked(&self, index: usize) -> &K;

    unsafe fn insert_new_key_unchecked(&mut self, key: K) -> usize;
}