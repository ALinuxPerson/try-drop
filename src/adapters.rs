use core::marker::PhantomData;
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, PureTryDrop, RepeatableTryDrop, TryDropStrategy};

/// An adapter which makes a type which implements [`TryDropStrategy`], an infallible or try drop
/// strategy which never fails, fallible.
///
/// Note that it's *still* infallible, it's just that it will return an [`Ok`].
#[cfg_attr(
feature = "derives",
derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct InfallibleToFallibleTryDropStrategyAdapter<T: TryDropStrategy, E: Into<anyhow::Error>> {
    /// The inner value.
    #[shrinkwrap(main_field)]
    pub inner: T,

    _error: PhantomData<E>,
}

impl<T: TryDropStrategy, E: Into<anyhow::Error>> InfallibleToFallibleTryDropStrategyAdapter<T, E> {
    /// Wrap the `value` in this adapter.
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            _error: PhantomData,
        }
    }

    /// Take the inner value.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> T {
        this.inner
    }
}

impl<T: TryDropStrategy, E: Into<anyhow::Error>> FallibleTryDropStrategy
for InfallibleToFallibleTryDropStrategyAdapter<T, E>
{
    type Error = E;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.inner.handle_error(error);
        Ok(())
    }
}


/// This type is an adapter for types which implement [`TryDrop`] which allow their
/// [`TryDrop::try_drop`] functions to be repeated multiple times.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct RepeatableTryDropAdapter<T: PureTryDrop> {
    /// The inner value.
    #[cfg_attr(feature = "shrinkwraprs", shrinkwrap(main_field))]
    pub inner: T,

    dropped: bool,
    panic_on_double_drop: bool,
}

impl<T: PureTryDrop + Default> Default for RepeatableTryDropAdapter<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: PureTryDrop> RepeatableTryDropAdapter<T> {
    /// Create a new `RepeatableTryDropAdapter` with the given value.
    pub fn new(item: T) -> Self {
        Self {
            inner: item,
            dropped: false,
            panic_on_double_drop: true,
        }
    }
}

#[cfg(not(feature = "shrinkwraprs"))]
impl<T: PureTryDrop> RepeatableTryDropAdapter<T> {
    /// Choose whether or not to panic when the [`RepeatableTryDropAdapter`] is dropped twice or
    /// multiple times.
    pub fn with_panic_on_double_drop(self, panic_on_double_drop: bool) -> Self {
        self.panic_on_double_drop = panic_on_double_drop;
        self
    }

    /// Check whether or not this object has it's destructor called.
    pub fn dropped(&self) -> bool {
        self.dropped
    }

    /// Check whether or not this object will panic when dropped twice or multiple times.
    pub fn panic_on_double_drop(&self) -> bool {
        self.panic_on_double_drop
    }
}

#[cfg(feature = "shrinkwraprs")]
impl<T: PureTryDrop> RepeatableTryDropAdapter<T> {
    /// Choose whether or not to panic when the [`RepeatableTryDropAdapter`] is dropped twice or
    /// multiple times.
    pub fn with_panic_on_double_drop(mut this: Self, panic_on_double_drop: bool) -> Self {
        this.panic_on_double_drop = panic_on_double_drop;
        this
    }

    /// Check whether or not this object has it's destructor called.
    pub fn dropped(this: &Self) -> bool {
        this.dropped
    }

    /// Check whether or not this object will panic when dropped twice or multiple times.
    pub fn panic_on_double_drop(this: &Self) -> bool {
        this.panic_on_double_drop
    }

    /// Take the inner value out of the adapter.
    pub fn take(this: Self) -> T {
        this.inner
    }
}

impl<T: PureTryDrop> PureTryDrop for RepeatableTryDropAdapter<T> {
    type Error = T::Error;
    type FallbackTryDropStrategy = T::FallbackTryDropStrategy;
    type TryDropStrategy = T::TryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        self.inner.fallback_try_drop_strategy()
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        self.inner.try_drop_strategy()
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        if self.dropped && self.panic_on_double_drop {
            panic!("tried to drop object twice, this is an invalid operation")
        } else {
            self.inner.try_drop()?;
            self.dropped = true;
            Ok(())
        }
    }
}

// SAFETY: if we try to drop this twice, either nothing happens or it panics.
unsafe impl<T: PureTryDrop> RepeatableTryDrop for RepeatableTryDropAdapter<T> {}

