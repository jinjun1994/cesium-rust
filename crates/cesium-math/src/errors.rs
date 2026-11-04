//! DeveloperError and RuntimeError matching CesiumJS `Core/DeveloperError` and `Core/RuntimeError`.
//! Adapted from CesiumJS (Apache-2.0).

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeveloperError {
    pub message: String,
}

impl DeveloperError {
    pub const NAME: &'static str = "DeveloperError";

    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn name(&self) -> &'static str {
        Self::NAME
    }
}

impl fmt::Display for DeveloperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Self::NAME, self.message)
    }
}

impl std::error::Error for DeveloperError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub const NAME: &'static str = "RuntimeError";

    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn name(&self) -> &'static str {
        Self::NAME
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Self::NAME, self.message)
    }
}

impl std::error::Error for RuntimeError {}
