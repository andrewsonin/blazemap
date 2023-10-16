use std::hash::Hash;

/// Necessary to protect the internal `usize`, which, in the absence of this wrapper,
/// would be public in the module calling [`register_blazemap_id`](crate::register_blazemap_id).
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