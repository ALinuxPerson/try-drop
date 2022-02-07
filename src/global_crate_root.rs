pub use crate::fallback::global::GlobalFallbackDropStrategyHandler;
pub use crate::global::GlobalDropStrategyHandler;
use crate::prelude::*;
use std::boxed::Box;

impl<TD: ImpureTryDrop> PureTryDrop for TD {
    type Error = TD::Error;
    type FallbackDropStrategy = GlobalFallbackDropStrategyHandler;
    type DropStrategy = GlobalDropStrategyHandler;

    fn fallback_drop_strategy(&self) -> &Self::FallbackDropStrategy {
        &GlobalFallbackDropStrategyHandler
    }

    fn drop_strategy(&self) -> &Self::DropStrategy {
        &GlobalDropStrategyHandler
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        TD::try_drop(self)
    }
}

pub fn install(
    drop_strategy: impl GlobalDynFallibleTryDropStrategy,
    fallback_drop_strategy: impl GlobalFallbackDropStrategy,
) {
    install_dyn(Box::new(drop_strategy), Box::new(fallback_drop_strategy))
}

pub fn install_dyn(
    drop_strategy: Box<dyn GlobalDynFallibleTryDropStrategy>,
    fallback_drop_strategy: Box<dyn GlobalFallbackDropStrategy>,
) {
    crate::global::install_dyn(drop_strategy);
    crate::fallback::global::install_dyn(fallback_drop_strategy);
}
