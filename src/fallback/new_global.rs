use std::boxed::Box;
use std::marker::PhantomData;
use anyhow::Error;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::drop_strategies::PanicDropStrategy;
use crate::TryDropStrategy;
use crate::fallback::GlobalFallbackTryDropStrategy;
use crate::on_uninit::{OnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::uninit_error::UninitializedError;

static FALLBACK_DROP_STRATEGY: RwLock<Option<Box<dyn GlobalFallbackTryDropStrategy>>> = parking_lot::const_rwlock(None);

const UNINITIALIZED_ERROR: &str = "the global drop strategy is not initialized yet";

/// The global fallback try drop strategy. This doesn't store anything, it just provides an
/// interface to the global fallback try drop strategy, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalFallbackDropStrategy<OU: OnUninit = PanicOnUninit>(PhantomData<OU>);

impl GlobalFallbackDropStrategy<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Get an interface to the global fallback try drop strategy. If there is no global try drop
    /// strategy initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "ds-panic")]
impl GlobalFallbackDropStrategy<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Get an interface to the global fallback try drop strategy. If there is no global fallback
    /// try drop strategy initialized, this will set it to the default.
    pub const fn on_uninit_use_default() -> Self {
        Self(PhantomData)
    }
}

impl TryDropStrategy for GlobalFallbackDropStrategy<PanicOnUninit> {
    fn handle_error(&self, error: Error) {
        read().handle_error_in_strategy(error)
    }
}

impl TryDropStrategy for GlobalFallbackDropStrategy<UseDefaultOnUninit> {
    fn handle_error(&self, error: Error) {
        read_or_default().handle_error_in_strategy(error)
    }
}

/// Install a new global fallback try drop strategy. Must be a dynamic trait object.
pub fn install_dyn(strategy: Box<dyn GlobalFallbackTryDropStrategy>) {
    FALLBACK_DROP_STRATEGY.write().replace(strategy);
}

/// Install a new global fallback try drop strategy.
pub fn install(strategy: impl GlobalFallbackTryDropStrategy) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the try drop strategy. If there is no global fallback try drop strategy
/// initialized, this will return an error.
pub fn try_read() -> Result<MappedRwLockReadGuard<'static, Box<dyn GlobalFallbackTryDropStrategy>>, UninitializedError> {
    let drop_strategy = FALLBACK_DROP_STRATEGY.read();

    if drop_strategy.is_some() {
        Ok(RwLockReadGuard::map(drop_strategy, |drop_strategy| drop_strategy.as_ref().unwrap()))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will panic.
pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalFallbackTryDropStrategy>> {
    try_read().expect(UNINITIALIZED_ERROR)
}

/// Get a reference to the try drop strategy. If there is no global fallback try drop strategy
/// initialized, this will set it to the default then return it.
#[cfg(feature = "ds-panic")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalFallbackTryDropStrategy>> {
    drop(write_or_default());
    read()
}

/// Get a mutable reference to the try drop strategy. If there is no global fallback try drop
/// strategy initialized, this will return an error.
pub fn try_write() -> Result<MappedRwLockWriteGuard<'static, Box<dyn GlobalFallbackTryDropStrategy>>, UninitializedError> {
    let drop_strategy = FALLBACK_DROP_STRATEGY.write();

    if drop_strategy.is_some() {
        Ok(RwLockWriteGuard::map(drop_strategy, |drop_strategy| drop_strategy.as_mut().unwrap()))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a mutable reference to the try drop strategy. If there is no global fallback try drop
/// strategy initialized, this will panic.
pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalFallbackTryDropStrategy>> {
    try_write().expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the try drop strategy. If there is no global fallback try drop
/// strategy initialized, this will set it to the default then return it.
#[cfg(feature = "ds-panic")]
pub fn write_or_default() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalFallbackTryDropStrategy>> {
    RwLockWriteGuard::map(
        FALLBACK_DROP_STRATEGY.write(),
        |drop_strategy| drop_strategy.get_or_insert_with(|| Box::new(PanicDropStrategy::DEFAULT))
    )
}

/// Uninstall or remove the global fallback try drop strategy.
pub fn uninstall() {
    *FALLBACK_DROP_STRATEGY.write() = None
}
