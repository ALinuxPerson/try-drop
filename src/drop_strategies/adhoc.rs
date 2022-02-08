#[cfg(feature = "ds-adhoc-mut")]
mod fn_mut {
    use std::marker::PhantomData;
    use anyhow::Error;
    use parking_lot::Mutex;
    use crate::{FallibleTryDropStrategy, TryDropStrategy};

    /// A quick and dirty try drop strategy which uses a function.
    ///
    /// This is more flexible compared to [`AdHocTryDropStrategy`], accepting also [`FnMut`]s
    /// instead of only [`Fn`]s, but the function is guarded by a [`Mutex`], which has more
    /// overhead.
    ///
    /// [`AdHocTryDropStrategy`]: super::AdHocTryDropStrategy
    #[cfg_attr(feature = "derives", derive(Debug, Default))]
    pub struct AdHocMutTryDropStrategy<F: FnMut(crate::Error)>(pub Mutex<F>);

    impl<F: FnMut(crate::Error)> AdHocMutTryDropStrategy<F> {
        /// Create a new ad-hoc try drop strategy.
        pub fn new(f: F) -> Self {
            Self(Mutex::new(f))
        }
    }

    impl<F: FnMut(crate::Error)> TryDropStrategy for AdHocMutTryDropStrategy<F> {
        fn handle_error(&self, error: crate::Error) {
            self.0.lock()(error)
        }
    }

    impl<F: FnMut(crate::Error)> From<F> for AdHocMutTryDropStrategy<F> {
        fn from(f: F) -> Self {
            Self::new(f)
        }
    }

    /// Signifies that this type can be converted into an [`AdHocMutTryDropStrategy`].
    pub trait IntoAdHocMutTryDropStrategy: FnMut(crate::Error) + Sized {
        /// Convert this type into an [`AdHocMutTryDropStrategy`].
        fn into_adhoc_mut_try_drop_strategy(self) -> AdHocMutTryDropStrategy<Self> {
            AdHocMutTryDropStrategy::new(self)
        }
    }

    impl<T: FnMut(crate::Error)> IntoAdHocMutTryDropStrategy for T {}

    /// A quick and dirty try drop strategy which uses a function.
    ///
    /// This is more flexible compared to [`AdHocFallibleTryDropStrategy`], accepting also
    /// [`FnMut`]s instead of only [`Fn`]s, but the function is guarded by a [`Mutex`], which has
    /// more overhead.
    ///
    /// [`AdHocTryDropStrategy`]: super::AdHocFallibleTryDropStrategy
    #[cfg_attr(feature = "derives", derive(Debug, Default))]
    pub struct AdHocMutFallibleTryDropStrategy<F, E>
    where
        F: FnMut(crate::Error) -> Result<(), E>,
        E: Into<anyhow::Error>,
    {
        /// The function to call.
        pub f: Mutex<F>,
        _error: PhantomData<E>,
    }

    impl<F, E> AdHocMutFallibleTryDropStrategy<F, E>
    where
        F: FnMut(crate::Error) -> Result<(), E>,
        E: Into<anyhow::Error>,
    {
        /// Create a new ad-hoc fallible try drop strategy.
        pub fn new(f: F) -> Self {
            Self {
                f: Mutex::new(f),
                _error: PhantomData,
            }
        }
    }

    impl<F, E> FallibleTryDropStrategy for AdHocMutFallibleTryDropStrategy<F, E>
        where
            F: FnMut(crate::Error) -> Result<(), E>,
            E: Into<anyhow::Error>,
    {
        type Error = E;

        fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
            self.f.lock()(error)
        }
    }

    impl<F, E> From<F> for AdHocMutFallibleTryDropStrategy<F, E>
    where
        F: FnMut(crate::Error) -> Result<(), E>,
        E: Into<anyhow::Error>,
    {
        fn from(f: F) -> Self {
            Self::new(f)
        }
    }

    /// Signifies that this type can be converted into an [`AdHocMutFallibleTryDropStrategy`].
    pub trait IntoAdHocMutFallibleTryDropStrategy: FnMut(crate::Error) -> Result<(), Self::Error> + Sized
    {
        /// The error type.
        type Error: Into<anyhow::Error>;

        /// Convert this type into an [`AdHocMutFallibleTryDropStrategy`].
        fn into_adhoc_mut_fallible_try_drop_strategy(self) -> AdHocMutFallibleTryDropStrategy<Self, Self::Error> {
            AdHocMutFallibleTryDropStrategy::new(self)
        }
    }
}

#[cfg(feature = "ds-adhoc-mut")]
pub use fn_mut::*;

use core::marker::PhantomData;
use crate::{FallibleTryDropStrategy, TryDropStrategy};

/// A quick and dirty try drop strategy which uses a function.
#[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default))]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocTryDropStrategy<F: Fn(crate::Error)>(pub F);

impl<F: Fn(crate::Error)> AdHocTryDropStrategy<F> {
    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.0
    }
}

impl<F: Fn(crate::Error)> TryDropStrategy for AdHocTryDropStrategy<F> {
    fn handle_error(&self, error: crate::Error) {
        self.0(error)
    }
}

impl<F: Fn(crate::Error)> From<F> for AdHocTryDropStrategy<F> {
    fn from(f: F) -> Self {
        AdHocTryDropStrategy(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocTryDropStrategy`].
pub trait IntoAdHocTryDropStrategy: Fn(crate::Error) + Sized {
    /// Convert this type into an [`AdHocTryDropStrategy`].
    fn into_adhoc_try_drop_strategy(self) -> AdHocTryDropStrategy<Self> {
        AdHocTryDropStrategy(self)
    }
}

impl<T: Fn(crate::Error)> IntoAdHocTryDropStrategy for T {}

/// A quick and dirty fallible try drop strategy which uses a function.
#[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default))]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocFallibleTryDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    /// The inner function.
    #[cfg_attr(feature = "shrinkwraprs", shrinkwrap(main_field))]
    pub f: F,

    _error: PhantomData<E>,
}

impl<F, E> AdHocFallibleTryDropStrategy<F, E>
    where
        F: Fn(crate::Error) -> Result<(), E>,
        E: Into<anyhow::Error>,
{
    /// Create a new ad-hoc fallible try drop strategy.
    pub fn new(f: F) -> Self {
        Self { f, _error: PhantomData }
    }

    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.f
    }
}

impl<F, E> FallibleTryDropStrategy for AdHocFallibleTryDropStrategy<F, E>
    where
        F: Fn(crate::Error) -> Result<(), E>,
        E: Into<anyhow::Error>,
{
    type Error = E;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        (self.f)(error)
    }
}

impl<F, E> From<F> for AdHocFallibleTryDropStrategy<F, E>
    where
        F: Fn(crate::Error) -> Result<(), E>,
        E: Into<anyhow::Error>,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocFallibleTryDropStrategy`].
pub trait IntoAdHocFallibleTryDropStrategy:
    Fn(crate::Error) -> Result<(), Self::Error>
    + Sized
{
    /// The error type.
    type Error: Into<anyhow::Error>;

    /// Convert this type into an [`AdHocFallibleTryDropStrategy`].
    fn into_adhoc_fallible_try_drop_strategy(self) -> AdHocFallibleTryDropStrategy<Self, Self::Error> {
        AdHocFallibleTryDropStrategy::new(self)
    }
}
