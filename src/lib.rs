#![doc = include_str!("../README.md")]
#![allow(drop_bounds)]
#![allow(clippy::declare_interior_mutable_const)]
#![warn(missing_docs)]
#![no_std]

#[cfg(feature = "shrinkwraprs")]
#[macro_use]
extern crate shrinkwraprs;

#[cfg(feature = "std")]
extern crate std;

pub mod prelude;

pub mod drop_strategies;

mod infallible;

pub use anyhow::Error;
use core::sync::atomic::Ordering;
pub use infallible::Infallible;

#[cfg(any(feature = "global", feature = "thread-local"))]
mod global_crate_root;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub use global_crate_root::*;

#[cfg(not(any(feature = "global", feature = "thread-local")))]
pub use self::PureTryDrop as TryDrop;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub use self::ImpureTryDrop as TryDrop;

#[cfg(any(feature = "__tests", test))]
pub mod test_utils;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub mod handlers;

pub mod adapters;

use adapters::DropAdapter;

#[allow(dead_code)]
const LOAD_ORDERING: Ordering = Ordering::Acquire;

#[allow(dead_code)]
const STORE_ORDERING: Ordering = Ordering::Release;

/// A trait for types which can be dropped, but which may fail to do so.
///
/// This is a pure version of try drop, meaning that the drop strategies have to be explicitly
/// specified, which means it does not depend on a global try drop strategy.
///
/// # Gotchas
/// Implementing this trait is not enough to make it droppable. In order for the try drop strategy
/// to be run, you need to put your type in a [`DropAdapter`].
///
/// An easier way to make your type droppable is to call [`PureTryDrop::adapt`] on it.
pub trait PureTryDrop {
    /// The type of the error that may occur during drop.
    type Error: Into<anyhow::Error>;

    /// The type which will be used if the drop strategy fails.
    type FallbackTryDropStrategy: TryDropStrategy;

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
    fn adapt(self) -> DropAdapter<Self>
    where
        Self: Sized,
    {
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
/// An easier way to make your type droppable is to call [`PureTryDrop::adapt`] on it.
#[cfg(any(feature = "global", feature = "thread-local"))]
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
    ///
    /// If the implementing type implements [`RepeatableTryDrop`], however, then this function is
    /// safe to call multiple times. If the `unsafe` seems ugly to you, you can use
    /// [`RepeatableTryDrop::safe_try_drop`].
    unsafe fn try_drop(&mut self) -> Result<(), Self::Error>;
}

/// A trait which signifies a try drop strategy which can fail.
pub trait FallibleTryDropStrategy {
    /// The type of the error that may occur when handling a drop error.
    type Error: Into<anyhow::Error>;

    /// Try and handle a drop error.
    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error>;
}

/// A trait which signifies a try drop strategy which can fail. Can be dynamically dispatched.
pub trait DynFallibleTryDropStrategy {
    /// Try to handle the drop error.
    fn dyn_try_handle_error(&self, error: anyhow::Error) -> anyhow::Result<()>;
}

impl<T: FallibleTryDropStrategy> DynFallibleTryDropStrategy for T {
    fn dyn_try_handle_error(&self, error: anyhow::Error) -> anyhow::Result<()> {
        self.try_handle_error(error).map_err(Into::into)
    }
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

/// A trait which signifies a try drop strategy which can be used in a thread local scenario. Must
/// be dynamically dispatched and must live as long as the program does.
#[cfg(feature = "thread-local")]
pub trait ThreadLocalFallibleTryDropStrategy: DynFallibleTryDropStrategy + 'static {}

#[cfg(feature = "thread-local")]
impl<T: DynFallibleTryDropStrategy + 'static> ThreadLocalFallibleTryDropStrategy for T {}

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

/// A trait which signifies a try drop strategy which can be used as the primary or fallback
/// handler.
#[cfg(feature = "global")]
#[cfg(not(feature = "downcast-rs"))]
pub trait GlobalTryDropStrategy: ThreadSafe + TryDropStrategy {}

/// A trait which signifies a try drop strategy which can be used as the primary or fallback
/// handler. Can be downcast.
#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
pub trait GlobalTryDropStrategy: ThreadSafe + downcast_rs::DowncastSync + TryDropStrategy {}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
downcast_rs::impl_downcast!(sync GlobalTryDropStrategy);

#[cfg(feature = "global")]
impl<T: ThreadSafe + TryDropStrategy> GlobalTryDropStrategy for T {}

/// A trait which signifies an infallible try drop strategy which can be used in a thread local.
#[cfg(feature = "thread-local")]
pub trait ThreadLocalTryDropStrategy: TryDropStrategy + 'static {}

#[cfg(feature = "thread-local")]
impl<T: TryDropStrategy + 'static> ThreadLocalTryDropStrategy for T {}

/// A trait which signifies a thread safe type. Can be used in a `static`.
pub trait ThreadSafe: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> ThreadSafe for T {}

/// Marker trait signifying that the implementing type can repeatedly call its [`TryDrop::try_drop`]
/// method.
///
/// # Safety
/// The implementor must ensure that no undefined behavior will occur when calling
/// [`TryDrop::try_drop`] multiple times.
pub unsafe trait RepeatableTryDrop: PureTryDrop {
    /// Safely try and drop the implementing type. You can call this function multiple times.
    fn safe_try_drop(&mut self) -> Result<(), Self::Error> {
        // SAFETY: This is safe because the implementing type has implemented `RepeatableTryDrop`,
        // which assures us that it is safe to call `try_drop` multiple times.
        unsafe { self.try_drop() }
    }
}
