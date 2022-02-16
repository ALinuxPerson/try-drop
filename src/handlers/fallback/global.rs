//! Manage the global fallback handler.

use super::DefaultOnUninit;
use crate::handlers::common::global::{Global as GenericGlobal, GlobalDefinition};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::Fallback;
use crate::handlers::common::Global as GlobalScope;
use crate::handlers::fallback::Abstracter;
use crate::handlers::on_uninit::{FlagOnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{GlobalTryDropStrategy, TryDropStrategy};
use anyhow::Error;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock};
use std::boxed::Box;

#[cfg(feature = "ds-panic")]
use crate::handlers::common::global::DefaultGlobalDefinition;

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

/// A fallback handler which uses the global scope.
pub type GlobalFallbackHandler<OU = DefaultOnUninit> = CommonHandler<OU, GlobalScope, Fallback>;

/// The default global fallback handler.
pub static DEFAULT_GLOBAL_FALLBACK_HANDLER: GlobalFallbackHandler = GlobalFallbackHandler::DEFAULT;

static FALLBACK_HANDLER: RwLock<Option<Box<dyn GlobalTryDropStrategy>>> =
    parking_lot::const_rwlock(None);

impl_try_drop_strategy_for!(GlobalFallbackHandler where Scope: GlobalScope);

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

    /// Install a new global fallback handler. Must be a dynamic trait object.
    install_dyn;

    /// Install a new global fallback handler.
    install;

    /// Try and get a reference to the global fallback handler.
    ///
    /// # Errors
    /// If the global fallback handler is not initialized yet, an error is returned.
    try_read;

    /// Get a reference to the global fallback handler.
    ///
    /// # Panics
    /// If the global fallback handler is not initialized yet, a panic is raised.
    read;

    /// Try and get a mutable reference to the global fallback handler.
    ///
    /// # Errors
    /// If the global fallback handler is not initialized yet, an error is returned.
    try_write;

    /// Get a mutable reference to the global fallback handler.
    ///
    /// # Panics
    /// If the global fallback handler is not initialized yet, a panic is raised.
    write;

    /// Uninstall the current global fallback handler.
    uninstall;

    /// Get a reference to the global fallback handler.
    ///
    /// If the global fallback handler is not initialized yet, it is initialized with the default
    /// one.
    read_or_default;

    /// Get a mutable reference to the global fallback handler.
    ///
    /// If the global fallback handler is not initialized yet, it is initialized with the default
    /// one.
    write_or_default;
}
