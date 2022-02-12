use super::*;
use crate::handlers::common::NestedScopeError;
use crate::TryDropStrategy;
use std::boxed::Box;
use std::fmt;

thread_local! {
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

/// This installs a thread local fallback drop strategy for the current scope.
pub struct ScopeGuard {
    last_strategy: Option<Box<dyn TryDropStrategy>>,
}

impl ScopeGuard {
    /// Create a new scope guard.
    ///
    /// # Panics
    /// This panics if the scope guard was nested.
    pub fn new(strategy: impl TryDropStrategy + 'static) -> Self {
        Self::new_dyn(Box::new(strategy))
    }

    /// Create a new scope guard. Must be a dynamic trait object.
    ///
    /// # Panics
    /// This panics if the scope guard was nested
    pub fn new_dyn(strategy: Box<dyn TryDropStrategy>) -> Self {
        Self::try_new_dyn(strategy).expect("you cannot nest scope guards")
    }

    /// Try and create a new scope guard.
    ///
    /// # Errors
    /// This returns an error if the scope guard was nested.
    pub fn try_new(strategy: impl TryDropStrategy + 'static) -> Result<Self, NestedScopeError> {
        Self::try_new_dyn(Box::new(strategy))
    }

    /// Try and create a new scope guard. Must be a dynamic trait object.
    ///
    /// # Errors
    /// This returns an error if the scope guard was nested.
    pub fn try_new_dyn(strategy: Box<dyn TryDropStrategy>) -> Result<Self, NestedScopeError> {
        if LOCKED.with(|cell| *cell.borrow()) {
            Err(NestedScopeError(()))
        } else {
            LOCKED.with(|cell| *cell.borrow_mut() = true);
            Ok(Self {
                last_strategy: replace_dyn(strategy),
            })
        }
    }
}

impl fmt::Debug for ScopeGuard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ScopeGuard")
            .field("last_strategy", &"Option<Box<dyn TryDropStrategy>>")
            .finish()
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        if let Some(last_strategy) = self.last_strategy.take() {
            install_dyn(last_strategy)
        }

        LOCKED.with(|cell| *cell.borrow_mut() = false)
    }
}
