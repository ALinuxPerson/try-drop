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
pub mod primary;

#[cfg(feature = "global")]
pub mod global;

pub mod prelude;

pub mod drop_strategies;

mod infallible;

use crate::fallback::FallbackTryDropStrategy;
pub use anyhow::Error;
use core::marker::PhantomData;
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

#[cfg(any(feature = "__tests", test))]
pub mod test_utils;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub mod on_uninit;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub mod uninit_error;

mod utils;

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

/// A type which implements [`Drop`] for types which implements [`TryDrop`].
///
/// # Notes
/// This does **not** implement [`TryDrop`] itself, as you could repeat calling the
/// [`TryDrop::try_drop`] method, potentially resulting in undefined behavior. *However*, it does
/// implement it if your type implements the [`RepeatableTryDrop`] trait.
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

impl<RTD: RepeatableTryDrop> PureTryDrop for DropAdapter<RTD> {
    type Error = RTD::Error;
    type FallbackTryDropStrategy = RTD::FallbackTryDropStrategy;
    type TryDropStrategy = RTD::TryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        self.0.fallback_try_drop_strategy()
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        self.0.try_drop_strategy()
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.0.try_drop()
    }
}

// SAFETY: since `RTD` is `RepeatableTryDrop`, we know that it is safe to call `try_drop` multiple
// times.
unsafe impl<RTD: RepeatableTryDrop> RepeatableTryDrop for DropAdapter<RTD> {}

impl<TD: PureTryDrop> From<TD> for DropAdapter<TD> {
    fn from(t: TD) -> Self {
        t.adapt()
    }
}

/// This type is an adapter for types which implement [`TryDrop`] which allow their
/// [`TryDrop::try_drop`] functions to be repeated multiple times.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct RepeatableTryDropAdapter<T: PureTryDrop> {
    /// The inner value.
    #[cfg_attr(feature = "shrinkwraprs", shrinkwrap(main_field))]
    pub inner: T,

    dropped: bool,
    panic_on_double_drop: bool,
}

impl<T: PureTryDrop + Default> Default for RepeatableTryDropAdapter<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: PureTryDrop> RepeatableTryDropAdapter<T> {
    /// Create a new `RepeatableTryDropAdapter` with the given value.
    pub fn new(item: T) -> Self {
        Self {
            inner: item,
            dropped: false,
            panic_on_double_drop: true,
        }
    }
}

#[cfg(not(feature = "shrinkwraprs"))]
impl<T: PureTryDrop> RepeatableTryDropAdapter<T> {
    /// Choose whether or not to panic when the [`RepeatableTryDropAdapter`] is dropped twice or
    /// multiple times.
    pub fn with_panic_on_double_drop(self, panic_on_double_drop: bool) -> Self {
        self.panic_on_double_drop = panic_on_double_drop;
        self
    }

    /// Check whether or not this object has it's destructor called.
    pub fn dropped(&self) -> bool {
        self.dropped
    }

    /// Check whether or not this object will panic when dropped twice or multiple times.
    pub fn panic_on_double_drop(&self) -> bool {
        self.panic_on_double_drop
    }
}

#[cfg(feature = "shrinkwraprs")]
impl<T: PureTryDrop> RepeatableTryDropAdapter<T> {
    /// Choose whether or not to panic when the [`RepeatableTryDropAdapter`] is dropped twice or
    /// multiple times.
    pub fn with_panic_on_double_drop(mut this: Self, panic_on_double_drop: bool) -> Self {
        this.panic_on_double_drop = panic_on_double_drop;
        this
    }

    /// Check whether or not this object has it's destructor called.
    pub fn dropped(this: &Self) -> bool {
        this.dropped
    }

    /// Check whether or not this object will panic when dropped twice or multiple times.
    pub fn panic_on_double_drop(this: &Self) -> bool {
        this.panic_on_double_drop
    }

    /// Take the inner value out of the adapter.
    pub fn take(this: Self) -> T {
        this.inner
    }
}

impl<T: PureTryDrop> PureTryDrop for RepeatableTryDropAdapter<T> {
    type Error = T::Error;
    type FallbackTryDropStrategy = T::FallbackTryDropStrategy;
    type TryDropStrategy = T::TryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        self.inner.fallback_try_drop_strategy()
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        self.inner.try_drop_strategy()
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        if self.dropped && self.panic_on_double_drop {
            panic!("tried to drop object twice, this is an invalid operation")
        } else {
            self.inner.try_drop()?;
            self.dropped = true;
            Ok(())
        }
    }
}

// SAFETY: if we try to drop this twice, either nothing happens or it panics.
unsafe impl<T: PureTryDrop> RepeatableTryDrop for RepeatableTryDropAdapter<T> {}

/// An adapter which makes a type which implements [`TryDropStrategy`], an infallible or try drop
/// strategy which never fails, fallible.
///
/// Note that it's *still* infallible, it's just that it will return an [`Ok`].
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct InfallibleToFallibleTryDropStrategyAdapter<T: TryDropStrategy, E: Into<anyhow::Error>> {
    /// The inner value.
    #[shrinkwrap(main_field)]
    pub inner: T,

    _error: PhantomData<E>,
}

impl<T: TryDropStrategy, E: Into<anyhow::Error>> InfallibleToFallibleTryDropStrategyAdapter<T, E> {
    /// Wrap the `value` in this adapter.
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            _error: PhantomData,
        }
    }

    /// Take the inner value.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> T {
        this.inner
    }
}

impl<T: TryDropStrategy, E: Into<anyhow::Error>> FallibleTryDropStrategy
    for InfallibleToFallibleTryDropStrategyAdapter<T, E>
{
    type Error = E;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        self.inner.handle_error(error);
        Ok(())
    }
}
