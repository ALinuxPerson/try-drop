//! Manage the global fallback handler.

use crate::handlers::on_uninit::{FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{GlobalTryDropStrategy, TryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock,
};
use std::boxed::Box;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use crate::handlers::common::Fallback;
use crate::handlers::common::global::{DefaultGlobalDefinition, GlobalDefinition, Global as GenericGlobal};

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

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

static FALLBACK_HANDLER: RwLock<Option<Box<dyn GlobalTryDropStrategy>>> = parking_lot::const_rwlock(None);

impl GlobalDefinition for Fallback {
    const UNINITIALIZED_ERROR: &'static str = "the global fallback handler is not initialized yet";
    type Global = Box<dyn GlobalTryDropStrategy>;

    fn global() -> &'static RwLock<Option<Self::Global>> {
        &FALLBACK_HANDLER
    }
}

#[cfg(feature = "ds-panic")]
impl DefaultGlobalDefinition for Fallback {
    fn default() -> Self::Global {
        Box::new(crate::drop_strategies::PanicDropStrategy::DEFAULT)
    }
}

impl<T: GlobalTryDropStrategy> From<T> for Box<dyn GlobalTryDropStrategy> {
    fn from(t: T) -> Self {
        Box::new(t)
    }
}

type Global = GenericGlobal<Fallback>;
type BoxDynGlobalTryDropStrategy = Box<dyn GlobalTryDropStrategy>;

global_methods! {
    Global = Global;
    GenericStrategy = GlobalTryDropStrategy;
    DynStrategy = BoxDynGlobalTryDropStrategy;
    feature = "ds-panic";

    install_dyn;
    install;
    try_read;
    read;
    try_write;
    write;
    uninstall;
    read_or_default;
    write_or_default;
}
