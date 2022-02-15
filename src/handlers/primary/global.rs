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

pub type GlobalPrimaryHandler<OU = DefaultOnUninit> = CommonHandler<OU, GlobalScope, Primary>;
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
pub type BoxDynGlobalFallibleTryDropStrategy = Box<dyn GlobalDynFallibleTryDropStrategy>;

global_methods! {
    Global = Global;
    GenericStrategy = GlobalDynFallibleTryDropStrategy;
    DynStrategy = BoxDynGlobalFallibleTryDropStrategy;
    feature = "ds-write";

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
