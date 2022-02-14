use std::marker::PhantomData;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::handlers::UninitializedError;

pub trait GlobalDefinition {
    const UNINITIALIZED_ERROR: &'static str;
    type Global: 'static;

    fn global() -> &'static RwLock<Option<Self::Global>>;
    fn default() -> Self::Global;
}

pub struct Global<T: GlobalDefinition>(PhantomData<T>);

impl<T: GlobalDefinition> Global<T> {
    pub fn install_dyn(strategy: T::Global) {
        T::global().write().replace(strategy);
    }

    pub fn install(strategy: impl Into<T::Global>) {
        Self::install_dyn(strategy.into())
    }

    pub fn try_read() -> Result<
        MappedRwLockReadGuard<'static, T::Global>,
        UninitializedError,
    > {
        let global = T::global().read();

        if global.is_some() {
            Ok(RwLockReadGuard::map(global, |global| global.as_ref().unwrap()))
        } else {
            Err(UninitializedError(()))
        }
    }

    pub fn read() -> MappedRwLockReadGuard<'static, T::Global> {
        Self::try_read().expect(T::UNINITIALIZED_ERROR)
    }

    pub fn read_or_default() -> MappedRwLockReadGuard<'static, T::Global> {
        drop(Self::write_or_default());
        Self::read()
    }

    pub fn try_write() -> Result<
        MappedRwLockWriteGuard<'static, T::Global>,
        UninitializedError,
    > {
        let global = T::global().write();

        if global.is_some() {
            Ok(RwLockWriteGuard::map(global, |global| global.as_mut().unwrap()))
        } else {
            Err(UninitializedError(()))
        }
    }

    pub fn write() -> MappedRwLockWriteGuard<'static, T::Global> {
        Self::try_write().expect(T::UNINITIALIZED_ERROR)
    }

    pub fn write_or_default() -> MappedRwLockWriteGuard<'static, T::Global> {
        RwLockWriteGuard::map(T::global().write(), |drop_strategy| {
            drop_strategy.get_or_insert_with(T::default)
        })
    }

    pub fn uninstall() {
        *T::global().write() = None
    }
}
