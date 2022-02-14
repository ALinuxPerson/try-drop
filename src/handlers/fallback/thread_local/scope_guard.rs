use super::*;
use crate::handlers::common::Fallback;
use crate::TryDropStrategy;
use std::boxed::Box;
use std::thread::LocalKey;
use crate::handlers::common::thread_local::scope_guard::{
    ScopeGuardDefinition,
    ScopeGuard as GenericScopeGuard,
};

thread_local! {
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

impl ScopeGuardDefinition for Fallback {
    const DYN: &'static str = "TryDropStrategy";
    type Dyn = Box<dyn TryDropStrategy>;

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

pub type ScopeGuard = GenericScopeGuard<Fallback>;

impl<T: TryDropStrategy + 'static> From<T> for Box<dyn TryDropStrategy> {
    fn from(strategy: T) -> Self {
        Box::new(strategy)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::drop_strategies::{AdHocFallibleTryDropStrategy, IntoAdHocTryDropStrategy, NoOpDropStrategy, PanicDropStrategy};
    use crate::PureTryDrop;
    use crate::test_utils::{ErrorsOnDrop, Fallible};
    use super::*;

    #[test]
    fn test_scope_guard() {
        crate::install_thread_local_handlers(
            AdHocFallibleTryDropStrategy(Err),
            PanicDropStrategy::DEFAULT,
        );
        let scope_guard_executed = Rc::new(RefCell::new(false));

        {
            let sge = Rc::clone(&scope_guard_executed);
            let _guard = ScopeGuard::new((move |_| *sge.borrow_mut() = true).into_adhoc_try_drop_strategy());
            let errors = ErrorsOnDrop::<Fallible, _>::not_given().adapt();
            drop(errors);
        }

        assert!(*scope_guard_executed.borrow(), "scope guard was not executed");

        crate::uninstall_for_thread()
    }

    #[test]
    fn test_scope_guard_errors_if_scope_is_nested() {
        {
            let _guard = ScopeGuard::new(NoOpDropStrategy);
            {
                ScopeGuard::try_new(NoOpDropStrategy).expect_err("scope guard was did not error when nested");
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_scope_guard_panics_if_scope_is_nested() {
        {
            let _guard = ScopeGuard::new(NoOpDropStrategy);
            {
                let _guard = ScopeGuard::new(NoOpDropStrategy);
            }
        }
    }
}