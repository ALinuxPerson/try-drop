use std::io;
use std::marker::PhantomData;
use crate::drop_strategies::{PanicDropStrategy, WriteDropStrategy};
use crate::on_uninit::OnUninit;

mod private {
    pub trait Sealed {}
}

pub trait ShimOnUninit: private::Sealed {
    type ExtraData;
}

impl<T: OnUninit> ShimOnUninit for T {
    type ExtraData = T::ExtraData;
}
impl<T: OnUninit> private::Sealed for T {}

pub struct UseDefaultOnUninitShim<H: Handler>(PhantomData<H>);

impl ShimOnUninit for UseDefaultOnUninitShim<PrimaryHandler> {
    type ExtraData = WriteDropStrategy<io::Stderr>;
}
impl ShimOnUninit for UseDefaultOnUninitShim<FallbackHandler> {
    type ExtraData = PanicDropStrategy;
}

impl<H: Handler> private::Sealed for UseDefaultOnUninitShim<H> {}

pub trait Handler: private::Sealed {}

pub enum FallbackHandler {}
impl Handler for FallbackHandler {}
impl private::Sealed for FallbackHandler {}

pub enum PrimaryHandler {}
impl Handler for PrimaryHandler {}
impl private::Sealed for PrimaryHandler {}
