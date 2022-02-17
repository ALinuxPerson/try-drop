mod thread_unsafe;

use crate::{FallibleTryDropStrategy, TryDropStrategy};
use anyhow::Error;
use parking_lot::Mutex;
use std::marker::PhantomData;
pub use thread_unsafe::*;

/// A quick and dirty drop strategy which uses a function.
///
/// This is more flexible compared to [`AdHocDropStrategy`], accepting also [`FnMut`]s instead of
/// only [`Fn`]s, but the function is guarded by a [`Mutex`], which has more overhead.
///
/// [`AdHocDropStrategy`]: super::AdHocDropStrategy
#[cfg_attr(feature = "derives", derive(Debug, Default))]
pub struct AdHocMutDropStrategy<F: FnMut(crate::Error)>(pub Mutex<F>);

impl<F: FnMut(crate::Error)> AdHocMutDropStrategy<F> {
    /// Create a new ad-hoc try drop strategy.
    pub fn new(f: F) -> Self {
        Self(Mutex::new(f))
    }
}

impl<F: FnMut(crate::Error)> TryDropStrategy for AdHocMutDropStrategy<F> {
    fn handle_error(&self, error: crate::Error) {
        self.0.lock()(error)
    }
}

impl<F: FnMut(crate::Error)> From<F> for AdHocMutDropStrategy<F> {
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocMutDropStrategy`].
pub trait IntoAdHocMutDropStrategy: FnMut(crate::Error) + Sized {
    /// Convert this type into an [`AdHocMutDropStrategy`].
    fn into_drop_strategy(self) -> AdHocMutDropStrategy<Self> {
        AdHocMutDropStrategy::new(self)
    }
}

impl<T: FnMut(crate::Error)> IntoAdHocMutDropStrategy for T {}

/// A quick and dirty try drop strategy which uses a function.
///
/// This is more flexible compared to [`AdHocFallibleDropStrategy`], accepting also [`FnMut`]s
/// instead of only [`Fn`]s, but the function is guarded by a [`Mutex`], which has more overhead.
///
/// [`AdHocFallibleDropStrategy`]: super::AdHocFallibleDropStrategy
#[cfg_attr(feature = "derives", derive(Debug, Default))]
pub struct AdHocMutFallibleDropStrategy<F, E>
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    /// The function to call.
    pub f: Mutex<F>,
    _error: PhantomData<E>,
}

impl<F, E> AdHocMutFallibleDropStrategy<F, E>
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    /// Create a new ad-hoc fallible drop strategy.
    pub fn new(f: F) -> Self {
        Self {
            f: Mutex::new(f),
            _error: PhantomData,
        }
    }
}

impl<F, E> FallibleTryDropStrategy for AdHocMutFallibleDropStrategy<F, E>
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    type Error = E;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        self.f.lock()(error)
    }
}

impl<F, E> From<F> for AdHocMutFallibleDropStrategy<F, E>
where
    F: FnMut(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocMutFallibleDropStrategy`].
pub trait IntoAdHocMutFallibleDropStrategy<E: Into<anyhow::Error>>:
    FnMut(crate::Error) -> Result<(), E> + Sized
{
    /// Convert this type into an [`AdHocMutFallibleDropStrategy`].
    fn into_drop_strategy(self) -> AdHocMutFallibleDropStrategy<Self, E> {
        AdHocMutFallibleDropStrategy::new(self)
    }
}

impl<T, E> IntoAdHocMutFallibleDropStrategy<E> for T
where
    T: FnMut(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::{LOAD_ORDERING, STORE_ORDERING};
    use crate::test_utils::fallible;
    use super::*;

    // we need this lock otherwise the test results will be inconsistent
    static LOCK: Mutex<()> = parking_lot::const_mutex(());

    #[test]
    fn test_adhoc_mut_drop_strategy() {
        let _lock = LOCK.lock();
        let works = Arc::new(AtomicBool::new(false));
        let w = Arc::clone(&works);
        let strategy = AdHocMutDropStrategy::new(move |_| w.store(true, STORE_ORDERING));
        crate::install_global_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.load(LOAD_ORDERING));
    }

    #[test]
    fn test_into_adhoc_mut_drop_strategy() {
        let _lock = LOCK.lock();
        let works = Arc::new(AtomicBool::new(false));
        let w = Arc::clone(&works);
        let strategy = move |_| w.store(true, STORE_ORDERING);
        let strategy = strategy.into_drop_strategy();
        crate::install_global_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.load(LOAD_ORDERING));
    }

    #[test]
    fn test_adhoc_mut_fallible_drop_strategy() {
        let _lock = LOCK.lock();
        let works = Arc::new(AtomicBool::new(false));
        let w = Arc::clone(&works);
        let strategy = AdHocMutFallibleDropStrategy::<_, crate::Error>::new(move |_| {
            w.store(true, STORE_ORDERING);
            Ok(())
        });
        crate::install_global_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.load(LOAD_ORDERING));
    }

    #[test]
    fn test_into_adhoc_mut_fallible_drop_strategy() {
        let _lock = LOCK.lock();
        let works = Arc::new(AtomicBool::new(false));
        let w = Arc::clone(&works);
        let strategy = move |_| {
            w.store(true, STORE_ORDERING);
            Ok::<_, crate::Error>(())
        };
        let strategy = strategy.into_drop_strategy();
        crate::install_global_handlers(strategy, PanicDropStrategy::DEFAULT);
        drop(fallible());
        assert!(works.load(LOAD_ORDERING));
    }
}