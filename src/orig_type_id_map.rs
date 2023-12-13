use std::borrow::Borrow;

#[doc(hidden)]
pub trait StaticInfoApi<K>
{
    type KeyUnchecked<'a>: Borrow<K>
        where Self: 'a;

    fn num_elems(&self) -> usize;

    unsafe fn get_key_unchecked(&self, index: usize) -> Self::KeyUnchecked<'_>;
}

#[doc(hidden)]
pub trait InsertableStaticInfoApi<K>: StaticInfoApi<K>
{
    fn get_index(&self, key: &K) -> Option<usize>;

    unsafe fn insert_new_key_unchecked(&mut self, key: K) -> usize;
}