//! Manage the thread local drop strategy.
use std::boxed::Box;
use std::cell::{Ref, RefCell, RefMut};
use std::error::Error;
use once_cell::unsync::{Lazy, OnceCell};
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy};
use crate::utils::NotSendNotSync;
use std::{fmt, thread_local};
use std::marker::PhantomData;
use crate::drop_strategies::WriteDropStrategy;
use crate::on_uninit::{ErrorOnUninit, OnUninit, PanicOnUninit, UseDefaultOnUninit};

thread_local! {
    static DROP_STRATEGY: OnceCell<RefCell<Box<dyn DynFallibleTryDropStrategy>>> = OnceCell::new();
}

fn try_drop_strategy<T>(f: impl FnOnce(&RefCell<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> Result<T, UninitializedError> {
    DROP_STRATEGY.with(|drop_strategy| {
        drop_strategy.get().map(f).ok_or(UninitializedError(()))
    })
}

const UNINITIALIZED_ERROR: &str = "the thread local drop strategy is not initialized yet";

fn drop_strategy<T>(f: impl FnOnce(&RefCell<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    try_drop_strategy(f).expect(UNINITIALIZED_ERROR)
}

#[cfg(feature = "ds-write")]
fn drop_strategy_or_default<T>(f: impl FnOnce(&RefCell<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    DROP_STRATEGY.with(|drop_strategy| {
        f(drop_strategy.get_or_init(|| {
            let mut strategy = WriteDropStrategy::stderr();
            strategy.prelude("error: ");
            RefCell::new(Box::new(strategy))
        }))
    })
}

/// This error occurs when an attempt to get the thread local drop strategy is made before it is
/// initialized.
#[cfg_attr(
    feature = "derives",
    derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[derive(Debug)]
pub struct UninitializedError(());

impl Error for UninitializedError {}
impl fmt::Display for UninitializedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(UNINITIALIZED_ERROR)
    }
}

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
pub fn read<T>(f: impl FnOnce(Ref<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy(|strategy| f(strategy.borrow()))
}

/// Get a reference to the thread local try drop strategy. If there is no value present in it, then
/// it will initialize it with the default strategy.
#[cfg(feature = "ds-write")]
pub fn read_or_default<T>(f: impl FnOnce(Ref<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy_or_default(|strategy| f(strategy.borrow()))
}

/// Get a reference to the thread local try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_read<T>(f: impl FnOnce(Ref<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> Result<T, UninitializedError> {
    try_drop_strategy(|strategy| f(strategy.borrow()))
}

/// Get a mutable reference to the thread local try drop strategy.
pub fn write<T>(f: impl FnOnce(RefMut<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy(|strategy| f(strategy.borrow_mut()))
}

/// Get a mutable reference to the thread local try drop strategy. This will return an error if the
/// thread local drop strategy has no value in it.
pub fn try_write<T>(f: impl FnOnce(RefMut<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> Result<T, UninitializedError> {
    try_drop_strategy(|strategy| f(strategy.borrow_mut()))
}

/// Get a mutable reference to the thread local try drop strategy. If there is no value present in
/// it, then it will initialize it with the default strategy.
#[cfg(feature = "ds-write")]
pub fn write_or_default<T>(f: impl FnOnce(Ref<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy_or_default(|strategy| f(strategy.borrow()))
}

/// Install this drop strategy to the current thread.
pub fn install_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) {
    DROP_STRATEGY.with(|drop_strategy| {
        match drop_strategy.get() {
            Some(thread_local_strategy) => *thread_local_strategy.borrow_mut() = strategy,
            None => {
                let _ = drop_strategy.set(RefCell::new(strategy));
            }
        }
    })
}
