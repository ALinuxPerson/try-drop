use crate::{Error, TryDropStrategy};
use std::borrow::Cow;
use std::string::String;

/// A drop strategy that panics with a message if a drop error occurs.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct PanicDropStrategy {
    /// The message to panic with.
    pub message: Cow<'static, str>,
}

impl PanicDropStrategy {
    /// The default panic drop strategy.
    pub const DEFAULT: Self = Self::with_static_message("error occurred when dropping an object");

    /// Creates a new panic drop strategy with the given message.
    pub fn with_message(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Creates a new panic drop strategy with the given static message.
    pub const fn with_static_message(message: &'static str) -> Self {
        Self {
            message: Cow::Borrowed(message),
        }
    }

    /// Creates a new panic drop strategy with the given string message.
    pub const fn with_dynamic_message(message: String) -> Self {
        Self {
            message: Cow::Owned(message),
        }
    }
}

impl TryDropStrategy for PanicDropStrategy {
    fn handle_error(&self, error: Error) {
        Err(error).expect(&self.message)
    }
}

impl Default for PanicDropStrategy {
    fn default() -> Self {
        Self::DEFAULT
    }
}
