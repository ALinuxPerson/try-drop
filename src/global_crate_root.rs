pub use crate::fallback::global::GlobalFallbackTryDropStrategyHandler;
pub use crate::global::GlobalFallibleTryDropStrategy;
use crate::prelude::*;
use std::boxed::Box;

impl<TD: ImpureTryDrop> PureTryDrop for TD {
    type Error = TD::Error;
    type FallbackTryDropStrategy = GlobalFallbackTryDropStrategyHandler;
    type TryDropStrategy = GlobalFallibleTryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &GlobalFallbackTryDropStrategyHandler
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &GlobalFallibleTryDropStrategy::PANIC_ON_UNINIT
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        TD::try_drop(self)
    }
}

/// Install a drop strategy and fallback drop strategy globally.
pub fn install(
    drop_strategy: impl GlobalDynFallibleTryDropStrategy,
    fallback_drop_strategy: impl GlobalFallbackTryDropStrategy,
) {
    install_dyn(Box::new(drop_strategy), Box::new(fallback_drop_strategy))
}

/// Install a drop strategy and fallback drop strategy globally. They both need to be a dynamic
/// trait object.
pub fn install_dyn(
    drop_strategy: Box<dyn GlobalDynFallibleTryDropStrategy>,
    fallback_drop_strategy: Box<dyn GlobalFallbackTryDropStrategy>,
) {
    crate::global::install_dyn(drop_strategy);
    crate::fallback::global::install_dyn(fallback_drop_strategy);
}
