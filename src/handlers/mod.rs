use std::boxed::Box;
use crate::{DynFallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy, GlobalTryDropStrategy, TryDropStrategy};

pub mod primary;
pub mod fallback;
mod common;
mod shim;

pub fn install_global_handlers(
    primary: impl GlobalDynFallibleTryDropStrategy,
    fallback: impl GlobalTryDropStrategy,
) {
    install_global_handlers_dyn(Box::new(primary), Box::new(fallback))
}

pub fn install_global_handlers_dyn(
    primary: Box<dyn GlobalDynFallibleTryDropStrategy>,
    fallback: Box<dyn GlobalTryDropStrategy>,
) {
    primary::global::install_dyn(primary);
    fallback::global::install_dyn(fallback);
}

pub fn uninstall_globally() {
    primary::global::uninstall();
    fallback::global::uninstall();
}

pub fn install_thread_local_handlers(
    primary: impl DynFallibleTryDropStrategy + 'static,
    fallback: impl TryDropStrategy + 'static
) {
    install_thread_local_handlers_dyn(Box::new(primary), Box::new(fallback))
}

pub fn install_thread_local_handlers_dyn(
    primary: Box<dyn DynFallibleTryDropStrategy>,
    fallback: Box<dyn TryDropStrategy>,
) {
    primary::thread_local::install_dyn(primary);
    fallback::thread_local::install_dyn(fallback);
}

pub fn uninstall_for_thread() {
    primary::thread_local::uninstall();
    fallback::thread_local::uninstall();
}
