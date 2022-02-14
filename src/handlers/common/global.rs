pub(crate) mod imports {
    use std::boxed::Box;
    use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard};
    use crate::handlers::UninitializedError;
}

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

macro_rules! global_methods {
    (
        Global = $global:ident;
        GenericStrategy = $generic_strategy:ident;
        DynStrategy = $dyn_strategy:ident;
        feature = $feature:literal;

        $(#[$($install_dyn_tt:tt)*])*
        install_dyn;

        $(#[$($install_tt:tt)*])*
        install;

        $(#[$($try_read_tt:tt)*])*
        try_read;

        $(#[$($read_tt:tt)*])*
        read;

        $(#[$($try_write_tt:tt)*])*
        try_write;

        $(#[$($write_tt:tt)*])*
        write;

        $(#[$($uninstall_tt:tt)*])*
        uninstall;

        $(#[$($read_or_default_tt:tt)*])*
        read_or_default;

        $(#[$($write_or_default_tt:tt)*])*
        write_or_default;
    ) => {
        use $crate::handlers::common::imports::*;

        $(#[$($install_dyn_tt)*])*
        pub fn install_dyn(strategy: $dyn_strategy) {
            $global::install_dyn(strategy)
        }

        $(#[$($install_tt)*])*
        pub fn install(strategy: impl $generic_strategy) {
            $global::install(strategy)
        }

        $(#[$($try_read_tt)*])*
        pub fn try_read() -> Result<MappedRwLockReadGuard<'static, $dyn_strategy>, UninitializedError> {
            $global::try_read()
        }

        $(#[$($read_tt)*])*
        pub fn read() -> MappedRwLockReadGuard<'static, $dyn_strategy> {
            $global::read()
        }

        $(#[$($try_write_tt)*])*
        pub fn try_write() -> Result<MappedRwLockWriteGuard<'static, $dyn_strategy>, UninitializedError> {
            $global::try_write()
        }

        $(#[$($write_tt)*])*
        pub fn write() -> MappedRwLockWriteGuard<'static, $dyn_strategy> {
            $global::write()
        }

        $(#[$($uninstall_tt)*])*
        pub fn uninstall() {
            $global::uninstall()
        }

        $(#[$($read_or_default_tt)*])*
        #[cfg(feature = $feature)]
        pub fn read_or_default() -> MappedRwLockReadGuard<'static, $dyn_strategy> {
            $global::read_or_default()
        }

        $(#[$($write_or_default_tt)*])*
        #[cfg(feature = $feature)]
        pub fn write_or_default() -> MappedRwLockWriteGuard<'static, $dyn_strategy> {
            $global::write_or_default()
        }
    };
}
