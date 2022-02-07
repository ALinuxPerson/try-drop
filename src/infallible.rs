#[cfg(feature = "std")]
mod with_std {
    pub use std::convert::Infallible;
}

#[cfg(not(feature = "std"))]
mod no_std {
    #[derive(Copy, Clone)]
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
