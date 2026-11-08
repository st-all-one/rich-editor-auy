//! Parse, invariant, and selection errors.

use alloc::boxed::Box;
use core::fmt;

/// A parse failure with a stable offset and an optional node path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Stable machine-readable code.
    pub code: Box<str>,
    /// Byte offset in the input.
    pub offset: u32,
    /// Node path, when the failure is structural.
    pub path: Box<str>,
}

impl ParseError {
    /// Creates a parse error.
    #[must_use]
    pub fn new(code: &str, offset: u32) -> Self {
        Self {
            code: Box::from(code),
            offset,
            path: Box::from(""),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at offset {}", self.code, self.offset)
    }
}

impl core::error::Error for ParseError {}

/// An internal invariant failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvariantError {
    /// Stable invariant identifier.
    pub invariant: Box<str>,
    /// Byte position associated with the failure, when available.
    pub position: Option<u32>,
}

impl fmt::Display for InvariantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invariant {} violated", self.invariant)
    }
}

impl core::error::Error for InvariantError {}

/// A selection cannot be represented in the current document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionError {
    /// Stable reason.
    pub reason: Box<str>,
    /// Offending position.
    pub position: u32,
}

impl fmt::Display for SelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at {}", self.reason, self.position)
    }
}

impl core::error::Error for SelectionError {}
