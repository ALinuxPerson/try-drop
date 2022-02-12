//! Manage the thread local drop strategy.
use std::boxed::Box;
use std::cell::RefCell;

use crate::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::uninit_error::UninitializedError;
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread_local;

#[cfg(feature = "ds-write")]
use crate::on_uninit::UseDefaultOnUninit;

thread_local! {
    static DROP_STRATEGY: RefCell<Option<Box<dyn DynFallibleTryDropStrategy>>> = RefCell::new(None);
}

const UNINITIALIZED_ERROR: &str = "the thread local drop strategy is not initialized yet";

#[cfg(not(feature = "ds-write"))]
pub type DefaultOnUninit = PanicOnUninit;

#[cfg(feature = "ds-write")]
pub type DefaultOnUninit = UseDefaultOnUninit;

pub static DEFAULT_THREAD_LOCAL_PRIMARY_DROP_STRATEGY: ThreadLocalPrimaryTryDropStrategy = ThreadLocalPrimaryTryDropStrategy::DEFAULT;

/// The thread local try drop strategy. This doesn't store anything, it just provides an interface
/// to the thread local try drop strategy, stored in a `static`.
///
/// # Note
/// This does **NOT** implement Send nor Sync because it not guaranteed that another thread will
/// have the same drop strategies as the thread that created this object; it could potentially be a
/// logic error. You can just create it on another thread as creating this is zero cost.
#[cfg_attr(
feature = "derives",
derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct ThreadLocalPrimaryTryDropStrategy<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl ThreadLocalPrimaryTryDropStrategy<DefaultOnUninit> {
    pub const DEFAULT: Self = Self { extra_data: (), _on_uninit: PhantomData };
}

impl Default for ThreadLocalPrimaryTryDropStrategy<DefaultOnUninit> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ThreadLocalPrimaryTryDropStrategy<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    pub const ERROR_ON_UNINIT: Self = Self::on_uninit_error();

    /// Create a new interface to the thread local drop strategy. If the thread local drop strategy
    /// is not initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self { extra_data: (), _on_uninit: PhantomData }
    }
}

impl ThreadLocalPrimaryTryDropStrategy<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Create a new interface to the thread local drop strategy. If the thread local drop strategy
    /// is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self { extra_data: (), _on_uninit: PhantomData }
    }
}

#[cfg(feature = "ds-write")]
impl ThreadLocalPrimaryTryDropStrategy<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Create a new interface to the thread local drop strategy. If the thread local drop strategy
    /// is not initialized, this will set it to the default drop strategy.
    pub const fn on_uninit_use_default() -> Self {
        Self { extra_data: (), _on_uninit: PhantomData }
    }
}

impl ThreadLocalPrimaryTryDropStrategy<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Create a new interface to the thread local drop strategy. If the thread local drop strategy
    /// is not initialized, this will set an internal flag stating that the drop failed.
    pub const fn on_uninit_flag() -> Self {
        Self { extra_data: AtomicBool::new(false), _on_uninit: PhantomData }
    }

    /// Check if the last drop failed due to the primary thread local drop strategy not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(Ordering::Acquire)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, Ordering::Release)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryTryDropStrategy<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryTryDropStrategy<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error)).expect(UNINITIALIZED_ERROR)
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for ThreadLocalPrimaryTryDropStrategy<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        read_or_default(|strategy| strategy.dyn_try_handle_error(error))
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryTryDropStrategy<FlagOnUninit> {
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

/// Install a new thread local try drop strategy. Since this drop strategy will only be used in one
/// thread, it is more flexible than the global try drop strategy.
pub fn install(strategy: impl DynFallibleTryDropStrategy + 'static) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the thread local try drop strategy. This will panic if the thread local drop
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

/// Get a reference to the thread local try drop strategy. If there is no value present in it, then
/// it will initialize it with the default strategy.
#[cfg(feature = "ds-write")]
pub fn read_or_default<T>(f: impl FnOnce(&dyn DynFallibleTryDropStrategy) -> T) -> T {
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
    f: impl FnOnce(&dyn DynFallibleTryDropStrategy) -> T,
) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| {
        cell.borrow()
            .as_deref()
            .map(f)
            .ok_or(UninitializedError(()))
    })
}

/// Get a mutable reference to the thread local try drop strategy.
pub fn write<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> T {
    try_write(f).expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the thread local try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_write<T>(
    f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T,
) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .map(f)
            .ok_or(UninitializedError(()))
    })
}

/// Get a mutable reference to the thread local try drop strategy. If there is no value present in
/// it, then it will initialize it with the default strategy.
#[cfg(feature = "ds-write")]
pub fn write_or_default<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> T {
    DROP_STRATEGY.with(|drop_strategy| f(drop_strategy.borrow_mut().get_or_insert_with(default)))
}

/// Install this drop strategy to the current thread.
pub fn install_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) {
    DROP_STRATEGY.with(|drop_strategy| {
        drop_strategy.borrow_mut().replace(strategy);
    })
}

/// Uninstall this drop strategy from the current thread.
pub fn uninstall() {
    DROP_STRATEGY.with(|drop_strategy| *drop_strategy.borrow_mut() = None)
}
