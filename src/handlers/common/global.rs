use std::marker::PhantomData;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::handlers::common::Handler;
use crate::handlers::UninitializedError;

pub trait GlobalDefinition: Handler {
    const UNINITIALIZED_ERROR: &'static str;
    type Global: 'static;

    fn global() -> &'static RwLock<Option<Self::Global>>;
}

pub trait DefaultGlobalDefinition: GlobalDefinition {
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

    pub fn uninstall() {
        *T::global().write() = None
    }
}

impl<T: DefaultGlobalDefinition> Global<T> {
    pub fn read_or_default() -> MappedRwLockReadGuard<'static, T::Global> {
        drop(Self::write_or_default());
        Self::read()
    }

    pub fn write_or_default() -> MappedRwLockWriteGuard<'static, T::Global> {
        RwLockWriteGuard::map(T::global().write(), |drop_strategy| {
            drop_strategy.get_or_insert_with(T::default)
        })
    }
}

macro_rules! globals {
    ($name:ident where Global: $global:ty, GenericGlobal: $generic_global:ident) => {
        pub fn install_dyn(strategy: $global) {
            $name::install_dyn(strategy)
        }

        pub fn install(strategy: $generic_global) {
            $name::install(strategy)
        }

        pub fn try_read() -> Result<
            MappedRwLockReadGuard<'static, $global>,
            UninitializedError,
        > {
            $name::try_read()
        }

        pub fn read() -> MappedRwLockReadGuard<'static, $global> {
            $name::read()
        }

        pub fn read_or_default() -> MappedRwLockReadGuard<'static, $global> {
            $name::read_or_default()
        }

        pub fn try_write() -> Result<
            MappedRwLockWriteGuard<'static, $global>,
            UninitializedError,
        > {
            $name::try_write()
        }

        pub fn write() -> MappedRwLockWriteGuard<'static, $global> {
            $name::write()
        }

        pub fn write_or_default() -> MappedRwLockWriteGuard<'static, $global> {
            $name::write_or_default()
        }

        pub fn uninstall() {
            $name::uninstall()
        }
    };
}
