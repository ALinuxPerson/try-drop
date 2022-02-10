use std::error::Error;
use std::fmt;

/// This error occurs when an attempt to get a drop strategy is made before it is initialized.
#[cfg_attr(
    feature = "derives",
    derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[derive(Debug)]
pub struct UninitializedError(pub(crate) ());

impl Error for UninitializedError {}

impl fmt::Display for UninitializedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the drop strategy is not initialized yet")
    }
}
