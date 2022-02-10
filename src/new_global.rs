use std::boxed::Box;
use std::marker::PhantomData;
use anyhow::Error;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::drop_strategies::PanicDropStrategy;
use crate::{FallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy};
use crate::on_uninit::{ErrorOnUninit, OnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::uninit_error::UninitializedError;

static DROP_STRATEGY: RwLock<Option<Box<dyn GlobalDynFallibleTryDropStrategy>>> = parking_lot::const_rwlock(None);

const UNINITIALIZED_ERROR: &str = "the global drop strategy is not initialized yet";

pub struct GlobalFallibleTryDropStrategy<OU: OnUninit>(PhantomData<OU>);

impl GlobalFallibleTryDropStrategy<ErrorOnUninit> {
    pub const fn on_uninit_error() -> Self {
        Self(PhantomData)
    }
}

impl GlobalFallibleTryDropStrategy<PanicOnUninit> {
    pub const fn on_uninit_panic() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "ds-write")]
impl GlobalFallibleTryDropStrategy<UseDefaultOnUninit> {
    pub const fn on_uninit_use_default() -> Self {
        Self(PhantomData)
    }
}

impl FallibleTryDropStrategy for GlobalFallibleTryDropStrategy<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        try_read().map(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for GlobalFallibleTryDropStrategy<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read().dyn_try_handle_error(error)
    }
}

impl FallibleTryDropStrategy for GlobalFallibleTryDropStrategy<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read_or_default().dyn_try_handle_error(error)
    }
}

pub fn install_dyn(strategy: Box<dyn GlobalDynFallibleTryDropStrategy>) {
    DROP_STRATEGY.write().replace(strategy);
}

pub fn install(strategy: impl GlobalDynFallibleTryDropStrategy) {
    install_dyn(Box::new(strategy))
}

pub fn try_read() -> Result<MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>, UninitializedError> {
    let drop_strategy = DROP_STRATEGY.read();

    if drop_strategy.is_some() {
        Ok(RwLockReadGuard::map(drop_strategy, |drop_strategy| drop_strategy.as_ref().unwrap()))
    } else {
        Err(UninitializedError(()))
    }
}

pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    try_read().expect(UNINITIALIZED_ERROR)
}

#[cfg(feature = "ds-write")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    drop(write_or_default());
    read()
}

pub fn try_write() -> Result<MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>, UninitializedError> {
    let drop_strategy = DROP_STRATEGY.write();

    if drop_strategy.is_some() {
        Ok(RwLockWriteGuard::map(drop_strategy, |drop_strategy| drop_strategy.as_mut().unwrap()))
    } else {
        Err(UninitializedError(()))
    }
}

pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    try_write().expect(UNINITIALIZED_ERROR)
}

#[cfg(feature = "ds-write")]
pub fn write_or_default() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    RwLockWriteGuard::map(
        DROP_STRATEGY.write(),
        |drop_strategy| drop_strategy.get_or_insert_with(|| Box::new(PanicDropStrategy::DEFAULT))
    )
}

pub fn uninstall() {
    *DROP_STRATEGY.write() = None
}
