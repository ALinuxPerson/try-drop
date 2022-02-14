use crate::handlers::on_uninit::OnUninit;

mod private {
    pub trait Sealed {}
}
#[cfg(any(feature = "ds-write", feature = "ds-panic"))]
mod use_default {
    use super::private;
    use crate::handlers::common::shim::OnUninitShim;
    use crate::handlers::common::{Fallback, Handler, Primary};
    use once_cell::sync::Lazy;
    use std::marker::PhantomData;

    #[cfg_attr(
        feature = "derives",
        derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
    )]
    pub struct UseDefaultOnUninitShim<H: Handler>(PhantomData<H>);

    #[cfg(feature = "ds-write")]
    impl OnUninitShim for UseDefaultOnUninitShim<Primary> {
        type ExtraData = Lazy<crate::drop_strategies::WriteDropStrategy<std::io::Stderr>>;
    }

    #[cfg(feature = "ds-panic")]
    impl OnUninitShim for UseDefaultOnUninitShim<Fallback> {
        type ExtraData = Lazy<crate::drop_strategies::PanicDropStrategy>;
    }

    impl<H: Handler> private::Sealed for UseDefaultOnUninitShim<H> {}
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
