use crate::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::uninit_error::UninitializedError;
use crate::{FallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy};
use anyhow::Error;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::boxed::Box;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};

static DROP_STRATEGY: RwLock<Option<Box<dyn GlobalDynFallibleTryDropStrategy>>> =
    parking_lot::const_rwlock(None);

const UNINITIALIZED_ERROR: &str = "the global drop strategy is not initialized yet";

#[cfg(not(feature = "ds-write"))]
pub type DefaultOnUninit = PanicOnUninit;

#[cfg(feature = "ds-write")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The global try drop strategy. This doesn't store anything, it just provides an interface
/// to the global try drop strategy, stored in a `static`.
#[cfg_attr(
feature = "derives",
derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalPrimaryDropStrategy<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl GlobalPrimaryDropStrategy<DefaultOnUninit> {
    pub const DEFAULT: Self = Self { extra_data: (), _on_uninit: PhantomData };
}

impl GlobalPrimaryDropStrategy<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    pub const ERROR_ON_UNINIT: Self = Self::on_uninit_error();

    /// Get an interface to the global try drop strategy. If there is no global try drop strategy
    /// initialized, this will error.
    pub const fn on_uninit_error() -> Self {
        Self { extra_data: (), _on_uninit: PhantomData }
    }
}

impl GlobalPrimaryDropStrategy<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Get an interface to the global try drop strategy. If there is no global try drop strategy
    /// initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self { extra_data: (), _on_uninit: PhantomData }
    }
}

#[cfg(feature = "ds-write")]
impl GlobalPrimaryDropStrategy<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Get an interface to the global try drop strategy. If there is no global try drop strategy
    /// initialized, this will set it to the default.
    pub const fn on_uninit_use_default() -> Self {
        Self { extra_data: (), _on_uninit: PhantomData }
    }
}

impl GlobalPrimaryDropStrategy<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Get an interface to the global try drop strategy. If there is no global try drop strategy
    /// initialized, this will set an internal flag stating that the drop failed.
    pub const fn on_uninit_flag() -> Self {
        Self { extra_data: AtomicBool::new(false), _on_uninit: PhantomData }
    }

    /// Check if acquiring a reference to the global drop strategy failed due to it not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(Ordering::Acquire)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, Ordering::Release)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryDropStrategy<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        try_read()
            .map(|strategy| strategy.dyn_try_handle_error(error))
            .map_err(Into::into)
            .and_then(std::convert::identity)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryDropStrategy<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read().dyn_try_handle_error(error)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryDropStrategy<UseDefaultOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read_or_default().dyn_try_handle_error(error)
    }
}

impl FallibleTryDropStrategy for GlobalPrimaryDropStrategy<FlagOnUninit> {
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

/// Install a new global try drop strategy. Must be a dynamic trait object.
pub fn install_dyn(strategy: Box<dyn GlobalDynFallibleTryDropStrategy>) {
    DROP_STRATEGY.write().replace(strategy);
}

/// Install a new global try drop strategy.
pub fn install(strategy: impl GlobalDynFallibleTryDropStrategy) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will return an error.
pub fn try_read() -> Result<
    MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>,
    UninitializedError,
> {
    let drop_strategy = DROP_STRATEGY.read();

    if drop_strategy.is_some() {
        Ok(RwLockReadGuard::map(drop_strategy, |drop_strategy| {
            drop_strategy.as_ref().unwrap()
        }))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will panic.
pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    try_read().expect(UNINITIALIZED_ERROR)
}

/// Get a reference to the try drop strategy. If there is no global try drop strategy initialized,
/// this will set it to the default then return it.
#[cfg(feature = "ds-write")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>
{
    drop(write_or_default());
    read()
}

/// Get a mutable reference to the try drop strategy. If there is no global try drop strategy
/// initialized, this will return an error.
pub fn try_write() -> Result<
    MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>,
    UninitializedError,
> {
    let drop_strategy = DROP_STRATEGY.write();

    if drop_strategy.is_some() {
        Ok(RwLockWriteGuard::map(drop_strategy, |drop_strategy| {
            drop_strategy.as_mut().unwrap()
        }))
    } else {
        Err(UninitializedError(()))
    }
}

/// Get a mutable reference to the try drop strategy. If there is no global try drop strategy
/// initialized, this will panic.
pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    try_write().expect(UNINITIALIZED_ERROR)
}

/// Get a mutable reference to the try drop strategy. If there is no global try drop strategy
/// initialized, this will set it to the default then return it.
#[cfg(feature = "ds-write")]
pub fn write_or_default(
) -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    use crate::drop_strategies::WriteDropStrategy;

    RwLockWriteGuard::map(DROP_STRATEGY.write(), |drop_strategy| {
        drop_strategy.get_or_insert_with(|| {
            let mut strategy = WriteDropStrategy::stderr();
            strategy.prelude("error: ");
            Box::new(strategy)
        })
    })
}

/// Uninstall or remove the global try drop strategy.
pub fn uninstall() {
    *DROP_STRATEGY.write() = None
}
