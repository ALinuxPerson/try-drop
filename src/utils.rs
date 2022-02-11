use core::marker::PhantomData;

#[cfg(feature = "thread-local")]
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct NotSendNotSync(PhantomData<*mut ()>);

impl NotSendNotSync {
    pub const fn new() -> Self {
        NotSendNotSync(PhantomData)
    }
}
