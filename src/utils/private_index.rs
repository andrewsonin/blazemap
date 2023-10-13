use std::hash::Hash;

use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
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