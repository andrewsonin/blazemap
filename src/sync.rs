#[cfg(loom)]
pub use loom::sync::{atomic::AtomicUsize, atomic::Ordering, RwLock, RwLockReadGuard};

#[cfg(not(loom))]
pub use {
    parking_lot::RwLock,
    std::sync::atomic::{AtomicUsize, Ordering},
};
