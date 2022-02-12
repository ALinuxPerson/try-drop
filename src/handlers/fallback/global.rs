//! Manage the global fallback handler.

use crate::handlers::on_uninit::{FlagOnUninit, OnUninit, PanicOnUninit};
use crate::uninit_error::UninitializedError;
use crate::{GlobalTryDropStrategy, LOAD_ORDERING, STORE_ORDERING, TryDropStrategy};
use anyhow::Error;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::boxed::Box;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

static FALLBACK_DROP_STRATEGY: RwLock<Option<Box<dyn GlobalTryDropStrategy>>> =
    parking_lot::const_rwlock(None);

const UNINITIALIZED_ERROR: &str = "the global drop strategy is not initialized yet";

/// The default thing to do when the global drop strategy is not initialized.
#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the global drop strategy is not initialized.
#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The default global fallback drop strategy.
pub static DEFAULT_GLOBAL_FALLBACK_STRATEGY: GlobalFallbackDropStrategy = GlobalFallbackDropStrategy::DEFAULT;

/// The global fallback try drop strategy. This doesn't store anything, it just provides an
/// interface to the global fallback try drop strategy, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalFallbackDropStrategy<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl GlobalFallbackDropStrategy<DefaultOnUninit> {
    /// The default global fallback drop strategy.
    pub const DEFAULT: Self = Self { extra_data: (), _on_uninit: PhantomData };
}

impl GlobalFallbackDropStrategy<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Get an interface to the global fallback try drop strategy. If there is no global try drop
    /// strategy initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

#[cfg(feature = "ds-panic")]
impl GlobalFallbackDropStrategy<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Get an interface to the global fallback try drop strategy. If there is no global fallback
    /// try drop strategy initialized, this will set it to the default.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl GlobalFallbackDropStrategy<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Get an interface to the global fallback try drop strategy. If there is no global fallback
    /// try drop strategy initialized, this will set the `last_drop_failed` flag to `true`.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _on_uninit: PhantomData,
        }
    }

    /// Did the last attempt to handle a drop failure fail because the global fallback try drop
    /// strategy wasn't initialized?
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl TryDropStrategy for GlobalFallbackDropStrategy<PanicOnUninit> {
    fn handle_error(&self, error: Error) {
        read().handle_error(error)
    }
}

#[cfg(feature = "ds-panic")]
impl TryDropStrategy for GlobalFallbackDropStrategy<UseDefaultOnUninit> {
    fn handle_error(&self, error: Error) {
        read_or_default().handle_error(error)
    }
}

impl TryDropStrategy for GlobalFallbackDropStrategy<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) = try_read().map(|s| s.handle_error(error)) {
            self.set_last_drop_failed(true);
        } else {
            self.set_last_drop_failed(false);
        }
    }
}

/// Install a new global fallback try drop strategy. Must be a dynamic trait object.
pub fn install_dyn(strategy: Box<dyn GlobalTryDropStrategy>) {
    FALLBACK_DROP_STRATEGY.write().replace(strategy);
}

/// Install a new global fallback try drop strategy.
pub fn install(strategy: impl GlobalTryDropStrategy) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the try drop strategy. If there is no global fallback try drop strategy
/// initialized, this will return an error.
pub fn try_read() -> Result<
    MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>>,
    UninitializedError,
> {
    let drop_strategy = FALLBACK_DROP_STRATEGY.read();

    if drop_strategy.is_some() {
        Ok(RwLockReadGuard::map(drop_strategy, |drop_strategy| {
            drop_strategy.as_ref().unwrap()
        }))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will panic.
pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    try_read().expect(UNINITIALIZED_ERROR)
}

/// Get a reference to the try drop strategy. If there is no global fallback try drop strategy
/// initialized, this will set it to the default then return it.
#[cfg(feature = "ds-panic")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    drop(write_or_default());
    read()
}

/// Get a mutable reference to the try drop strategy. If there is no global fallback try drop
/// strategy initialized, this will return an error.
pub fn try_write() -> Result<
    MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>>,
    UninitializedError,
> {
    let drop_strategy = FALLBACK_DROP_STRATEGY.write();

    if drop_strategy.is_some() {
        Ok(RwLockWriteGuard::map(drop_strategy, |drop_strategy| {
            drop_strategy.as_mut().unwrap()
        }))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a mutable reference to the try drop strategy. If there is no global fallback try drop
/// strategy initialized, this will panic.
pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    try_write().expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the try drop strategy. If there is no global fallback try drop
/// strategy initialized, this will set it to the default then return it.
#[cfg(feature = "ds-panic")]
pub fn write_or_default() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>>
{
    use crate::drop_strategies::PanicDropStrategy;

    RwLockWriteGuard::map(FALLBACK_DROP_STRATEGY.write(), |drop_strategy| {
        drop_strategy.get_or_insert_with(|| Box::new(PanicDropStrategy::DEFAULT))
    })
}

/// Uninstall or remove the global fallback try drop strategy.
pub fn uninstall() {
    *FALLBACK_DROP_STRATEGY.write() = None
}
