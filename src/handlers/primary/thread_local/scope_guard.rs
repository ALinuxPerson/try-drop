use super::*;
use crate::handlers::common::{NestedScopeError, Primary};
use crate::DynFallibleTryDropStrategy;
use std::boxed::Box;
use std::thread::LocalKey;
use crate::handlers::common::thread_local::scope_guard::{
    ScopeGuard as GenericScopeGuard,
    ScopeGuardDefinition,
};

thread_local! {
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

impl ScopeGuardDefinition for Primary {
    const DYN: &'static str = "DynFallibleTryDropStrategy";
    type Dyn = Box<dyn DynFallibleTryDropStrategy>;

    fn locked() -> &'static LocalKey<RefCell<bool>> {
        &LOCKED
    }

    fn replace_dyn(new: Self::Dyn) -> Option<Self::Dyn> {
        super::replace_dyn(new)
    }

    fn install_dyn(strategy: Self::Dyn) {
        super::install_dyn(strategy)
    }
}

pub type ScopeGuard = GenericScopeGuard<Primary>;

impl<D: DynFallibleTryDropStrategy + 'static> From<D> for Box<dyn DynFallibleTryDropStrategy> {
    fn from(d: D) -> Self {
        Box::new(d)
    }
}
