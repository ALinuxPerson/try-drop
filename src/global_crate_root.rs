use crate::handlers::{DEFAULT_FALLBACK_HANDLER, DEFAULT_PRIMARY_HANDLER, FallbackHandler, PrimaryHandler};
pub use crate::handlers::fns::*;
use crate::prelude::*;

impl<TD: ImpureTryDrop> PureTryDrop for TD {
    type Error = TD::Error;
    type FallbackTryDropStrategy = FallbackHandler;
    type TryDropStrategy = PrimaryHandler;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &DEFAULT_FALLBACK_HANDLER
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &DEFAULT_PRIMARY_HANDLER
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        TD::try_drop(self)
    }
}
