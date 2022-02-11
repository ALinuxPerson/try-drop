use crate::prelude::*;
pub use crate::handlers::fns::*;
use crate::handlers::{DEFAULT_FALLBACK_DROP_STRATEGY, DEFAULT_PRIMARY_DROP_STRATEGY, PrimaryDropStrategy, FallbackDropStrategy};

impl<TD: ImpureTryDrop> PureTryDrop for TD {
    type Error = TD::Error;
    type FallbackTryDropStrategy = FallbackDropStrategy;
    type TryDropStrategy = PrimaryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &DEFAULT_FALLBACK_DROP_STRATEGY
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &DEFAULT_PRIMARY_DROP_STRATEGY
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        TD::try_drop(self)
    }
}
