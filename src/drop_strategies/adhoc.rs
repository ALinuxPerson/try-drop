use core::marker::PhantomData;
use crate::{FallibleTryDropStrategy, TryDropStrategy};

#[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default))]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocTryDropStrategy<F: Fn(crate::Error)>(pub F);

impl<F: Fn(crate::Error)> AdHocTryDropStrategy<F> {
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

#[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default))]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocFallibleTryDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    #[cfg_attr(feature = "shrinkwraprs", shrinkwrap(main_field))]
    pub f: F,

    _error: PhantomData<E>,
}

impl<F, E> AdHocFallibleTryDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.f
    }
}

impl<F, E> AdHocFallibleTryDropStrategy<F, E>
    where
        F: Fn(crate::Error) -> Result<(), E>,
        E: Into<anyhow::Error>,
{
    pub fn new(f: F) -> Self {
        Self { f, _error: PhantomData }
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
