pub use crate::double::global::GlobalDoubleDropStrategyHandler;
pub use crate::global::GlobalDropStrategyHandler;
use crate::prelude::*;
use std::boxed::Box;

impl<TD: TryDrop> SpecificTryDrop for TD {
    type Error = TD::Error;
    type DoubleDropStrategy = GlobalDoubleDropStrategyHandler;
    type DropStrategy = GlobalDropStrategyHandler;

    fn double_drop_strategy(&self) -> &Self::DoubleDropStrategy {
        &GlobalDoubleDropStrategyHandler
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
    double_drop_strategy: impl GlobalDoubleDropStrategy,
) {
    install_dyn(Box::new(drop_strategy), Box::new(double_drop_strategy))
}

pub fn install_dyn(
    drop_strategy: Box<dyn GlobalDynFallibleTryDropStrategy>,
    double_drop_strategy: Box<dyn GlobalDoubleDropStrategy>,
) {
    crate::global::install_dyn(drop_strategy);
    crate::double::global::install_dyn(double_drop_strategy);
}
