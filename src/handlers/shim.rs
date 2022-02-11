use std::io;
use std::marker::PhantomData;
use once_cell::unsync::Lazy;
use crate::drop_strategies::{PanicDropStrategy, WriteDropStrategy};
use crate::on_uninit::OnUninit;

mod private {
    pub trait Sealed {}
}

pub trait OnUninitShim: private::Sealed {
    type ExtraData;
}

impl<T: OnUninit> OnUninitShim for T {
    type ExtraData = T::ExtraData;
}
impl<T: OnUninit> private::Sealed for T {}

pub struct UseDefaultOnUninitShim<H: Handler>(PhantomData<H>);

impl OnUninitShim for UseDefaultOnUninitShim<PrimaryHandler> {
    type ExtraData = Lazy<WriteDropStrategy<io::Stderr>>;
}
impl OnUninitShim for UseDefaultOnUninitShim<FallbackHandler> {
    type ExtraData = Lazy<PanicDropStrategy>;
}

impl<H: Handler> private::Sealed for UseDefaultOnUninitShim<H> {}

pub trait Handler: private::Sealed {}

pub enum FallbackHandler {}
impl Handler for FallbackHandler {}
impl private::Sealed for FallbackHandler {}

pub enum PrimaryHandler {}
impl Handler for PrimaryHandler {}
impl private::Sealed for PrimaryHandler {}
