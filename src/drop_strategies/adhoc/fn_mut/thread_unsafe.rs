use std::cell::RefCell;
use anyhow::Error;
use crate::{FallibleTryDropStrategy, TryDropStrategy};

pub struct ThreadUnsafeAdHocMutDropStrategy<F: FnMut(crate::Error)>(pub RefCell<F>);

impl<F: FnMut(crate::Error)> ThreadUnsafeAdHocMutDropStrategy<F> {
    pub fn new(f: F) -> Self {
        Self(RefCell::new(f))
    }
}

impl<F: FnMut(crate::Error)> TryDropStrategy for ThreadUnsafeAdHocMutDropStrategy<F> {
    fn handle_error(&self, error: crate::Error) {
        self.0.borrow_mut()(error)
    }
}

pub trait IntoThreadUnsafeAdHocMutDropStrategy: FnMut(crate::Error) + Sized {
    fn into_drop_strategy(self) -> ThreadUnsafeAdHocMutDropStrategy<Self> {
        ThreadUnsafeAdHocMutDropStrategy::new(self)
    }
}

pub struct ThreadUnsafeAdHocMutFallibleDropStrategy<F, E>(pub RefCell<F>)
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<crate::Error>;

impl<F, E> ThreadUnsafeAdHocMutFallibleDropStrategy<F, E>
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<crate::Error>,
{
    pub fn new(f: F) -> Self {
        Self(RefCell::new(f))
    }
}

impl<F, E> FallibleTryDropStrategy for ThreadUnsafeAdHocMutFallibleDropStrategy<F, E>
    where
        F: FnMut(crate::Error) -> Result<(), E>,
        E: Into<crate::Error>
{
    type Error = E;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        self.0.borrow_mut()(error)
    }
}

pub trait IntoThreadUnsafeAdHocMutFallibleDropStrategy: FnMut(crate::Error) -> Result<(), Self::Error> + Sized {
    type Error: Into<crate::Error>;

    fn into_drop_strategy(self) -> ThreadUnsafeAdHocMutFallibleDropStrategy<Self, Self::Error> {
        ThreadUnsafeAdHocMutFallibleDropStrategy::new(self)
    }
}
