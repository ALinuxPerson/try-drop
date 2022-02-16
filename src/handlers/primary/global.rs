//! Manage the primary global handler.

use crate::handlers::common::global::{
    DefaultGlobalDefinition, Global as GenericGlobal, GlobalDefinition,
};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::{Global as GlobalScope, Primary};
use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::primary::{Abstracter, DefaultOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{
    FallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy, LOAD_ORDERING, STORE_ORDERING,
};
use anyhow::Error;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::boxed::Box;
use std::convert;

#[cfg(feature = "ds-write")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

/// The primary global handler which uses the global scope.
pub type GlobalPrimaryHandler<OU = DefaultOnUninit> = CommonHandler<OU, GlobalScope, Primary>;

/// The default global primary handler.
pub static DEFAULT_GLOBAL_PRIMARY_HANDLER: GlobalPrimaryHandler = GlobalPrimaryHandler::DEFAULT;

impl_fallible_try_drop_strategy_for!(GlobalPrimaryHandler
where
    Scope: GlobalScope,
    Definition: GlobalDefinition
);

static PRIMARY_HANDLER: RwLock<Option<Box<dyn GlobalDynFallibleTryDropStrategy>>> =
    parking_lot::const_rwlock(None);

impl GlobalDefinition for Primary {
    const UNINITIALIZED_ERROR: &'static str = "the global primary handler is not initialized yet";
    type Global = Box<dyn GlobalDynFallibleTryDropStrategy>;

    fn global() -> &'static RwLock<Option<Self::Global>> {
        &PRIMARY_HANDLER
    }
}

#[cfg(feature = "ds-write")]
impl DefaultGlobalDefinition for Primary {
    fn default() -> Self::Global {
        let mut strategy = crate::drop_strategies::WriteDropStrategy::stderr();
        strategy.prelude("error: ");
        Box::new(strategy)
    }
}

impl<T: GlobalDynFallibleTryDropStrategy + 'static> From<T>
    for Box<dyn GlobalDynFallibleTryDropStrategy>
{
    fn from(handler: T) -> Self {
        Box::new(handler)
    }
}

type Global = GenericGlobal<Primary>;

/// A handy type alias to `Box<dyn GlobalDynFallibleTryDropStrategy>`.
pub type BoxDynGlobalFallibleTryDropStrategy = Box<dyn GlobalDynFallibleTryDropStrategy>;

global_methods! {
    Global = Global;
    GenericStrategy = GlobalDynFallibleTryDropStrategy;
    DynStrategy = BoxDynGlobalFallibleTryDropStrategy;
    feature = "ds-write";

    /// Set the global primary handler. Must be a dynamic trait object.
    install_dyn;

    /// Get the global primary handler.
    install;

    /// Try and get a reference to the global primary handler.
    ///
    /// # Errors
    /// If the global primary handler is not initialized yet, an error is returned.
    try_read;

    /// Get a reference to the global primary handler.
    ///
    /// # Panics
    /// If the global primary handler is not initialized yet, a panic is raised.
    read;

    /// Try and get a mutable reference to the global primary handler.
    ///
    /// # Errors
    /// If the global primary handler is not initialized yet, an error is returned.
    try_write;

    /// Get a mutable reference to the global primary handler.
    ///
    /// # Panics
    /// If the global primary handler is not initialized yet, a panic is raised.
    write;

    /// Uninstall the global primary handler.
    uninstall;

    /// Get a reference to the global primary handler.
    ///
    /// If the global primary handler is not initialized yet, it is initialized with the default
    /// value.
    read_or_default;

    /// Get a mutable reference to the global primary handler.
    ///
    /// If the global primary handler is not initialized yet, it is initialized with the default
    /// value.
    write_or_default;
}
