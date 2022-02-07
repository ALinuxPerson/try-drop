use std::convert::Infallible as StdInfallible;
use std::marker::PhantomData;
use std::println;
use try_drop::prelude::*;

mod private {
    pub trait Sealed {}
}

pub trait Mode: private::Sealed {}

pub enum Fallible {}

impl Mode for Fallible {}
impl private::Sealed for Fallible {}

pub enum Infallible {}

impl Mode for Infallible {}
impl private::Sealed for Infallible {}

pub enum Random {}

impl Mode for Random {}
impl private::Sealed for Random {}

pub trait TryDropTypes: private::Sealed {}

pub struct NotGiven;

impl TryDropTypes for NotGiven {}
impl private::Sealed for NotGiven {}

pub struct Given<D: FallibleTryDropStrategy, DD: FallbackTryDropStrategy> {
    fallible_try_drop_strategy: D,
    double_drop_strategy: DD,
}

impl<D: FallibleTryDropStrategy, DD: FallbackTryDropStrategy> Given<D, DD> {
    pub fn new(fallible_try_drop_strategy: D, double_drop_strategy: DD) -> Self {
        Self {
            fallible_try_drop_strategy,
            double_drop_strategy,
        }
    }
}

impl<D: FallibleTryDropStrategy, DD: FallbackTryDropStrategy> TryDropTypes for Given<D, DD> {}
impl<D: FallibleTryDropStrategy, DD: FallbackTryDropStrategy> private::Sealed for Given<D, DD> {}

pub struct ErrorsOnDrop<M: Mode, TDT: TryDropTypes> {
    times_try_drop_was_called: usize,
    check_try_drop: bool,
    _marker: PhantomData<M>,
    try_drop_types: TDT,
}

impl<M: Mode, TDT: TryDropTypes> ErrorsOnDrop<M, TDT> {
    fn try_drop_check(&mut self) {
        if self.check_try_drop {
            self.times_try_drop_was_called += 1;
            println!(
                "times try drop was called: {}",
                self.times_try_drop_was_called
            );

            if self.times_try_drop_was_called >= 2 {
                println!("possible soundness hole: try drop called twice or more");
            }
        }
    }

    pub fn check_try_drop(&mut self, check_try_drop: bool) -> &mut Self {
        self.check_try_drop = check_try_drop;
        self
    }
}

impl<M: Mode> ErrorsOnDrop<M, NotGiven> {
    pub fn not_given() -> Self {
        Self {
            times_try_drop_was_called: 0,
            check_try_drop: true,
            _marker: PhantomData,
            try_drop_types: NotGiven,
        }
    }
}

impl<M, D, DD> ErrorsOnDrop<M, Given<D, DD>>
where
    M: Mode,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    pub fn given(fallible_try_drop_strategy: D, double_drop_strategy: DD) -> Self {
        Self {
            times_try_drop_was_called: 0,
            check_try_drop: true,
            _marker: PhantomData,
            try_drop_types: Given::new(fallible_try_drop_strategy, double_drop_strategy),
        }
    }
}

impl<M: Mode> Default for ErrorsOnDrop<M, NotGiven> {
    fn default() -> Self {
        Self::not_given()
    }
}

impl ImpureTryDrop for ErrorsOnDrop<Infallible, NotGiven> {
    type Error = StdInfallible;

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        Ok(())
    }
}

impl<D: FallibleTryDropStrategy, DD: FallbackTryDropStrategy> PureTryDrop
    for ErrorsOnDrop<Infallible, Given<D, DD>>
{
    type Error = StdInfallible;
    type FallbackTryDropStrategy = DD;
    type TryDropStrategy = D;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &self.try_drop_types.double_drop_strategy
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &self.try_drop_types.fallible_try_drop_strategy
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        Ok(())
    }
}

impl ImpureTryDrop for ErrorsOnDrop<Fallible, NotGiven> {
    type Error = try_drop::Error;

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        anyhow::bail!("this will always fail")
    }
}

impl<D: FallibleTryDropStrategy, DD: FallbackTryDropStrategy> PureTryDrop
    for ErrorsOnDrop<Fallible, Given<D, DD>>
{
    type Error = try_drop::Error;
    type FallbackTryDropStrategy = DD;
    type TryDropStrategy = D;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &self.try_drop_types.double_drop_strategy
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &self.try_drop_types.fallible_try_drop_strategy
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        anyhow::bail!("this will always fail")
    }
}

impl ImpureTryDrop for ErrorsOnDrop<Random, NotGiven> {
    type Error = try_drop::Error;

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        let error_out = rand::random::<bool>();

        if error_out {
            anyhow::bail!("random error occured")
        } else {
            Ok(())
        }
    }
}

impl<D: FallibleTryDropStrategy, DD: FallbackTryDropStrategy> PureTryDrop
    for ErrorsOnDrop<Random, Given<D, DD>>
{
    type Error = try_drop::Error;
    type FallbackTryDropStrategy = DD;
    type TryDropStrategy = D;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &self.try_drop_types.double_drop_strategy
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &self.try_drop_types.fallible_try_drop_strategy
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        let error_out = rand::random::<bool>();

        if error_out {
            anyhow::bail!("random error occured")
        } else {
            Ok(())
        }
    }
}

fn main() {}
