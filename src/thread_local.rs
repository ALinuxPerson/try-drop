//! Manage the thread local drop strategy.
use std::boxed::Box;
use std::cell::{Ref, RefCell, RefMut};
use std::error::Error;
use once_cell::unsync::{Lazy, OnceCell};
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy};
use crate::utils::NotSendNotSync;
use std::{fmt, thread_local};
use std::marker::PhantomData;
use crate::on_uninit::{ErrorOnUninit, OnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::uninit_error::UninitializedError;

thread_local! {
    static DROP_STRATEGY: RefCell<Option<Box<dyn DynFallibleTryDropStrategy>>> = RefCell::new(None);
}

const UNINITIALIZED_ERROR: &str = "the thread local drop strategy is not initialized yet";

#[cfg(not(feature = "ds-write"))]
pub type DefaultOnUninit = PanicOnUninit;

#[cfg(feature = "ds-write")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The thread local try drop strategy. This doesn't store anything, it just provides an interface
/// to the thread local try drop strategy, stored in a `static`.
///
/// # Note
/// This does **NOT** implement Send nor Sync because it not guaranteed that another thread will
/// have the same drop strategies as the thread that created this object; it could potentially be a
/// logic error. You can just create it on another thread as creating this is zero cost.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct ThreadLocalDropStrategy<OU: OnUninit = PanicOnUninit>(PhantomData<(OU, NotSendNotSync)>);

impl ThreadLocalDropStrategy<DefaultOnUninit> {
    pub const DEFAULT: Self = Self(PhantomData);
}

impl ThreadLocalDropStrategy<ErrorOnUninit> {
    /// Create a new interface to the thread local drop strategy. If the thread local drop strategy
    /// is not initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self(PhantomData)
    }
}

impl ThreadLocalDropStrategy<PanicOnUninit> {
    /// Create a new interface to the thread local drop strategy. If the thread local drop strategy
    /// is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "ds-write")]
impl ThreadLocalDropStrategy<UseDefaultOnUninit> {
    /// Create a new interface to the thread local drop strategy. If the thread local drop strategy
    /// is not initialized, this will set it to the default drop strategy.
    pub const fn on_uninit_use_default() -> Self {
        Self(PhantomData)
    }
}

impl FallibleTryDropStrategy for ThreadLocalDropStrategy<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for ThreadLocalDropStrategy<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error)).expect(UNINITIALIZED_ERROR)
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for ThreadLocalDropStrategy<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        read_or_default(|strategy| strategy.dyn_try_handle_error(error))
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
pub fn try_read<T>(f: impl FnOnce(&dyn DynFallibleTryDropStrategy) -> T) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| cell.borrow().as_deref().map(f).ok_or(UninitializedError(())))
}

/// Get a mutable reference to the thread local try drop strategy.
pub fn write<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> T {
    try_write(f).expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the thread local try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_write<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|cell| cell.borrow_mut().as_mut().map(f).ok_or(UninitializedError(())))
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
