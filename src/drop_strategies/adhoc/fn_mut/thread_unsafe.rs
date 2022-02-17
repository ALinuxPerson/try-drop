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

impl<F: FnMut(crate::Error)> IntoThreadUnsafeAdHocMutDropStrategy for F {}

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
pub trait IntoThreadUnsafeAdHocMutFallibleDropStrategy<E: Into<anyhow::Error>>:
    FnMut(crate::Error) -> Result<(), E> + Sized
{
    /// Turn this type into a [`ThreadUnsafeAdHocMutFallibleDropStrategy`].
    fn into_drop_strategy(self) -> ThreadUnsafeAdHocMutFallibleDropStrategy<Self, E> {
        ThreadUnsafeAdHocMutFallibleDropStrategy::new(self)
    }
}

impl<T, E> IntoThreadUnsafeAdHocMutFallibleDropStrategy<E> for T
where
    T: FnMut(crate::Error) -> Result<(), E>,
    E: Into<crate::Error>,
{}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::test_utils::fallible;
    use super::*;

    #[test]
    fn test_thread_unsafe_adhoc_mut_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = ThreadUnsafeAdHocMutDropStrategy::new(move |_| w.set(true));
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get());
    }

    #[test]
    fn test_into_thread_unsafe_adhoc_mut_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = move |_| w.set(true);
        let strategy = IntoThreadUnsafeAdHocMutDropStrategy::into_drop_strategy(strategy);
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get());
    }

    #[test]
    fn test_thread_unsafe_adhoc_mut_fallible_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = ThreadUnsafeAdHocMutFallibleDropStrategy::<_, crate::Error>::new(move |_| {
            w.set(true);
            Ok(())
        });
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get());
    }

    #[test]
    fn test_into_thread_unsafe_adhoc_mut_fallible_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = move |_| {
            w.set(true);
            Ok::<_, crate::Error>(())
        };
        let strategy = IntoThreadUnsafeAdHocMutFallibleDropStrategy::into_drop_strategy(strategy);
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get());
    }
}
