#[cfg(feature = "std")]
mod with_std {
    pub use std::convert::Infallible;
}

#[cfg(not(feature = "std"))]
mod no_std {
    /// The error type for errors which can never happen.
    ///
    /// This is only used as a drop-in replacement for [`core::convert::Infallible`], so that
    /// `anyhow` has a never error type.
    ///
    /// For more information, see [`core::convert::Infallible`].
    #[derive(Copy, Clone)]
    #[cfg_attr(
        feature = "derives",
        derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)
    )]
    pub enum Infallible {}

    impl From<Infallible> for anyhow::Error {
        fn from(infallible: Infallible) -> anyhow::Error {
            match infallible {}
        }
    }
}

#[cfg(feature = "std")]
pub use with_std::*;

#[cfg(not(feature = "std"))]
pub use no_std::*;
