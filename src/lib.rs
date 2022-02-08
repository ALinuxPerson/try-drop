#![doc = include_str!("../README.md")]
#![allow(drop_bounds)]
#![warn(missing_docs)]
#![no_std]

#[cfg(feature = "shrinkwraprs")]
#[macro_use]
extern crate shrinkwraprs;

#[cfg(feature = "std")]
extern crate std;

pub mod fallback;

#[cfg(feature = "global")]
pub mod global;

pub mod prelude;

pub mod drop_strategies;

mod infallible;

use crate::fallback::FallbackTryDropStrategy;
pub use anyhow::Error;
pub use fallback::{FallbackTryDropStrategyHandler, FallbackTryDropStrategyRef};
pub use infallible::Infallible;

#[cfg(feature = "global")]
mod global_crate_root;

#[cfg(feature = "global")]
pub use global_crate_root::*;

#[cfg(not(feature = "global"))]
pub use self::PureTryDrop as TryDrop;

#[cfg(feature = "global")]
pub use self::ImpureTryDrop as TryDrop;

/// A trait for types which can be dropped, but which may fail to do so.
///
/// This is a pure version of try drop, meaning that the drop strategies have to be explicitly
/// specified, which means it does not depend on a global try drop strategy.
///
/// # Gotchas
/// Implementing this trait is not enough to make it droppable. In order for the try drop strategy
/// to be run, you need to put your type in a [`DropAdapter`].
///
/// An easier way to make your type droppable is to call [`TryDrop::adapt`] on it.
pub trait PureTryDrop {
    /// The type of the error that may occur during drop.
    type Error: Into<anyhow::Error>;

    /// The type which will be used if the drop strategy fails.
    type FallbackTryDropStrategy: FallbackTryDropStrategy;

    /// The type which will be used if dropping fails.
    type TryDropStrategy: FallibleTryDropStrategy;

    /// Get a reference to the fallback try drop strategy.
    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy;

    /// Get a reference to the try drop strategy.
    fn try_drop_strategy(&self) -> &Self::TryDropStrategy;

    /// Adapts this type to take advantage of the specified try drop strategies.
    ///
    /// # Notes
    /// If [`Self`] implements [`Copy`], and you call this function, at first it seems like there
    /// would be a soundness hole:
    ///
    /// ```rust
    /// use try_drop::{Infallible, PureTryDrop, TryDrop};
    ///
    /// #[derive(Copy, Clone)]
    /// struct T(usize);
    ///
    /// impl TryDrop for T {
    ///     type Error = Infallible;
    ///
    ///     unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
    ///         self.0 += 1;
    ///         println!("{}", self.0);
    ///         Ok(())
    ///     }
    /// }
    ///
    /// // this is valid code and does not result in a compilation error
    /// let t = T.adapt().adapt();
    /// ```
    ///
    /// You'd think the output would be:
    ///
    /// ```ignore
    /// 1
    /// 2
    /// ```
    ///
    /// However, it's actually:
    ///
    /// ```ignore
    /// 1
    /// 1
    /// ```
    ///
    /// This is because [`Self`] implicitly get copied.
    /// <sup><i>I may or may not have spent a large amount of time trying to get rid of this "soundness hole".</i></sup>
    fn adapt(self) -> DropAdapter<Self> where Self: Sized {
        DropAdapter(self)
    }

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

/// A trait for types which can be dropped, but which may fail to do so.
///
/// This is an impure version of try drop, meaning that it depends on the global try drop strategy.
///
/// # Gotchas
/// Implementing this trait is not enough to make it droppable. In order for the try drop strategy
/// to be run, you need to put your type in a [`DropAdapter`].
///
/// An easier way to make your type droppable is to call [`TryDrop::adapt`] on it.
#[cfg(feature = "global")]
pub trait ImpureTryDrop {
    /// The type of the error that may occur during drop.
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

/// A trait which signifies a try drop strategy which can fail.
pub trait FallibleTryDropStrategy {
    /// The type of the error that may occur when handling a drop error.
    type Error: Into<anyhow::Error>;

    /// Try and handle a drop error.
    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error>;
}

/// A reference to a type which implements [`FallibleTryDropStrategy`]. Used as a workaround for
/// implementing [`FallibleTryDropStrategy`] on references.
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

/// A trait which signifies a try drop strategy which can fail. Can be dynamically dispatched.
pub trait DynFallibleTryDropStrategy {
    /// Try to handle the drop error.
    fn dyn_try_handle_error(&self, error: anyhow::Error) -> anyhow::Result<()>;
}

/// A trait which signifies a try drop strategy which can fail, can be dynamically dispatched, and
/// can be used as the global try drop strategy.
#[cfg(feature = "global")]
#[cfg(not(feature = "downcast-rs"))]
pub trait GlobalDynFallibleTryDropStrategy: ThreadSafe + DynFallibleTryDropStrategy {}

/// A trait which signifies a try drop strategy which can fail, can be dynamically dispatched, and
/// can be used as the global try drop strategy.
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

/// A trait which signifies a try drop strategy. This can never fail. If it can, use
/// [`FallibleTryDropStrategy`] instead.
pub trait TryDropStrategy {
    /// Handle the drop error.
    fn handle_error(&self, error: anyhow::Error);
}

impl<TDS: TryDropStrategy> FallibleTryDropStrategy for TDS {
    type Error = Infallible;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.handle_error(error);
        Ok(())
    }
}

/// A trait which signifies a thread safe type. Can be used in a `static`.
pub trait ThreadSafe: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> ThreadSafe for T {}

/// A type which implements [`Drop`] for types which implements [`TryDrop`].
///
/// # Implementation
/// We call `try_drop`, which is safe because we only do it in [`Drop::drop`]. If it returns an
/// error, we redirect the error to the fallback try drop strategy.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct DropAdapter<TD: PureTryDrop>(pub TD);

impl<TD: PureTryDrop> Drop for DropAdapter<TD> {
    fn drop(&mut self) {
        // SAFETY: we called this function inside a `Drop::drop` context.
        let result = unsafe { self.0.try_drop() };
        if let Err(error) = result {
            let handler = FallbackTryDropStrategyHandler::new(
                FallbackTryDropStrategyRef(self.0.fallback_try_drop_strategy()),
                FallibleTryDropStrategyRef(self.0.try_drop_strategy()),
            );

            handler.handle_error(error.into())
        }
    }
}

impl<TD: PureTryDrop> From<TD> for DropAdapter<TD> {
    fn from(t: TD) -> Self {
        t.adapt()
    }
}
