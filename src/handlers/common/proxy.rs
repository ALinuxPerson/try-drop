#![allow(dead_code)]

use crate::handlers::common::Scope;
use crate::handlers::UninitializedError;
use std::marker::PhantomData;

#[cfg(feature = "global")]
use std::ops::{Deref, DerefMut};

#[cfg(feature = "global")]
use crate::handlers::common::Global;

#[cfg(feature = "thread-local")]
use crate::handlers::common::ThreadLocal;

#[cfg(feature = "thread-local")]
use crate::handlers::common::thread_local::{
    ThreadLocalDefinition,
    DefaultThreadLocalDefinition,
    ThreadLocal as ThreadLocalAbstracter,
};

#[cfg(feature = "global")]
use crate::handlers::common::global::{
    GlobalDefinition,
    DefaultGlobalDefinition,
    Global as GlobalAbstracter,
};

pub struct TheGreatAbstracter<D, S: Scope>(PhantomData<(D, S)>);

#[cfg(feature = "global")]
impl<D: GlobalDefinition> TheGreatAbstracter<D, Global>
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

#[cfg(feature = "global")]
impl<D: DefaultGlobalDefinition> TheGreatAbstracter<D, Global>
{
    pub fn read_or_default<R>(f: impl FnOnce(&D::Global) -> R) -> R {
        f(GlobalAbstracter::<D>::read_or_default().deref())
    }

    pub fn write_or_default<R>(f: impl FnOnce(&mut D::Global) -> R) -> R {
        f(GlobalAbstracter::<D>::write_or_default().deref_mut())
    }
}

#[cfg(feature = "thread-local")]
impl<D: ThreadLocalDefinition> TheGreatAbstracter<D, ThreadLocal>
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

#[cfg(feature = "thread-local")]
impl<D: DefaultThreadLocalDefinition> TheGreatAbstracter<D, ThreadLocal>
{
    pub fn read_or_default<R>(f: impl FnOnce(&D::ThreadLocal) -> R) -> R {
        ThreadLocalAbstracter::<D>::read_or_default(f)
    }

    pub fn write_or_default<R>(f: impl FnOnce(&mut D::ThreadLocal) -> R) -> R {
        ThreadLocalAbstracter::<D>::write_or_default(f)
    }
}
