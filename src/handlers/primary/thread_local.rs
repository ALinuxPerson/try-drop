//! Manage the thread local primary handler.

use super::{Abstracter, DefaultOnUninit};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::thread_local::{
    scope_guard::ScopeGuard as GenericScopeGuard, DefaultThreadLocalDefinition,
    ThreadLocal as GenericThreadLocal, ThreadLocalDefinition,
};
use crate::handlers::common::Primary;
use crate::handlers::common::ThreadLocal as ThreadLocalScope;
use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use std::boxed::Box;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::thread::LocalKey;
use std::{convert, thread_local};

#[cfg(feature = "ds-write")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

pub type ThreadLocalPrimaryHandler<OU = DefaultOnUninit> =
    CommonHandler<OU, ThreadLocalScope, Primary>;

pub static DEFAULT_THREAD_LOCAL_PRIMARY_HANDLER: ThreadLocalPrimaryHandler =
    ThreadLocalPrimaryHandler::DEFAULT;

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        Abstracter::<ThreadLocalScope>::try_read(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(convert::identity)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        Abstracter::<ThreadLocalScope>::try_read(|strategy| strategy.dyn_try_handle_error(error))
            .expect(<Primary as ThreadLocalDefinition>::UNINITIALIZED_ERROR)
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        Abstracter::<ThreadLocalScope>::read_or_default(|strategy| {
            strategy.dyn_try_handle_error(error)
        })
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        let (last_drop_failed, ret) =
            match Abstracter::<ThreadLocalScope>::try_read(|s| s.dyn_try_handle_error(error)) {
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
    const UNINITIALIZED_ERROR: &'static str =
        "the thread local primary handler is not initialized yet";
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

impl<T: ThreadLocalFallibleTryDropStrategy> From<T>
    for Box<dyn ThreadLocalFallibleTryDropStrategy>
{
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
