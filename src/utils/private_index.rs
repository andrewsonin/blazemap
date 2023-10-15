use std::hash::Hash;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
#[doc(hidden)]
pub struct PrivateIndex(usize);

impl PrivateIndex
{
    #[inline(always)]
    pub unsafe fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub fn into_inner(self) -> usize {
        self.0
    }
}