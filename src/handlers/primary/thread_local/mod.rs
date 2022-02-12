//! Manage the thread local primary handler.
mod scope_guard;

pub use scope_guard::ScopeGuard;
use std::boxed::Box;
use std::cell::RefCell;

use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::thread_local;

#[cfg(feature = "ds-write")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

thread_local! {
    static PRIMARY_HANDLER: RefCell<Option<Box<dyn DynFallibleTryDropStrategy>>> = RefCell::new(None);
}

const UNINITIALIZED_ERROR: &str = "the thread local primary handler is not initialized yet";

/// The default thing to do when the primary thread local primary handler is uninitialized, that is
/// to panic.
#[cfg(not(feature = "ds-write"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the primary thread local primary handler is uninitialized, that is
/// to use the default strategy. Note that this mutates the thread local primary handler.
#[cfg(feature = "ds-write")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The default thread local primary handler.
pub static DEFAULT_THREAD_LOCAL_PRIMARY_HANDLER: ThreadLocalPrimaryHandler =
    ThreadLocalPrimaryHandler::DEFAULT;

/// The thread local primary handler. This doesn't store anything, it just provides an interface
/// to the thread local primary handler, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct ThreadLocalPrimaryHandler<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl ThreadLocalPrimaryHandler<DefaultOnUninit> {
    /// The default thread local primary handler.
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _on_uninit: PhantomData,
    };
}

impl Default for ThreadLocalPrimaryHandler<DefaultOnUninit> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ThreadLocalPrimaryHandler<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    pub const ERROR_ON_UNINIT: Self = Self::on_uninit_error();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl ThreadLocalPrimaryHandler<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

#[cfg(feature = "ds-write")]
impl ThreadLocalPrimaryHandler<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will set it to the default primary handler.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl ThreadLocalPrimaryHandler<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will set an internal flag stating that the drop failed.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _on_uninit: PhantomData,
        }
    }

    /// Check if the last drop failed due to the primary thread local primary handler not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error)).expect(UNINITIALIZED_ERROR)
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        read_or_default(|strategy| strategy.dyn_try_handle_error(error))
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        let (last_drop_failed, ret) = match try_read(|s| s.dyn_try_handle_error(error)) {
            Ok(Ok(())) => (false, Ok(())),
            Ok(Err(error)) => (false, Err(error)),
            Err(error) => (true, Err(error.into())),
        };
        self.set_last_drop_failed(last_drop_failed);
        ret
    }
}

/// Install a new thread local primary handler. Since this drop strategy will only be used in one
/// thread, it is more flexible than the global primary handler.
pub fn install(strategy: impl DynFallibleTryDropStrategy + 'static) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the thread local primary handler. This will panic if the thread local drop
/// strategy has no value in it.
pub fn read<T>(f: impl FnOnce(&dyn DynFallibleTryDropStrategy) -> T) -> T {
    try_read(f).expect(UNINITIALIZED_ERROR)
}

#[cfg(feature = "ds-write")]
fn default() -> Box<dyn DynFallibleTryDropStrategy> {
    let mut strategy = crate::drop_strategies::WriteDropStrategy::stderr();
    strategy.prelude("error: ");
    Box::new(strategy)
}

/// Get a reference to the thread local primary handler. If there is no value present in it, then
/// it will initialize it with the default drop strategy.
#[cfg(feature = "ds-write")]
pub fn read_or_default<T>(f: impl FnOnce(&dyn DynFallibleTryDropStrategy) -> T) -> T {
    PRIMARY_HANDLER.with(|drop_strategy| {
        let mut strategy = drop_strategy.borrow_mut();
        let strategy = strategy.get_or_insert_with(default);
        let strategy = &*strategy;
        f(strategy.as_ref())
    })
}

/// Get a reference to the thread local primary handler. This will return an error if the
/// thread local primary handler has no value in it.
pub fn try_read<T>(
    f: impl FnOnce(&dyn DynFallibleTryDropStrategy) -> T,
) -> Result<T, UninitializedError> {
    PRIMARY_HANDLER.with(|cell| {
        cell.borrow()
            .as_deref()
            .map(f)
            .ok_or(UninitializedError(()))
    })
}

/// Get a mutable reference to the thread local primary handler.
pub fn write<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> T {
    try_write(f).expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the thread local primary handler. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_write<T>(
    f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T,
) -> Result<T, UninitializedError> {
    PRIMARY_HANDLER.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .map(f)
            .ok_or(UninitializedError(()))
    })
}

/// Get a mutable reference to the thread local primary handler. If there is no value present in
/// it, then it will initialize it with the default strategy.
#[cfg(feature = "ds-write")]
pub fn write_or_default<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> T {
    PRIMARY_HANDLER.with(|drop_strategy| f(drop_strategy.borrow_mut().get_or_insert_with(default)))
}

/// Install this drop strategy to the current thread.
pub fn install_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) {
    PRIMARY_HANDLER.with(|drop_strategy| {
        drop_strategy.borrow_mut().replace(strategy);
    })
}

/// Uninstall this drop strategy from the current thread.
pub fn uninstall() {
    take();
}

/// Take this drop strategy from the current thread, if there is any.
pub fn take() -> Option<Box<dyn DynFallibleTryDropStrategy>> {
    PRIMARY_HANDLER.with(|drop_strategy| drop_strategy.borrow_mut().take())
}

/// Replace the current primary handler with another, returning the previous primary handler if
/// any.
pub fn replace(
    new: impl DynFallibleTryDropStrategy + 'static,
) -> Option<Box<dyn DynFallibleTryDropStrategy>> {
    replace_dyn(Box::new(new))
}

/// Replace the current primary handler with another, returning the previous primary handler if
/// any. Must be a dynamic trait object.
pub fn replace_dyn(
    new: Box<dyn DynFallibleTryDropStrategy>,
) -> Option<Box<dyn DynFallibleTryDropStrategy>> {
    PRIMARY_HANDLER.with(|previous| previous.borrow_mut().replace(new))
}

/// Install this strategy for the current scope.
///
/// # Panics
/// This panics if a strategy was already installed for the previous scope.
pub fn scope(strategy: impl DynFallibleTryDropStrategy + 'static) -> ScopeGuard {
    scope_dyn(Box::new(strategy))
}

/// Install this strategy for the current scope. Must be a dynamic trait object.
///
/// # Panics
/// This panics if a strategy was already installed for the previous scope.
pub fn scope_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) -> ScopeGuard {
    ScopeGuard::new_dyn(strategy)
}
