use crate::handlers::common::thread_local::{ThreadLocal, ThreadLocalDefinition};
use crate::handlers::common::NestedScopeError;
use std::{fmt, format};

pub struct ScopeGuard<D: ThreadLocalDefinition> {
    last_strategy: Option<D::ThreadLocal>,
}

impl<D: ThreadLocalDefinition> ScopeGuard<D> {
    pub fn new(strategy: impl Into<D::ThreadLocal>) -> Self {
        Self::new_dyn(strategy.into())
    }

    pub fn new_dyn(strategy: D::ThreadLocal) -> Self {
        Self::try_new_dyn(strategy).expect("you cannot nest scope guards")
    }

    pub fn try_new(strategy: impl Into<D::ThreadLocal>) -> Result<Self, NestedScopeError> {
        Self::try_new_dyn(strategy.into())
    }

    pub fn try_new_dyn(strategy: D::ThreadLocal) -> Result<Self, NestedScopeError> {
        if D::locked().with(|cell| cell.get()) {
            Err(NestedScopeError(()))
        } else {
            D::locked().with(|cell| cell.set(true));
            Ok(Self {
                last_strategy: ThreadLocal::<D>::replace_dyn(strategy),
            })
        }
    }
}

impl<D: ThreadLocalDefinition> fmt::Debug for ScopeGuard<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ScopeGuard")
            .field("last_strategy", &format!("Option<Box<dyn {}>>", D::DYN))
            .finish()
    }
}

impl<D: ThreadLocalDefinition> Drop for ScopeGuard<D> {
    fn drop(&mut self) {
        if let Some(last_strategy) = self.last_strategy.take() {
            ThreadLocal::<D>::install_dyn(last_strategy)
        }

        D::locked().with(|cell| cell.set(false))
    }
}
