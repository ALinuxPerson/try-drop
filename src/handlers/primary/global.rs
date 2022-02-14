//! Manage the primary global handler.

use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{
    FallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy, LOAD_ORDERING, STORE_ORDERING,
};
use anyhow::Error;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::boxed::Box;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use crate::handlers::common::Primary;
use crate::handlers::common::global::{DefaultGlobalDefinition, Global as GenericGlobal, GlobalDefinition};

#[cfg(feature = "ds-write")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

/// The default global primary handler.
pub static DEFAULT_GLOBAL_PRIMARY_DROP_STRATEGY: GlobalPrimaryHandler =
    GlobalPrimaryHandler::DEFAULT;

/// The default thing to do when the global primary primary handler is uninitialized, that is to
/// panic.
#[cfg(not(feature = "ds-write"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the global primary handler is uninitialized, that is to use the
/// default. Note that this mutates the global primary handler.
#[cfg(feature = "ds-write")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The global primary handler. This doesn't store anything, it just provides an interface
/// to the global primary handler, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalPrimaryHandler<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl GlobalPrimaryHandler<DefaultOnUninit> {
    /// The default global primary handler.
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _on_uninit: PhantomData,
    };
}

impl GlobalPrimaryHandler<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    pub const ERROR_ON_UNINIT: Self = Self::on_uninit_error();

    /// Get an interface to the global primary handler. If there is no global primary handler
    /// initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl GlobalPrimaryHandler<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Get an interface to the global primary handler. If there is no global primary handler
    /// initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

#[cfg(feature = "ds-write")]
impl GlobalPrimaryHandler<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Get an interface to the global primary handler. If there is no global primary handler
    /// initialized, this will set it to the default.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl GlobalPrimaryHandler<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Get an interface to the global primary handler. If there is no global primary handler
    /// initialized, this will set an internal flag stating that the drop failed.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _on_uninit: PhantomData,
        }
    }

    /// Check if acquiring a reference to the global primary handler failed due to it not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryHandler<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        try_read()
            .map(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryHandler<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read().dyn_try_handle_error(error)
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for GlobalPrimaryHandler<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read_or_default().dyn_try_handle_error(error)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        let (last_drop_failed, ret) = match try_read().map(|s| s.dyn_try_handle_error(error)) {
            Ok(Ok(())) => (false, Ok(())),
            Ok(Err(error)) => (false, Err(error)),
            Err(error) => (true, Err(error.into())),
        };
        self.set_last_drop_failed(last_drop_failed);
        ret
    }
}

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

impl<T: GlobalDynFallibleTryDropStrategy + 'static> From<T> for Box<dyn GlobalDynFallibleTryDropStrategy> {
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
