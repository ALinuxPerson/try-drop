//! try-drop - utilities for drops which can fail

#![allow(drop_bounds)]
#![no_std]

#[cfg(feature = "shrinkwraprs")]
#[macro_use]
extern crate shrinkwraprs;

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
pub mod drop_adapter;

pub mod double;

#[cfg(feature = "global")]
pub mod global;

pub mod prelude;

#[cfg(feature = "debugging")]
pub mod debugging;

mod infallible;

use crate::double::DoubleDropStrategy;
pub use anyhow::Error;
pub use double::{DoubleDropStrategyHandler, DoubleDropStrategyRef};
pub use infallible::Infallible;

#[cfg(feature = "global")]
mod global_crate_root;

#[cfg(feature = "global")]
pub use global_crate_root::*;

pub trait SpecificTryDrop {
    type Error: Into<anyhow::Error>;
    type DoubleDropStrategy: DoubleDropStrategy;
    type DropStrategy: FallibleTryDropStrategy;

    fn double_drop_strategy(&self) -> &Self::DoubleDropStrategy;
    fn drop_strategy(&self) -> &Self::DropStrategy;

    /// Execute the fallible destructor for this type. This function is unsafe because if this is
    /// called outside of a [`Drop::drop`] context, once the scope of the object implementing trait
    /// ends, this function will be called twice, potentially resulting in a double-free.
    ///
    /// Use [`DropAdapter`] to ensure that the destructor is only called once.
    ///
    /// # Safety
    /// The caller must ensure that this function is called within a [`Drop::drop`] context.
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error>;
}

pub trait TryDrop {
    type Error: Into<anyhow::Error>;

    /// Execute the fallible destructor for this type. This function is unsafe because if this is
    /// called outside of a [`Drop::drop`] context, once the scope of the object implementing trait
    /// ends, this function will be called twice, potentially resulting in a double-free.
    ///
    /// Use [`DropAdapter`] to ensure that the destructor is only called once.
    ///
    /// # Safety
    /// The caller must ensure that this function is called within a [`Drop::drop`] context.
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error>;
}

pub trait FallibleTryDropStrategy {
    type Error: Into<anyhow::Error>;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error>;
}

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct FallibleTryDropStrategyRef<'a, T: FallibleTryDropStrategy>(pub &'a T);

impl<'a, T: FallibleTryDropStrategy> FallibleTryDropStrategy for FallibleTryDropStrategyRef<'a, T> {
    type Error = T::Error;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.0.try_handle_error(error)
    }
}

pub trait DynFallibleTryDropStrategy {
    fn dyn_try_handle_error(&self, error: anyhow::Error) -> anyhow::Result<()>;
}

#[cfg(feature = "global")]
#[cfg(not(feature = "downcast-rs"))]
pub trait GlobalDynFallibleTryDropStrategy: ThreadSafe + DynFallibleTryDropStrategy {}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
pub trait GlobalDynFallibleTryDropStrategy:
    ThreadSafe + downcast_rs::DowncastSync + DynFallibleTryDropStrategy
{
}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
downcast_rs::impl_downcast!(sync GlobalDynFallibleTryDropStrategy);

#[cfg(feature = "global")]
impl<T: ThreadSafe + DynFallibleTryDropStrategy> GlobalDynFallibleTryDropStrategy for T {}

impl<FTDS: FallibleTryDropStrategy> DynFallibleTryDropStrategy for FTDS {
    fn dyn_try_handle_error(&self, error: anyhow::Error) -> anyhow::Result<()> {
        self.try_handle_error(error).map_err(Into::into)
    }
}

pub trait TryDropStrategy {
    fn handle_error(&self, error: anyhow::Error);
}

impl<TDS: TryDropStrategy> FallibleTryDropStrategy for TDS {
    type Error = Infallible;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.handle_error(error);
        Ok(())
    }
}

pub trait ThreadSafe: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> ThreadSafe for T {}
