use crate::handlers::on_uninit::OnUninit;

mod private {
    pub trait Sealed {}
}
#[cfg(any(feature = "ds-write", feature = "ds-panic"))]
mod use_default {
    use super::private;
    use crate::handlers::common::shim::OnUninitShim;
    use once_cell::sync::Lazy;
    use std::marker::PhantomData;

    #[cfg_attr(
        feature = "derives",
        derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
    )]
    pub struct UseDefaultOnUninitShim<H: Handler>(PhantomData<H>);

    #[cfg(feature = "ds-write")]
    impl OnUninitShim for UseDefaultOnUninitShim<PrimaryHandler> {
        type ExtraData = Lazy<crate::drop_strategies::WriteDropStrategy<std::io::Stderr>>;
    }

    #[cfg(feature = "ds-panic")]
    impl OnUninitShim for UseDefaultOnUninitShim<FallbackHandler> {
        type ExtraData = Lazy<crate::drop_strategies::PanicDropStrategy>;
    }

    impl<H: Handler> private::Sealed for UseDefaultOnUninitShim<H> {}

    pub trait Handler: private::Sealed {}

    #[cfg_attr(
        feature = "derives",
        derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
    )]
    pub enum FallbackHandler {}
    impl Handler for FallbackHandler {}
    impl private::Sealed for FallbackHandler {}

    #[cfg_attr(
        feature = "derives",
        derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
    )]
    pub enum PrimaryHandler {}
    impl Handler for PrimaryHandler {}
    impl private::Sealed for PrimaryHandler {}
}

#[cfg(any(feature = "ds-write", feature = "ds-panic"))]
pub use use_default::*;

pub trait OnUninitShim: private::Sealed {
    type ExtraData;
}

impl<T: OnUninit> OnUninitShim for T {
    type ExtraData = T::ExtraData;
}
impl<T: OnUninit> private::Sealed for T {}