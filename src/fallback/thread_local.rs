//! Manage the thread local fallback drop strategy.
use std::boxed::Box;
use std::cell::{Ref, RefCell, RefMut};
use once_cell::unsync::{Lazy, OnceCell};
use crate::FallbackTryDropStrategy;
use crate::utils::NotSendNotSync;
use std::{fmt, thread_local};
use std::marker::PhantomData;
use crate::on_uninit::{ErrorOnUninit, OnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::uninit_error::UninitializedError;

thread_local! {
    static DROP_STRATEGY: RefCell<Option<Box<dyn FallbackTryDropStrategy>>> = RefCell::new(None);
}

const UNINITIALIZED_ERROR: &str = "the thread local fallback drop strategy is not initialized yet";

/// The thread local fallback try drop strategy. This doesn't store anything, it just provides an
/// interface to the thread local fallback try drop strategy, stored in a `static`.
///
/// # Note
/// This does **NOT** implement Send nor Sync because it not guaranteed that another thread will
/// have the same drop strategies as the thread that created this object; it could potentially be a
/// logic error. You can just create it on another thread as creating this is zero cost.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct ThreadLocalFallbackDropStrategy<OU: OnUninit = PanicOnUninit>(PhantomData<(OU, NotSendNotSync)>);

impl ThreadLocalFallbackDropStrategy<ErrorOnUninit> {
    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self(PhantomData)
    }
}

impl ThreadLocalFallbackDropStrategy<PanicOnUninit> {
    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "ds-panic")]
impl ThreadLocalFallbackDropStrategy<UseDefaultOnUninit> {
    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will set it to the default drop strategy.
    pub const fn on_uninit_use_default() -> Self {
        Self(PhantomData)
    }
}

impl FallbackTryDropStrategy for ThreadLocalFallbackDropStrategy<PanicOnUninit> {
    fn handle_error_in_strategy(&self, error: anyhow::Error) {
        read(|strategy| strategy.handle_error_in_strategy(error))
    }
}

#[cfg(feature = "ds-panic")]
impl FallbackTryDropStrategy for ThreadLocalFallbackDropStrategy<UseDefaultOnUninit> {
    fn handle_error_in_strategy(&self, error: anyhow::Error) {
        read_or_default(|strategy| strategy.handle_error_in_strategy(error))
    }
}

/// Install a new thread local fallback try drop strategy. Since this drop strategy will only be
/// used in one thread, it is more flexible than the global try drop strategy.
pub fn install(strategy: impl FallbackTryDropStrategy + 'static) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the thread local fallback try drop strategy. This will panic if the thread
/// local drop strategy has no value in it.
pub fn read<T>(f: impl FnOnce(&dyn FallbackTryDropStrategy) -> T) -> T {
    try_read(f).expect(UNINITIALIZED_ERROR)
}

#[cfg(feature = "ds-panic")]
fn default() -> Box<dyn FallbackTryDropStrategy> {
    Box::new(crate::drop_strategies::PanicDropStrategy::DEFAULT)
}

/// Get a reference to the thread local fallback try drop strategy. If there is no value present in
/// it, then it will initialize it with the default strategy.
#[cfg(feature = "ds-panic")]
pub fn read_or_default<T>(f: impl FnOnce(&dyn FallbackTryDropStrategy) -> T) -> T {
    DROP_STRATEGY.with(|drop_strategy| {
        let mut strategy = drop_strategy.borrow_mut();
        let strategy = strategy.get_or_insert_with(default);
        let strategy = &*strategy;
        f(strategy.as_ref())
    })
}

/// Get a reference to the thread local try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_read<T>(f: impl FnOnce(&dyn FallbackTryDropStrategy) -> T) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| cell.borrow().as_deref().map(f).ok_or(UninitializedError(())))
}

/// Get a mutable reference to the thread local fallback try drop strategy.
pub fn write<T>(f: impl FnOnce(&mut Box<dyn FallbackTryDropStrategy>) -> T) -> T {
    try_write(f).expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the thread local fallback try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_write<T>(f: impl FnOnce(&mut Box<dyn FallbackTryDropStrategy>) -> T) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| cell.borrow_mut().as_mut().map(f).ok_or(UninitializedError(())))
}

/// Get a mutable reference to the thread local fallback try drop strategy. If there is no value
/// present in it, then it will initialize it with the default strategy.
#[cfg(feature = "ds-panic")]
pub fn write_or_default<T>(f: impl FnOnce(&mut Box<dyn FallbackTryDropStrategy>) -> T) -> T {
    DROP_STRATEGY.with(|drop_strategy| f(drop_strategy.borrow_mut().get_or_insert_with(default)))
}

/// Install this fallback drop strategy to the current thread.
pub fn install_dyn(strategy: Box<dyn FallbackTryDropStrategy>) {
    DROP_STRATEGY.with(|drop_strategy| {
        drop_strategy.borrow_mut().replace(strategy);
    })
}

/// Uninstall this fallback drop strategy from the current thread.
pub fn uninstall() {
    DROP_STRATEGY.with(|drop_strategy| *drop_strategy.borrow_mut() = None)
}
