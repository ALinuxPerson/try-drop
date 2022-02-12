//! Manage the thread local fallback drop strategy.
mod scope_guard;

pub use scope_guard::ScopeGuard;
use std::boxed::Box;
use std::cell::RefCell;
use crate::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::uninit_error::UninitializedError;
use crate::{LOAD_ORDERING, STORE_ORDERING, TryDropStrategy};
use anyhow::Error;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread_local;

#[cfg(feature = "ds-panic")]
use crate::on_uninit::UseDefaultOnUninit;

thread_local! {
    static DROP_STRATEGY: RefCell<Option<Box<dyn TryDropStrategy>>> = RefCell::new(None);
}

/// The default thing to do when the fallback drop strategy is uninitialized.
#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the fallback drop strategy is uninitialized.
#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The default thread local fallback strategy.
pub static DEFAULT_THREAD_LOCAL_FALLBACK_STRATEGY: ThreadLocalFallbackDropStrategy = ThreadLocalFallbackDropStrategy::DEFAULT;

const UNINITIALIZED_ERROR: &str = "the thread local fallback drop strategy is not initialized yet";

/// The thread local fallback try drop strategy. This doesn't store anything, it just provides an
/// interface to the thread local fallback try drop strategy, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct ThreadLocalFallbackDropStrategy<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl ThreadLocalFallbackDropStrategy<DefaultOnUninit> {
    /// The default thread local fallback try drop strategy.
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _on_uninit: PhantomData,
    };
}

impl ThreadLocalFallbackDropStrategy<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    pub const ERROR_ON_UNINIT: Self = Self::on_uninit_error();

    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl ThreadLocalFallbackDropStrategy<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

#[cfg(feature = "ds-panic")]
impl ThreadLocalFallbackDropStrategy<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, this will set it to the default drop strategy.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl ThreadLocalFallbackDropStrategy<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Create a new interface to the thread local fallback drop strategy. If the thread local drop
    /// strategy is not initialized, a flag `last_drop_failed` will be set to true.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _on_uninit: PhantomData,
        }
    }

    /// Check if the last drop failed due to the thread local fallback drop strategy not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl TryDropStrategy for ThreadLocalFallbackDropStrategy<PanicOnUninit> {
    fn handle_error(&self, error: Error) {
        read(|strategy| strategy.handle_error(error))
    }
}

#[cfg(feature = "ds-panic")]
impl TryDropStrategy for ThreadLocalFallbackDropStrategy<UseDefaultOnUninit> {
    fn handle_error(&self, error: Error) {
        read_or_default(|strategy| strategy.handle_error(error))
    }
}

impl TryDropStrategy for ThreadLocalFallbackDropStrategy<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) =
        try_read(|strategy| strategy.handle_error(error))
        {
            self.set_last_drop_failed(true)
        } else {
            self.set_last_drop_failed(false)
        }
    }
}

/// Install a new thread local fallback try drop strategy. Since this drop strategy will only be
/// used in one thread, it is more flexible than the global try drop strategy.
pub fn install(strategy: impl TryDropStrategy + 'static) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the thread local fallback try drop strategy. This will panic if the thread
/// local drop strategy has no value in it.
pub fn read<T>(f: impl FnOnce(&dyn TryDropStrategy) -> T) -> T {
    try_read(f).expect(UNINITIALIZED_ERROR)
}

#[cfg(feature = "ds-panic")]
fn default() -> Box<dyn TryDropStrategy> {
    Box::new(crate::drop_strategies::PanicDropStrategy::DEFAULT)
}

/// Get a reference to the thread local fallback try drop strategy. If there is no value present in
/// it, then it will initialize it with the default strategy.
#[cfg(feature = "ds-panic")]
pub fn read_or_default<T>(f: impl FnOnce(&dyn TryDropStrategy) -> T) -> T {
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
    f: impl FnOnce(&dyn TryDropStrategy) -> T,
) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| {
        cell.borrow()
            .as_deref()
            .map(f)
            .ok_or(UninitializedError(()))
    })
}

/// Get a mutable reference to the thread local fallback try drop strategy.
pub fn write<T>(f: impl FnOnce(&mut Box<dyn TryDropStrategy>) -> T) -> T {
    try_write(f).expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the thread local fallback try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_write<T>(
    f: impl FnOnce(&mut Box<dyn TryDropStrategy>) -> T,
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
pub fn write_or_default<T>(f: impl FnOnce(&mut Box<dyn TryDropStrategy>) -> T) -> T {
    DROP_STRATEGY.with(|drop_strategy| f(drop_strategy.borrow_mut().get_or_insert_with(default)))
}

/// Install this fallback drop strategy to the current thread.
pub fn install_dyn(strategy: Box<dyn TryDropStrategy>) {
    DROP_STRATEGY.with(|drop_strategy| {
        drop_strategy.borrow_mut().replace(strategy);
    })
}


/// Uninstall this fallback drop strategy from the current thread.
pub fn uninstall() {
    take();
}

/// Take this fallback drop strategy from the current thread, if there is any.
pub fn take() -> Option<Box<dyn TryDropStrategy>> {
    DROP_STRATEGY.with(|drop_strategy| drop_strategy.borrow_mut().take())
}

/// Replace the current fallback drop strategy with another, returning the previous drop strategy if
/// any.
pub fn replace(new: impl TryDropStrategy + 'static) -> Option<Box<dyn TryDropStrategy>> {
    replace_dyn(Box::new(new))
}

/// Replace the current fallback drop strategy with another, returning the previous drop strategy if
/// any. Must be a dynamic trait object.
pub fn replace_dyn(new: Box<dyn TryDropStrategy>) -> Option<Box<dyn TryDropStrategy>> {
    DROP_STRATEGY.with(|previous| previous.borrow_mut().replace(new))
}

/// Install this strategy for the current scope.
///
/// # Panics
/// This panics if a strategy was already installed for the previous scope.
pub fn scope(strategy: impl TryDropStrategy + 'static) -> ScopeGuard {
    scope_dyn(Box::new(strategy))
}

/// Install this strategy for the current scope. Must be a dynamic trait object.
///
/// # Panics
/// This panics if a strategy was already installed for the previous scope.
pub fn scope_dyn(strategy: Box<dyn TryDropStrategy>) -> ScopeGuard {
    ScopeGuard::new_dyn(strategy)
}

