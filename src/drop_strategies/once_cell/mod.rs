//! Types and utilities for the once cell try drop strategy.
mod thread_unsafe;
pub use thread_unsafe::ThreadUnsafeOnceCellDropStrategy;
mod private {
    pub trait Sealed {}
}

use crate::{FallibleTryDropStrategy, TryDropStrategy};
pub use once_cell::sync::OnceCell;
use std::error::Error as StdError;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
pub use thread_unsafe::*;

/// Ignore the occupied error value and continue.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub enum Ignore {}

impl Mode for Ignore {}
impl private::Sealed for Ignore {}

/// Return an error with the underlying error value if the cell is occupied.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub enum Error {}

impl Mode for Error {}
impl private::Sealed for Error {}

/// How to handle cases where the error value is already occupied.
pub trait Mode: private::Sealed {}

/// An error which is returned if the cell is already occupied.
#[derive(Debug)]
pub struct AlreadyOccupiedError(pub anyhow::Error);

impl StdError for AlreadyOccupiedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.0.as_ref())
    }
}

impl fmt::Display for AlreadyOccupiedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an already existing error was occupied in this cell")
    }
}

/// A try drop strategy which sets an error value once.
//
/// This try drop strategy can only handle single errors. If you want to handle multiple errors,
/// see the [`BroadcastDropStrategy`].
///
/// The most common use case of this is when you want to get an error from inside a function which
/// calls [`TryDrop`](crate::TryDrop).
///
/// # Examples
/// ```ignore
/// use once_cell::sync::OnceCell;
/// use std::sync::Arc;
/// use try_drop::drop_strategies::once_cell::Ignore;
/// use try_drop::drop_strategies::OnceCellTryDropStrategy;
///
/// fn calls_try_drop(may_fail: ThisDropMayFail) {
///     // do something with `may_fail`
/// }
///
/// let error = Arc::new(OnceCell::new());
/// let strategy = OnceCellTryDropStrategy::<Ignore>::new(Arc::clone(&error));
/// let may_fail = ThisDropMayFail::new_with_strategy(strategy);
///
/// calls_try_drop(may_fail);
///
/// if let Some(error) = Arc::try_unwrap(error)
///     .expect("arc still referenced by `calls_try_drop`")
///     .take()
/// {
///     println!("an error occurred in `calls_try_drop`: {error}")
/// }
/// ```
///
/// [`BroadcastDropStrategy`]: crate::drop_strategies::BroadcastDropStrategy
#[cfg_attr(feature = "derives", derive(Debug, Clone, Default))]
pub struct OnceCellDropStrategy<M: Mode> {
    /// The inner error value.
    pub inner: Arc<OnceCell<anyhow::Error>>,
    _mode: PhantomData<M>,
}

impl OnceCellDropStrategy<Ignore> {
    /// Create a new once cell drop strategy which will ignore if there is already an error value in
    /// its cell.
    pub fn ignore(item: Arc<OnceCell<anyhow::Error>>) -> Self {
        Self::new(item)
    }
}

impl OnceCellDropStrategy<Error> {
    /// Create a new once cell drop strategy which will error if there is already an error value in
    /// its cell.
    pub fn error(item: Arc<OnceCell<anyhow::Error>>) -> Self {
        Self::new(item)
    }
}

impl<M: Mode> OnceCellDropStrategy<M> {
    /// Creates a new drop strategy which sets an error value once.
    pub fn new(item: Arc<OnceCell<anyhow::Error>>) -> Self {
        Self {
            inner: item,
            _mode: PhantomData,
        }
    }
}

impl TryDropStrategy for OnceCellDropStrategy<Ignore> {
    fn handle_error(&self, error: anyhow::Error) {
        let _ = self.inner.set(error);
    }
}

impl FallibleTryDropStrategy for OnceCellDropStrategy<Error> {
    type Error = AlreadyOccupiedError;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.inner.set(error).map_err(AlreadyOccupiedError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::test_utils::{ErrorsOnDrop, Fallible};
    use crate::PureTryDrop;

    fn test<M: Mode>()
    where
        OnceCellDropStrategy<M>: FallibleTryDropStrategy,
    {
        let item = Arc::new(OnceCell::new());
        let strategy = OnceCellDropStrategy::<M>::new(Arc::clone(&item));
        let errors =
            ErrorsOnDrop::<Fallible, _>::given(strategy, PanicDropStrategy::DEFAULT).adapt();
        drop(errors);
        Arc::try_unwrap(item)
            .expect("item still referenced by `errors`")
            .into_inner()
            .expect("no error occupied in `OnceCellDropStrategy`");
    }

    #[test]
    fn test_ignore() {
        test::<Ignore>();
    }

    #[test]
    fn test_error() {
        test::<Error>();
    }
}
