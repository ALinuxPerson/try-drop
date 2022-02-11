//! Manage the thread local fallback drop strategy.
use std::boxed::Box;
use std::cell::RefCell;

use crate::fallback::{FlagOnUninit, OnUninitFallback};
use crate::on_uninit::{ErrorOnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::uninit_error::UninitializedError;
use crate::utils::NotSendNotSync;
use crate::{FallbackTryDropStrategy, TryDropStrategy};
use anyhow::Error;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread_local;

thread_local! {
    static DROP_STRATEGY: RefCell<Option<Box<dyn FallbackTryDropStrategy>>> = RefCell::new(None);
}

#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

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
pub struct ThreadLocalFallbackDropStrategy<OU: OnUninitFallback = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _marker: PhantomData<(OU, NotSendNotSync)>,
}

impl ThreadLocalFallbackDropStrategy<DefaultOnUninit> {
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _marker: PhantomData,
    };
}

impl ThreadLocalFallbackDropStrategy<ErrorOnUninit> {
    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self {
            extra_data: (),
            _marker: PhantomData,
        }
    }
}

impl ThreadLocalFallbackDropStrategy<PanicOnUninit> {
    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "ds-panic")]
impl ThreadLocalFallbackDropStrategy<UseDefaultOnUninit> {
    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will set it to the default drop strategy.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _marker: PhantomData,
        }
    }
}

impl ThreadLocalFallbackDropStrategy<FlagOnUninit> {
    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, a flag `last_drop_failed` will be set to true.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _marker: PhantomData,
        }
    }

    /// Check if the last drop failed due to the thread local fallback drop strategy not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(Ordering::Acquire)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, Ordering::Release)
    }
}

impl TryDropStrategy for ThreadLocalFallbackDropStrategy<PanicOnUninit> {
    fn handle_error(&self, error: Error) {
        read(|strategy| strategy.handle_error_in_strategy(error))
    }
}

#[cfg(feature = "ds-panic")]
impl TryDropStrategy for ThreadLocalFallbackDropStrategy<UseDefaultOnUninit> {
    fn handle_error(&self, error: Error) {
        read_or_default(|strategy| strategy.handle_error_in_strategy(error))
    }
}

impl TryDropStrategy for ThreadLocalFallbackDropStrategy<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) =
            try_read(|strategy| strategy.handle_error_in_strategy(error))
        {
            self.set_last_drop_failed(true)
        }
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
pub fn try_read<T>(
    f: impl FnOnce(&dyn FallbackTryDropStrategy) -> T,
) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| {
        cell.borrow()
            .as_deref()
            .map(f)
            .ok_or(UninitializedError(()))
    })
}

/// Get a mutable reference to the thread local fallback try drop strategy.
pub fn write<T>(f: impl FnOnce(&mut Box<dyn FallbackTryDropStrategy>) -> T) -> T {
    try_write(f).expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the thread local fallback try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_write<T>(
    f: impl FnOnce(&mut Box<dyn FallbackTryDropStrategy>) -> T,
) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .map(f)
            .ok_or(UninitializedError(()))
    })
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
