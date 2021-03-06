#![allow(missing_docs)]

use crate::prelude::*;
use crate::{DropAdapter, Infallible as TryDropInfallible};
use std::marker::PhantomData;
use std::println;
use anyhow::Error;

mod private {
    pub trait Sealed {}
}

/// A drop strategy which always fails.
pub struct FallibleDropStrategy;

impl FallibleTryDropStrategy for FallibleDropStrategy {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        Err(error)
    }
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

pub fn fallible() -> DropAdapter<ErrorsOnDrop<Fallible, NotGiven>> {
    ErrorsOnDrop::<Fallible, _>::not_given().adapt()
}

pub fn fallible_given<D: FallibleTryDropStrategy, DD: TryDropStrategy>(primary: D, fallback: DD) -> DropAdapter<ErrorsOnDrop<Fallible, Given<D, DD>>> {
    ErrorsOnDrop::<Fallible, _>::given(primary, fallback).adapt()
}

pub struct Given<D: FallibleTryDropStrategy, DD: TryDropStrategy> {
    fallible_try_drop_strategy: D,
    double_drop_strategy: DD,
}

impl<D: FallibleTryDropStrategy, DD: TryDropStrategy> Given<D, DD> {
    pub fn new(fallible_try_drop_strategy: D, double_drop_strategy: DD) -> Self {
        Self {
            fallible_try_drop_strategy,
            double_drop_strategy,
        }
    }
}

impl<D: FallibleTryDropStrategy, DD: TryDropStrategy> TryDropTypes for Given<D, DD> {}
impl<D: FallibleTryDropStrategy, DD: TryDropStrategy> private::Sealed for Given<D, DD> {}

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
    DD: TryDropStrategy,
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
    type Error = TryDropInfallible;

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        Ok(())
    }
}

impl<D: FallibleTryDropStrategy, DD: TryDropStrategy> PureTryDrop
    for ErrorsOnDrop<Infallible, Given<D, DD>>
{
    type Error = TryDropInfallible;
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
    type Error = crate::Error;

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        anyhow::bail!("this will always fail")
    }
}

impl<D: FallibleTryDropStrategy, DD: TryDropStrategy> PureTryDrop
    for ErrorsOnDrop<Fallible, Given<D, DD>>
{
    type Error = crate::Error;
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
    type Error = crate::Error;

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.try_drop_check();
        let error_out = rand::random::<bool>();

        if error_out {
            anyhow::bail!("random error occurred")
        } else {
            Ok(())
        }
    }
}

impl<D: FallibleTryDropStrategy, DD: TryDropStrategy> PureTryDrop
    for ErrorsOnDrop<Random, Given<D, DD>>
{
    type Error = crate::Error;
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
