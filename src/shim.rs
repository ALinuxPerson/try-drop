use std::marker::PhantomData;
use crate::{FallibleTryDropStrategy, GlobalTryDropStrategyHandler};
use crate::thread_local::ThreadLocalDropStrategy;

mod private {
    pub trait Sealed {}
}

pub trait Precedence: private::Sealed {
    type DropStrategy: FallibleTryDropStrategy;
}

pub enum Global {}

impl Precedence for Global {
    type DropStrategy = ThreadLocalDropStrategy;
}
impl private::Sealed for Global {}

pub enum ThreadLocal {}

impl Precedence for ThreadLocal {
    type DropStrategy = GlobalTryDropStrategyHandler;
}
impl private::Sealed for ThreadLocal {}

/// This drop strategy is a shim which merges the thread local drop strategy and global drop
/// strategy together.
///
/// The thread local drop strategy takes precedence over the global drop strategy. If it's not
/// available, the global drop strategy is used.
pub struct ShimDropStrategy<P: Precedence>(PhantomData<P>);
