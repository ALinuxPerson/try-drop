//! Manage the thread local primary handler.

use std::boxed::Box;
use std::cell::RefCell;
use std::thread::LocalKey;
use crate::handlers::common::Primary;
use crate::handlers::common::thread_local::{
    DefaultThreadLocalDefinition,
    ThreadLocalDefinition,
    ThreadLocal as GenericThreadLocal,
    scope_guard::ScopeGuard as GenericScopeGuard,
};
use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::thread_local;

#[cfg(feature = "ds-write")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

const UNINITIALIZED_ERROR: &str = "the thread local primary handler is not initialized yet";

/// The default thing to do when the primary thread local primary handler is uninitialized, that is
/// to panic.
#[cfg(not(feature = "ds-write"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the primary thread local primary handler is uninitialized, that is
/// to use the default strategy. Note that this mutates the thread local primary handler.
#[cfg(feature = "ds-write")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The default thread local primary handler.
pub static DEFAULT_THREAD_LOCAL_PRIMARY_HANDLER: ThreadLocalPrimaryHandler =
    ThreadLocalPrimaryHandler::DEFAULT;

/// The thread local primary handler. This doesn't store anything, it just provides an interface
/// to the thread local primary handler, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct ThreadLocalPrimaryHandler<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl ThreadLocalPrimaryHandler<DefaultOnUninit> {
    /// The default thread local primary handler.
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _on_uninit: PhantomData,
    };
}

impl Default for ThreadLocalPrimaryHandler<DefaultOnUninit> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ThreadLocalPrimaryHandler<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    pub const ERROR_ON_UNINIT: Self = Self::on_uninit_error();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl ThreadLocalPrimaryHandler<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

#[cfg(feature = "ds-write")]
impl ThreadLocalPrimaryHandler<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will set it to the default primary handler.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl ThreadLocalPrimaryHandler<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Create a new interface to the thread local primary handler. If the thread local primary
    /// handler is not initialized, this will set an internal flag stating that the drop failed.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _on_uninit: PhantomData,
        }
    }

    /// Check if the last drop failed due to the primary thread local primary handler not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        try_read(|strategy| strategy.dyn_try_handle_error(error)).expect(UNINITIALIZED_ERROR)
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        read_or_default(|strategy| strategy.dyn_try_handle_error(error))
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        let (last_drop_failed, ret) = match try_read(|s| s.dyn_try_handle_error(error)) {
            Ok(Ok(())) => (false, Ok(())),
            Ok(Err(error)) => (false, Err(error)),
            Err(error) => (true, Err(error.into())),
        };
        self.set_last_drop_failed(last_drop_failed);
        ret
    }
}

thread_local! {
    static PRIMARY_HANDLER: RefCell<Option<Box<dyn ThreadLocalFallibleTryDropStrategy>>> = RefCell::new(None);
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

impl ThreadLocalDefinition for Primary {
    const UNINITIALIZED_ERROR: &'static str = "the thread local primary handler is not initialized yet";
    const DYN: &'static str = "ThreadLocalFallibleTryDropStrategy";
    type ThreadLocal = Box<dyn ThreadLocalFallibleTryDropStrategy>;

    fn thread_local() -> &'static LocalKey<RefCell<Option<Self::ThreadLocal>>> {
        &PRIMARY_HANDLER
    }

    fn locked() -> &'static LocalKey<RefCell<bool>> {
        &LOCKED
    }
}

#[cfg(feature = "ds-write")]
impl DefaultThreadLocalDefinition for Primary {
    fn default() -> Self::ThreadLocal {
        let mut strategy = crate::drop_strategies::WriteDropStrategy::stderr();
        strategy.prelude("error: ");
        Box::new(strategy)
    }
}

impl<T: ThreadLocalFallibleTryDropStrategy> From<T> for Box<dyn ThreadLocalFallibleTryDropStrategy> {
    fn from(strategy: T) -> Self {
        Box::new(strategy)
    }
}

type ThreadLocal = GenericThreadLocal<Primary>;
pub type ScopeGuard = GenericScopeGuard<Primary>;
pub type BoxDynFallibleTryDropStrategy = Box<dyn ThreadLocalFallibleTryDropStrategy>;

thread_local_methods! {
    ThreadLocal = ThreadLocal;
    ScopeGuard = ScopeGuard;
    GenericStrategy = ThreadLocalFallibleTryDropStrategy;
    DynStrategy = BoxDynFallibleTryDropStrategy;
    feature = "ds-write";

    install;
    install_dyn;
    read;
    try_read;
    read_or_default;
    write;
    try_write;
    write_or_default;
    uninstall;
    take;
    replace;
    replace_dyn;
    scope;
    scope_dyn;
}

