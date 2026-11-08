//! Step errors.

use alloc::boxed::Box;
use core::fmt;

use super::{InvariantError, LimitError};

/// A step cannot be applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepError {
    /// Position or range is outside the document.
    OutOfBounds {
        /// Offending position.
        position: u32,
        /// Document size.
        size: u32,
    },
    /// A node or mark does not exist in the schema.
    Unknown {
        /// Node path or name.
        path: Box<str>,
    },
    /// Content does not satisfy the schema.
    ContentMismatch {
        /// Node path.
        path: Box<str>,
    },
    /// A node identifier is malformed or duplicated.
    DuplicateId {
        /// Offending identifier.
        id: Box<str>,
    },
    /// The operation is not representable by the current step algebra.
    Unsupported {
        /// Stable operation name.
        operation: Box<str>,
    },
    /// A configured limit was exceeded.
    Limit(LimitError),
    /// An internal invariant failed.
    Invariant(InvariantError),
}

impl fmt::Display for StepError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { position, size } => {
                write!(
                    formatter,
                    "position {position} outside document size {size}"
                )
            }
            Self::Unknown { path } => write!(formatter, "unknown node or mark: {path}"),
            Self::ContentMismatch { path } => write!(formatter, "content mismatch at {path}"),
            Self::DuplicateId { id } => write!(formatter, "duplicate node id: {id}"),
            Self::Unsupported { operation } => write!(formatter, "unsupported step: {operation}"),
            Self::Limit(error) => error.fmt(formatter),
            Self::Invariant(error) => error.fmt(formatter),
        }
    }
}

impl core::error::Error for StepError {}

impl From<LimitError> for StepError {
    fn from(value: LimitError) -> Self {
        Self::Limit(value)
    }
}

impl From<InvariantError> for StepError {
    fn from(value: InvariantError) -> Self {
        Self::Invariant(value)
    }
}
