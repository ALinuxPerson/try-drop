//! Manage the global fallback handler.

use crate::handlers::on_uninit::{FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{GlobalTryDropStrategy, TryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::boxed::Box;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

static FALLBACK_HANDLER: RwLock<Option<Box<dyn GlobalTryDropStrategy>>> =
    parking_lot::const_rwlock(None);

const UNINITIALIZED_ERROR: &str = "the global fallback handler is not initialized yet";

/// The default thing to do when the global fallback handler is not initialized.
#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the global fallback handler is not initialized.
#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The default global fallback handler.
pub static DEFAULT_GLOBAL_FALLBACK_HANDLER: GlobalFallbackHandler =
    GlobalFallbackHandler::DEFAULT;

/// The global fallback try fallback handler. This doesn't store anything, it just provides an
/// interface to the global fallback handler, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalFallbackHandler<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl GlobalFallbackHandler<DefaultOnUninit> {
    /// The default global fallback handler.
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _on_uninit: PhantomData,
    };
}

impl GlobalFallbackHandler<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Get an interface to the global fallback handler. If there is no global fallback handler
    /// initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

#[cfg(feature = "ds-panic")]
impl GlobalFallbackHandler<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Get an interface to the global fallback handler. If there is no global fallback
    /// fallback handler initialized, this will set it to the default.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl GlobalFallbackHandler<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Get an interface to the global fallback handler. If there is no global fallback
    /// fallback handler initialized, this will set the `last_drop_failed` flag to `true`.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _on_uninit: PhantomData,
        }
    }

    /// Did the last attempt to handle a drop failure fail because the global fallback handler
    /// wasn't initialized?
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl TryDropStrategy for GlobalFallbackHandler<PanicOnUninit> {
    fn handle_error(&self, error: Error) {
        read().handle_error(error)
    }
}

#[cfg(feature = "ds-panic")]
impl TryDropStrategy for GlobalFallbackHandler<UseDefaultOnUninit> {
    fn handle_error(&self, error: Error) {
        read_or_default().handle_error(error)
    }
}

impl TryDropStrategy for GlobalFallbackHandler<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) = try_read().map(|s| s.handle_error(error)) {
            self.set_last_drop_failed(true);
        } else {
            self.set_last_drop_failed(false);
        }
    }
}

/// Install a new global fallback handler. Must be a dynamic trait object.
pub fn install_dyn(strategy: Box<dyn GlobalTryDropStrategy>) {
    FALLBACK_HANDLER.write().replace(strategy);
}

/// Install a new global fallback handler.
pub fn install(strategy: impl GlobalTryDropStrategy) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the fallback handler. If there is no global fallback handler
/// initialized, this will return an error.
pub fn try_read(
) -> Result<MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>>, UninitializedError> {
    let fallback_handler = FALLBACK_HANDLER.read();

    if fallback_handler.is_some() {
        Ok(RwLockReadGuard::map(fallback_handler, |drop_strategy| {
            drop_strategy.as_ref().unwrap()
        }))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a reference to the fallback handler. If there is no global fallback handler initialized,
/// this will panic.
pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    try_read().expect(UNINITIALIZED_ERROR)
}

/// Get a reference to the fallback handler. If there is no global fallback handler
/// initialized, this will set it to the default then return it.
#[cfg(feature = "ds-panic")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    drop(write_or_default());
    read()
}

/// Get a mutable reference to the fallback handler. If there is no global fallback try drop
/// strategy initialized, this will return an error.
pub fn try_write(
) -> Result<MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>>, UninitializedError> {
    let fallback_handler = FALLBACK_HANDLER.write();

    if fallback_handler.is_some() {
        Ok(RwLockWriteGuard::map(fallback_handler, |drop_strategy| {
            drop_strategy.as_mut().unwrap()
        }))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a mutable reference to the fallback handler. If there is no global fallback handler
/// initialized, this will panic.
pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    try_write().expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the fallback handler. If there is no global fallback handler
/// initialized, this will set it to the default then return it.
#[cfg(feature = "ds-panic")]
pub fn write_or_default() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    use crate::drop_strategies::PanicDropStrategy;

    RwLockWriteGuard::map(FALLBACK_HANDLER.write(), |drop_strategy| {
        drop_strategy.get_or_insert_with(|| Box::new(PanicDropStrategy::DEFAULT))
    })
}

/// Uninstall or remove the global fallback handler.
pub fn uninstall() {
    *FALLBACK_HANDLER.write() = None
}
