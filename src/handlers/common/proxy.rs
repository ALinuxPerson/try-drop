use crate::handlers::common::global::Global as GlobalAbstracter;
use crate::handlers::common::global::{DefaultGlobalDefinition, GlobalDefinition};
use crate::handlers::common::thread_local::ThreadLocalDefinition;
use crate::handlers::common::thread_local::{
    DefaultThreadLocalDefinition, ThreadLocal as ThreadLocalAbstracter,
};
use crate::handlers::common::{Global, Scope, ThreadLocal};
use crate::handlers::UninitializedError;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct TheGreatAbstracter<D, S>(PhantomData<(D, S)>)
where
    D: GlobalDefinition,
    D: ThreadLocalDefinition,
    S: Scope;

impl<D> TheGreatAbstracter<D, Global>
where
    D: GlobalDefinition,
    D: ThreadLocalDefinition,
{
    pub fn install(strategy: impl Into<D::Global>) {
        GlobalAbstracter::<D>::install(strategy)
    }

    pub fn install_dyn(strategy: D::Global) {
        GlobalAbstracter::<D>::install_dyn(strategy)
    }

    pub fn try_read<R>(f: impl FnOnce(&D::Global) -> R) -> Result<R, UninitializedError> {
        GlobalAbstracter::<D>::try_read().map(|lock| f(lock.deref()))
    }

    pub fn read<R>(f: impl FnOnce(&D::Global) -> R) -> R {
        f(GlobalAbstracter::<D>::read().deref())
    }

    pub fn try_write<R>(f: impl FnOnce(&mut D::Global) -> R) -> Result<R, UninitializedError> {
        GlobalAbstracter::<D>::try_write().map(|mut lock| f(lock.deref_mut()))
    }

    pub fn write<R>(f: impl FnOnce(&mut D::Global) -> R) -> R {
        f(GlobalAbstracter::<D>::write().deref_mut())
    }

    pub fn uninstall() {
        GlobalAbstracter::<D>::uninstall()
    }
}

impl<D> TheGreatAbstracter<D, Global>
where
    D: DefaultGlobalDefinition,
    D: DefaultThreadLocalDefinition,
{
    pub fn read_or_default<R>(f: impl FnOnce(&D::Global) -> R) -> R {
        f(GlobalAbstracter::<D>::read_or_default().deref())
    }

    pub fn write_or_default<R>(f: impl FnOnce(&mut D::Global) -> R) -> R {
        f(GlobalAbstracter::<D>::write_or_default().deref_mut())
    }
}

impl<D> TheGreatAbstracter<D, ThreadLocal>
where
    D: GlobalDefinition,
    D: ThreadLocalDefinition,
{
    pub fn install(strategy: impl Into<D::ThreadLocal>) {
        ThreadLocalAbstracter::<D>::install(strategy)
    }

    pub fn install_dyn(strategy: D::ThreadLocal) {
        ThreadLocalAbstracter::<D>::install_dyn(strategy)
    }

    pub fn try_read<R>(f: impl FnOnce(&D::ThreadLocal) -> R) -> Result<R, UninitializedError> {
        ThreadLocalAbstracter::<D>::try_read(f)
    }

    pub fn read<R>(f: impl FnOnce(&D::ThreadLocal) -> R) -> R {
        ThreadLocalAbstracter::<D>::read(f)
    }

    pub fn try_write<R>(f: impl FnOnce(&mut D::ThreadLocal) -> R) -> Result<R, UninitializedError> {
        ThreadLocalAbstracter::<D>::try_write(f)
    }

    pub fn write<R>(f: impl FnOnce(&mut D::ThreadLocal) -> R) -> R {
        ThreadLocalAbstracter::<D>::write(f)
    }

    pub fn uninstall() {
        ThreadLocalAbstracter::<D>::uninstall()
    }
}

impl<D> TheGreatAbstracter<D, ThreadLocal>
where
    D: DefaultGlobalDefinition,
    D: DefaultThreadLocalDefinition,
{
    pub fn read_or_default<R>(f: impl FnOnce(&D::ThreadLocal) -> R) -> R {
        ThreadLocalAbstracter::<D>::read_or_default(f)
    }

    pub fn write_or_default<R>(f: impl FnOnce(&mut D::ThreadLocal) -> R) -> R {
        ThreadLocalAbstracter::<D>::write_or_default(f)
    }
}
