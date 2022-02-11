use crate::prelude::*;
pub use crate::handlers::{
    install_global_handlers_dyn,
    install_global_handlers,
    install_thread_local_handlers_dyn,
    install_thread_local_handlers,
    uninstall_globally,
    uninstall_for_thread
};
use crate::handlers::fallback::global::GlobalFallbackDropStrategy;
use crate::handlers::primary::global::GlobalPrimaryTryDropStrategy;

impl<TD: ImpureTryDrop> PureTryDrop for TD {
    type Error = TD::Error;
    type FallbackTryDropStrategy = GlobalFallbackDropStrategy;
    type TryDropStrategy = GlobalPrimaryTryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &GlobalFallbackDropStrategy::DEFAULT
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &GlobalPrimaryTryDropStrategy::DEFAULT
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        TD::try_drop(self)
    }
}