/// A type which implements [`Drop`] for types which implements [`TryDrop`].
///
/// # Notes
/// This does **not** implement [`TryDrop`] itself, as you could repeat calling the
/// [`TryDrop::try_drop`] method, potentially resulting in undefined behavior. *However*, it does
/// implement it if your type implements the [`RepeatableTryDrop`] trait.
///
/// # Implementation
/// We call `try_drop`, which is safe because we only do it in [`Drop::drop`]. If it returns an
/// error, we redirect the error to the fallback try drop strategy.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct DropAdapter<TD: PureTryDrop>(pub TD);

impl<TD: PureTryDrop> Drop for DropAdapter<TD> {
    fn drop(&mut self) {
        // SAFETY: we called this function inside a `Drop::drop` context.
        let result = unsafe { self.0.try_drop() };
        if let Err(error) = result {
            let handler = FallbackTryDropStrategyHandler::new(
                FallbackTryDropStrategyRef(self.0.fallback_try_drop_strategy()),
                FallibleTryDropStrategyRef(self.0.try_drop_strategy()),
            );

            handler.handle_error(error.into())
        }
    }
}

impl<RTD: RepeatableTryDrop> PureTryDrop for DropAdapter<RTD> {
    type Error = RTD::Error;
    type FallbackTryDropStrategy = RTD::FallbackTryDropStrategy;
    type TryDropStrategy = RTD::TryDropStrategy;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        self.0.fallback_try_drop_strategy()
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        self.0.try_drop_strategy()
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.0.try_drop()
    }
}

// SAFETY: since `RTD` is `RepeatableTryDrop`, we know that it is safe to call `try_drop` multiple
// times.
unsafe impl<RTD: RepeatableTryDrop> RepeatableTryDrop for DropAdapter<RTD> {}

impl<TD: PureTryDrop> From<TD> for DropAdapter<TD> {
    fn from(t: TD) -> Self {
        t.adapt()
    }
}

/// A reference to a type which implements [`FallibleTryDropStrategy`]. Used as a workaround for
/// implementing [`FallibleTryDropStrategy`] on references.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct FallibleTryDropStrategyRef<'a, T: FallibleTryDropStrategy>(pub &'a T);

impl<'a, T: FallibleTryDropStrategy> FallibleTryDropStrategy for FallibleTryDropStrategyRef<'a, T> {
    type Error = T::Error;

    fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
        self.0.try_handle_error(error)
    }
}

/// A reference to a type which implements [`FallbackTryDropStrategy`]. Used as a workaround for
/// implementing [`FallbackTryDropStrategy`] on references.
#[cfg_attr(
feature = "derives",
derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct FallbackTryDropStrategyRef<'a, T: TryDropStrategy>(pub &'a T);

impl<'a, T: TryDropStrategy> TryDropStrategy for FallbackTryDropStrategyRef<'a, T> {
    fn handle_error(&self, error: anyhow::Error) {
        self.0.handle_error(error)
    }
}

/// A type which chains two try drop strategies together, one of which may fail and if so, will be
/// redirected to the fallback, infallible try drop strategy.
///
/// This type implements [`TryDropStrategy`] because, as said before, any and all errors in the
/// fallible try drop strategy will be redirected to the fallback, which can never fail.
#[cfg_attr(
feature = "derives",
derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct FallbackTryDropStrategyHandler<FDS, FTDS>
    where
        FDS: TryDropStrategy,
        FTDS: FallibleTryDropStrategy,
{
    /// The fallback try drop strategy. This will be called if the first try drop strategy fails and
    /// is a last resort to recovering sanely.
    pub fallback_try_drop_strategy: FDS,

    /// The try drop strategy which may fail. This will be called first.
    pub fallible_try_drop_strategy: FTDS,
}

impl<FDS, FTDS> FallbackTryDropStrategyHandler<FDS, FTDS>
    where
        FDS: TryDropStrategy,
        FTDS: FallibleTryDropStrategy,
{
    /// Create a new fallback try drop strategy handler.
    pub fn new(fallback_try_drop_strategy: FDS, fallible_try_drop_strategy: FTDS) -> Self {
        Self {
            fallback_try_drop_strategy,
            fallible_try_drop_strategy,
        }
    }
}

impl<FDS, FTDS> TryDropStrategy for FallbackTryDropStrategyHandler<FDS, FTDS>
    where
        FDS: TryDropStrategy,
        FTDS: FallibleTryDropStrategy,
{
    fn handle_error(&self, error: anyhow::Error) {
        if let Err(error) = self.fallible_try_drop_strategy.dyn_try_handle_error(error) {
            self.fallback_try_drop_strategy
                .handle_error(error)
        }
    }
}
