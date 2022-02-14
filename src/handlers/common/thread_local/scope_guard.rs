use std::cell::RefCell;
use std::{fmt, format};
use std::thread::LocalKey;
use crate::handlers::common::{Handler, NestedScopeError};

pub trait ScopeGuardDefinition: Handler {
    const DYN: &'static str;
    type Dyn;

    fn locked() -> &'static LocalKey<RefCell<bool>>;
    fn replace_dyn(new: Self::Dyn) -> Option<Self::Dyn>;
    fn install_dyn(strategy: Self::Dyn);
}

pub struct ScopeGuard<D: ScopeGuardDefinition> {
    last_strategy: Option<D::Dyn>,
}

impl<D: ScopeGuardDefinition> ScopeGuard<D> {
    pub fn new(strategy: impl Into<D::Dyn>) -> Self {
        Self::new_dyn(strategy.into())
    }

    pub fn new_dyn(strategy: D::Dyn) -> Self {
        Self::try_new_dyn(strategy).expect("you cannot nest scope guards")
    }

    pub fn try_new(strategy: impl Into<D::Dyn>) -> Result<Self, NestedScopeError> {
        Self::try_new_dyn(strategy.into())
    }

    pub fn try_new_dyn(strategy: D::Dyn) -> Result<Self, NestedScopeError> {
        if D::locked().with(|cell| *cell.borrow()) {
            Err(NestedScopeError(()))
        } else {
            D::locked().with(|cell| *cell.borrow_mut() = true);
            Ok(Self { last_strategy: D::replace_dyn(strategy) })
        }
    }
}

impl<D: ScopeGuardDefinition> fmt::Debug for ScopeGuard<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ScopeGuard")
            .field("last_strategy", &format!("Option<Box<dyn {}>>", D::DYN))
            .finish()
    }
}

impl<D: ScopeGuardDefinition> Drop for ScopeGuard<D> {
    fn drop(&mut self) {
        if let Some(last_strategy) = self.last_strategy.take() {
            D::install_dyn(last_strategy)
        }

        D::locked().with(|cell| *cell.borrow_mut() = false)
    }
}
