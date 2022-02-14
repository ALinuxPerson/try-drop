use std::convert;
use std::marker::PhantomData;
use anyhow::Error;
use crate::FallibleTryDropStrategy;
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::{Global, Primary, Scope, ThreadLocal};
use crate::handlers::common::global::GlobalDefinition;
use crate::handlers::common::proxy::TheGreatAbstracter;
use crate::handlers::common::thread_local::ThreadLocalDefinition;
use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit, UseDefaultOnUninit};

pub type GlobalPrimaryHandler<OU = DefaultOnUninit> = CommonHandler<OU, Global, Primary>;
pub type ThreadLocalPrimaryHandler<OU = DefaultOnUninit> = CommonHandler<OU, ThreadLocal, Primary>;

pub static DEFAULT_GLOBAL_PRIMARY_HANDLER: GlobalPrimaryHandler = GlobalPrimaryHandler::DEFAULT;
pub static DEFAULT_THREAD_LOCAL_PRIMARY_HANDLER: ThreadLocalPrimaryHandler = ThreadLocalPrimaryHandler::DEFAULT;

impl FallibleTryDropStrategy for GlobalPrimaryHandler<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        Abstracter::<Global>::try_read(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(convert::identity)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        Abstracter::<ThreadLocal>::try_read(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(convert::identity)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryHandler<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        Abstracter::<Global>::try_read(|strategy| strategy.dyn_try_handle_error(error))
            .expect(<Primary as GlobalDefinition>::UNINITIALIZED_ERROR)
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        Abstracter::<ThreadLocal>::try_read(|strategy| strategy.dyn_try_handle_error(error))
            .expect(<Primary as ThreadLocalDefinition>::UNINITIALIZED_ERROR)
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for GlobalPrimaryHandler<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        Abstracter::<Global>::read_or_default(|strategy| strategy.dyn_try_handle_error(error))
    }
}

#[cfg(feature = "ds-write")]
impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        Abstracter::<ThreadLocal>::read_or_default(|strategy| strategy.dyn_try_handle_error(error))
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        let (last_drop_failed, ret) = match Abstracter::<Global>::try_read(|s| s.dyn_try_handle_error(error)) {
            Ok(Ok(())) => (false, Ok(())),
            Ok(Err(error)) => (false, Err(error)),
            Err(error) => (true, Err(error.into())),
        };
        self.set_last_drop_failed(last_drop_failed);
        ret
    }
}

impl FallibleTryDropStrategy for ThreadLocalPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        let (last_drop_failed, ret) = match Abstracter::<ThreadLocal>::try_read(|s| s.dyn_try_handle_error(error)) {
            Ok(Ok(())) => (false, Ok(())),
            Ok(Err(error)) => (false, Err(error)),
            Err(error) => (true, Err(error.into())),
        };
        self.set_last_drop_failed(last_drop_failed);
        ret
    }
}

