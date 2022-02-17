#[cfg(feature = "ds-adhoc-mut")]
mod fn_mut;

#[cfg(feature = "ds-adhoc-mut")]
pub use fn_mut::*;

use crate::{FallibleTryDropStrategy, TryDropStrategy};

/// A quick and dirty drop strategy which uses a function.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocDropStrategy<F: Fn(crate::Error)>(pub F);

impl<F: Fn(crate::Error)> AdHocDropStrategy<F> {
    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.0
    }
}

impl<F: Fn(crate::Error)> TryDropStrategy for AdHocDropStrategy<F> {
    fn handle_error(&self, error: crate::Error) {
        self.0(error)
    }
}

impl<F: Fn(crate::Error)> From<F> for AdHocDropStrategy<F> {
    fn from(f: F) -> Self {
        AdHocDropStrategy(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocDropStrategy`].
pub trait IntoAdHocDropStrategy: Fn(crate::Error) + Sized {
    /// Convert this type into an [`AdHocDropStrategy`].
    fn into_drop_strategy(self) -> AdHocDropStrategy<Self> {
        AdHocDropStrategy(self)
    }
}

impl<T: Fn(crate::Error)> IntoAdHocDropStrategy for T {}

/// A quick and dirty fallible drop strategy which uses a function.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocFallibleDropStrategy<F, E>(pub F)
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>;

impl<F, E> AdHocFallibleDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.0
    }
}

impl<F, E> FallibleTryDropStrategy for AdHocFallibleDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    type Error = E;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        (self.0)(error)
    }
}

impl<F, E> From<F> for AdHocFallibleDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    fn from(f: F) -> Self {
        Self(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocFallibleDropStrategy`].
pub trait IntoAdHocFallibleDropStrategy<E: Into<anyhow::Error>>:
    Fn(crate::Error) -> Result<(), E> + Sized
{
    /// Convert this type into an [`AdHocFallibleDropStrategy`].
    fn into_drop_strategy(self) -> AdHocFallibleDropStrategy<Self, E> {
        AdHocFallibleDropStrategy(self)
    }
}

impl<T, E> IntoAdHocFallibleDropStrategy<E> for T
where
    T: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::test_utils::fallible;
    use super::*;

    #[test]
    fn test_adhoc_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = AdHocDropStrategy(move |_| w.set(true));
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get(), "the strategy should have worked");
    }

    #[test]
    fn test_into_adhoc_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = move |_| w.set(true);
        let strategy = IntoAdHocDropStrategy::into_drop_strategy(strategy);
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get(), "the strategy should have worked");
    }

    #[test]
    fn test_adhoc_fallible_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = AdHocFallibleDropStrategy::<_, crate::Error>(move |_| {
            w.set(true);
            Ok(())
        });
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get(), "the strategy should have worked");
    }

    #[test]
    fn test_into_adhoc_fallible_drop_strategy() {
        let works = Rc::new(Cell::new(false));
        let w = Rc::clone(&works);
        let strategy = move |_| {
            w.set(true);
            Ok::<_, crate::Error>(())
        };
        let strategy = IntoAdHocFallibleDropStrategy::into_drop_strategy(strategy);
        crate::install_thread_local_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.get(), "the strategy should have worked");
    }
}