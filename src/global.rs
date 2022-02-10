use std::boxed::Box;
use std::marker::PhantomData;
use anyhow::Error;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::drop_strategies::PanicDropStrategy;
use crate::{FallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy};
use crate::on_uninit::{ErrorOnUninit, OnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::uninit_error::UninitializedError;

static DROP_STRATEGY: RwLock<Option<Box<dyn GlobalDynFallibleTryDropStrategy>>> = parking_lot::const_rwlock(None);

const UNINITIALIZED_ERROR: &str = "the global drop strategy is not initialized yet";

/// The global try drop strategy. This doesn't store anything, it just provides an interface
/// to the global try drop strategy, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalFallibleTryDropStrategy<OU: OnUninit = PanicOnUninit>(PhantomData<OU>);

impl GlobalFallibleTryDropStrategy<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    pub const ERROR_ON_UNINIT: Self = Self::on_uninit_error();

    /// Get an interface to the global try drop strategy. If there is no global try drop strategy
    /// initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self(PhantomData)
    }
}

impl GlobalFallibleTryDropStrategy<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Get an interface to the global try drop strategy. If there is no global try drop strategy
    /// initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "ds-write")]
impl GlobalFallibleTryDropStrategy<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Get an interface to the global try drop strategy. If there is no global try drop strategy
    /// initialized, this will set it to the default.
    pub const fn on_uninit_use_default() -> Self {
        Self(PhantomData)
    }
}

impl FallibleTryDropStrategy for GlobalFallibleTryDropStrategy<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        try_read().map(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for GlobalFallibleTryDropStrategy<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read().dyn_try_handle_error(error)
    }
}

impl FallibleTryDropStrategy for GlobalFallibleTryDropStrategy<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read_or_default().dyn_try_handle_error(error)
    }
}

/// Install a new global try drop strategy. Must be a dynamic trait object.
pub fn install_dyn(strategy: Box<dyn GlobalDynFallibleTryDropStrategy>) {
    DROP_STRATEGY.write().replace(strategy);
}

/// Install a new global try drop strategy.
pub fn install(strategy: impl GlobalDynFallibleTryDropStrategy) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will return an error.
pub fn try_read() -> Result<MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>, UninitializedError> {
    let drop_strategy = DROP_STRATEGY.read();

    if drop_strategy.is_some() {
        Ok(RwLockReadGuard::map(drop_strategy, |drop_strategy| drop_strategy.as_ref().unwrap()))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will panic.
pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    try_read().expect(UNINITIALIZED_ERROR)
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will set it to the default then return it.
#[cfg(feature = "ds-write")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    drop(write_or_default());
    read()
}

/// Get a mutable reference to the try drop strategy. If there is no global try drop strategy
/// initialized, this will return an error.
pub fn try_write() -> Result<MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>, UninitializedError> {
    let drop_strategy = DROP_STRATEGY.write();

    if drop_strategy.is_some() {
        Ok(RwLockWriteGuard::map(drop_strategy, |drop_strategy| drop_strategy.as_mut().unwrap()))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a mutable reference to the try drop strategy. If there is no global try drop strategy
/// initialized, this will panic.
pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    try_write().expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the try drop strategy. If there is no global try drop strategy
/// initialized, this will set it to the default then return it.
#[cfg(feature = "ds-write")]
pub fn write_or_default() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    RwLockWriteGuard::map(
        DROP_STRATEGY.write(),
        |drop_strategy| drop_strategy.get_or_insert_with(|| Box::new(PanicDropStrategy::DEFAULT))
    )
}

/// Uninstall or remove the global try drop strategy.
pub fn uninstall() {
    *DROP_STRATEGY.write() = None
}
