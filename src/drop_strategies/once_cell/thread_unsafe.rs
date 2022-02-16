use super::{AlreadyOccupiedError, Error, Ignore, Mode};
use crate::{FallibleTryDropStrategy, TryDropStrategy};
use once_cell::unsync::OnceCell;
use std::marker::PhantomData;
use std::rc::Rc;

/// A drop strategy which sets an error value once.
///
/// For more information see it's thread safe counterpart.
#[cfg_attr(feature = "derives", derive(Debug, Clone, Default))]
pub struct ThreadUnsafeOnceCellDropStrategy<M: Mode> {
    /// The inner error value.
    pub inner: Rc<OnceCell<anyhow::Error>>,
    _marker: PhantomData<M>,
}

impl ThreadUnsafeOnceCellDropStrategy<Ignore> {
    /// Create a new once cell drop strategy which will ignore if there is already an error value in
    /// its cell.
    pub fn ignore(value: Rc<OnceCell<anyhow::Error>>) -> Self {
        Self::new(value)
    }
}

impl ThreadUnsafeOnceCellDropStrategy<Error> {
    /// Create a new once cell drop strategy which will error if there is already an error value in
    /// its cell.
    pub fn error(value: Rc<OnceCell<anyhow::Error>>) -> Self {
        Self::new(value)
    }
}

impl<M: Mode> ThreadUnsafeOnceCellDropStrategy<M> {
    /// Create a new once cell drop strategy which sets an error value once.
    pub fn new(value: Rc<OnceCell<anyhow::Error>>) -> Self {
        Self {
            inner: value,
            _marker: PhantomData,
        }
    }
}

impl TryDropStrategy for ThreadUnsafeOnceCellDropStrategy<Ignore> {
    fn handle_error(&self, error: anyhow::Error) {
        let _ = self.inner.set(error);
    }
}

impl FallibleTryDropStrategy for ThreadUnsafeOnceCellDropStrategy<Error> {
    type Error = AlreadyOccupiedError;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.inner.set(error).map_err(AlreadyOccupiedError)
    }
}
