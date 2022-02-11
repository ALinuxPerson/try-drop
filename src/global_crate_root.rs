use crate::prelude::*;
pub use crate::handlers::{
    install_global_handlers_dyn,
    install_global_handlers,
    install_thread_local_handlers_dyn,
    install_thread_local_handlers,
    uninstall_globally,
    uninstall_for_thread
};
use crate::handlers::PrimaryDropStrategy;
use crate::handlers::FallbackDropStrategy;

impl<TD: ImpureTryDrop> PureTryDrop for TD {
    type Error = TD::Error;
    type FallbackTryDropStrategy = FallbackDropStrategy;
    type TryDropStrategy = PrimaryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &FallbackDropStrategy::DEFAULT
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &PrimaryDropStrategy::DEFAULT
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        TD::try_drop(self)
    }
}
