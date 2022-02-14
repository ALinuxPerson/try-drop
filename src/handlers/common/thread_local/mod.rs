pub mod scope_guard;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::thread::LocalKey;
use crate::handlers::common::Handler;
use crate::handlers::common::thread_local::scope_guard::ScopeGuard;
use crate::handlers::UninitializedError;

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
        T::thread_local()
            .with(|cell| f(cell.borrow_mut().get_or_insert_with(T::default)))
    }
}
