pub mod scope_guard;
pub(crate) mod imports {
    pub use crate::handlers::UninitializedError;
    pub use crate::{DynFallibleTryDropStrategy, ThreadLocalFallibleTryDropStrategy};
    pub use std::boxed::Box;
}

use crate::handlers::common::thread_local::scope_guard::ScopeGuard;
use crate::handlers::common::Handler;
use crate::handlers::UninitializedError;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::thread::LocalKey;

macro_rules! thread_local_methods {
    (
        ThreadLocal = $thread_local:ident;
        ScopeGuard = $scope_guard:ident;
        GenericStrategy = $generic_strategy:ident;
        DynStrategy = $dyn_strategy:ident;
        feature = $feature:literal;

        $(#[$($install_meta:meta)*])*
        install;

        $(#[$($install_dyn_meta:meta)*])*
        install_dyn;

        $(#[$($read_meta:meta)*])*
        read;

        $(#[$($try_read_meta:meta)*])*
        try_read;

        $(#[$($read_or_default_meta:meta)*])*
        read_or_default;

        $(#[$($write_meta:meta)*])*
        write;

        $(#[$($try_write_meta:meta)*])*
        try_write;

        $(#[$($write_or_default_meta:meta)*])*
        write_or_default;

        $(#[$($uninstall_meta:meta)*])*
        uninstall;

        $(#[$($take_meta:meta)*])*
        take;

        $(#[$($replace_meta:meta)*])*
        replace;

        $(#[$($replace_dyn_meta:meta)*])*
        replace_dyn;

        $(#[$($scope_meta:meta)*])*
        scope;

        $(#[$($scope_dyn_meta:meta)*])*
        scope_dyn;
    ) => {
        #[allow(unused_imports)]
        use $crate::handlers::common::thread_local::imports::*;

        $(#[$($install_meta)*])*
        pub fn install(strategy: impl $generic_strategy) {
            $thread_local::install(strategy)
        }

        $(#[$($install_dyn_meta)*])*
        pub fn install_dyn(strategy: $dyn_strategy) {
            $thread_local::install_dyn(strategy)
        }

        $(#[$($read_meta)*])*
        pub fn read<T>(f: impl FnOnce(&$dyn_strategy) -> T) -> T {
            $thread_local::read(f)
        }

        $(#[$($try_read_meta)*])*
        pub fn try_read<T>(f: impl FnOnce(&$dyn_strategy) -> T) -> Result<T, UninitializedError> {
            $thread_local::try_read(f)
        }

        $(#[$($read_or_default_meta)*])*
        #[cfg(feature = $feature)]
        pub fn read_or_default<T>(f: impl FnOnce(&$dyn_strategy) -> T) -> T {
            $thread_local::read_or_default(f)
        }

        $(#[$($write_meta)*])*
        pub fn write<T>(f: impl FnOnce(&mut $dyn_strategy) -> T) -> T {
            $thread_local::write(f)
        }

        $(#[$($try_write_meta)*])*
        pub fn try_write<T>(f: impl FnOnce(&mut $dyn_strategy) -> T) -> Result<T, UninitializedError> {
            $thread_local::try_write(f)
        }

        $(#[$($write_or_default_meta)*])*
        #[cfg(feature = $feature)]
        pub fn write_or_default<T>(f: impl FnOnce(&mut $dyn_strategy) -> T) -> T {
            $thread_local::write_or_default(f)
        }

        $(#[$($uninstall_meta)*])*
        pub fn uninstall() {
            $thread_local::uninstall()
        }

        $(#[$($take_meta)*])*
        pub fn take() -> Option<$dyn_strategy> {
            $thread_local::take()
        }

        $(#[$($replace_meta)*])*
        pub fn replace(strategy: impl $generic_strategy) -> Option<$dyn_strategy> {
            $thread_local::replace(strategy)
        }

        $(#[$($replace_dyn_meta)*])*
        pub fn replace_dyn(strategy: $dyn_strategy) -> Option<$dyn_strategy> {
            $thread_local::replace_dyn(strategy)
        }

        $(#[$($scope_meta)*])*
        pub fn scope(strategy: impl $generic_strategy) -> $scope_guard {
            $thread_local::scope(strategy)
        }

        $(#[$($scope_dyn_meta)*])*
        pub fn scope_dyn(strategy: $dyn_strategy) -> $scope_guard {
            $thread_local::scope_dyn(strategy)
        }
    };
}

pub trait ThreadLocalDefinition: Handler {
    const UNINITIALIZED_ERROR: &'static str;
    const DYN: &'static str;
    type ThreadLocal: 'static;

    fn thread_local() -> &'static LocalKey<RefCell<Option<Self::ThreadLocal>>>;
    fn locked() -> &'static LocalKey<RefCell<bool>>;
}

pub trait DefaultThreadLocalDefinition: ThreadLocalDefinition {
    fn default() -> Self::ThreadLocal;
}

pub struct ThreadLocal<T: ThreadLocalDefinition>(PhantomData<T>);

impl<T: ThreadLocalDefinition> ThreadLocal<T> {
    pub fn read<R>(f: impl FnOnce(&T::ThreadLocal) -> R) -> R {
        Self::try_read(f).expect(T::UNINITIALIZED_ERROR)
    }

    pub fn try_read<R>(f: impl FnOnce(&T::ThreadLocal) -> R) -> Result<R, UninitializedError> {
        T::thread_local().with(|cell| {
            cell.borrow_mut()
                .as_ref()
                .map(f)
                .ok_or(UninitializedError(()))
        })
    }

    pub fn write<R>(f: impl FnOnce(&mut T::ThreadLocal) -> R) -> R {
        Self::try_write(f).expect(T::UNINITIALIZED_ERROR)
    }

    pub fn try_write<R>(f: impl FnOnce(&mut T::ThreadLocal) -> R) -> Result<R, UninitializedError> {
        T::thread_local().with(|cell| {
            cell.borrow_mut()
                .as_mut()
                .map(f)
                .ok_or(UninitializedError(()))
        })
    }

    pub fn install(strategy: impl Into<T::ThreadLocal>) {
        Self::install_dyn(strategy.into())
    }

    pub fn install_dyn(strategy: T::ThreadLocal) {
        Self::replace_dyn(strategy);
    }

    pub fn uninstall() {
        Self::take();
    }

    pub fn take() -> Option<T::ThreadLocal> {
        T::thread_local().with(|cell| cell.borrow_mut().take())
    }

    pub fn replace(new: impl Into<T::ThreadLocal>) -> Option<T::ThreadLocal> {
        Self::replace_dyn(new.into())
    }

    pub fn replace_dyn(new: T::ThreadLocal) -> Option<T::ThreadLocal> {
        T::thread_local().with(|cell| cell.borrow_mut().replace(new))
    }

    pub fn scope(strategy: impl Into<T::ThreadLocal>) -> ScopeGuard<T> {
        Self::scope_dyn(strategy.into())
    }

    pub fn scope_dyn(strategy: T::ThreadLocal) -> ScopeGuard<T> {
        ScopeGuard::new_dyn(strategy)
    }
}

impl<T: DefaultThreadLocalDefinition> ThreadLocal<T> {
    pub fn read_or_default<R>(f: impl FnOnce(&T::ThreadLocal) -> R) -> R {
        T::thread_local().with(|cell| {
            let mut strategy = cell.borrow_mut();
            let strategy = strategy.get_or_insert_with(T::default);
            let strategy = &*strategy;
            f(strategy)
        })
    }

    pub fn write_or_default<R>(f: impl FnOnce(&mut T::ThreadLocal) -> R) -> R {
        T::thread_local().with(|cell| f(cell.borrow_mut().get_or_insert_with(T::default)))
    }
}
