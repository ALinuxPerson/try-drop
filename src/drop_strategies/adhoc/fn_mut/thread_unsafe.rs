use crate::{FallibleTryDropStrategy, TryDropStrategy};
use anyhow::Error;
use std::cell::RefCell;

/// A drop strategy which uses a function to handle errors. This is less flexible than its thread
/// safe counterpart however there is less overhead.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)
)]
pub struct ThreadUnsafeAdHocMutDropStrategy<F: FnMut(crate::Error)>(pub RefCell<F>);

impl<F: FnMut(crate::Error)> ThreadUnsafeAdHocMutDropStrategy<F> {
    /// Create a new thread unsafe adhoc mut drop strategy.
    pub fn new(f: F) -> Self {
        Self(RefCell::new(f))
    }
}

impl<F: FnMut(crate::Error)> TryDropStrategy for ThreadUnsafeAdHocMutDropStrategy<F> {
    fn handle_error(&self, error: crate::Error) {
        self.0.borrow_mut()(error)
    }
}

/// Turn this type into a [`ThreadUnsafeAdHocMutDropStrategy`].
pub trait IntoThreadUnsafeAdHocMutDropStrategy: FnMut(crate::Error) + Sized {
    /// Turn this type into a [`ThreadUnsafeAdHocMutDropStrategy`].
    fn into_drop_strategy(self) -> ThreadUnsafeAdHocMutDropStrategy<Self> {
        ThreadUnsafeAdHocMutDropStrategy::new(self)
    }
}

/// A fallible drop strategy which uses a function to handle errors. This is less flexible than its
/// thread safe counterpart however there is less overhead.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)
)]
pub struct ThreadUnsafeAdHocMutFallibleDropStrategy<F, E>(pub RefCell<F>)
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<crate::Error>;

impl<F, E> ThreadUnsafeAdHocMutFallibleDropStrategy<F, E>
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<crate::Error>,
{
    /// Create a new thread unsafe adhoc mut fallible drop strategy.
    pub fn new(f: F) -> Self {
        Self(RefCell::new(f))
    }
}

impl<F, E> FallibleTryDropStrategy for ThreadUnsafeAdHocMutFallibleDropStrategy<F, E>
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<crate::Error>,
{
    type Error = E;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        self.0.borrow_mut()(error)
    }
}

/// Turn this type into a [`ThreadUnsafeAdHocMutFallibleDropStrategy`].
pub trait IntoThreadUnsafeAdHocMutFallibleDropStrategy:
    FnMut(crate::Error) -> Result<(), Self::Error> + Sized
{
    /// The error type which will be used.
    type Error: Into<crate::Error>;

    /// Turn this type into a [`ThreadUnsafeAdHocMutFallibleDropStrategy`].
    fn into_drop_strategy(self) -> ThreadUnsafeAdHocMutFallibleDropStrategy<Self, Self::Error> {
        ThreadUnsafeAdHocMutFallibleDropStrategy::new(self)
    }
}
