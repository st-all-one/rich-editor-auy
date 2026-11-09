//! Closed error types shared by the core.

mod limit;
mod problem;
mod schema;
mod step;
mod wire;

pub use limit::LimitError;
pub use problem::{PROBLEM_BASE, ProblemDetails, suggestion_for_token};
pub use schema::SchemaError;
pub use step::StepError;
pub use wire::{InvariantError, ParseError, SelectionError};

use alloc::string::String;
use core::fmt;

/// Stable, implementation-independent error category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Invalid schema declaration.
    Schema,
    /// Invalid edit step.
    Step,
    /// Invalid selection.
    Selection,
    /// Configured limit exceeded.
    Limit,
    /// Malformed wire data.
    Parse,
    /// Document is incompatible with the active schema.
    SchemaMismatch,
    /// Internal invariant violated.
    Invariant,
}

/// Top-level editing error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// Schema failure.
    Schema(SchemaError),
    /// Step failure.
    Step(StepError),
    /// Selection failure.
    Selection(SelectionError),
    /// Limit failure.
    Limit(LimitError),
    /// Parse failure.
    Parse(ParseError),
    /// Internal invariant failure.
    Invariant(InvariantError),
}

impl fmt::Display for EditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Schema(error) => error.fmt(formatter),
            Self::Step(error) => error.fmt(formatter),
            Self::Selection(error) => error.fmt(formatter),
            Self::Limit(error) => error.fmt(formatter),
            Self::Parse(error) => error.fmt(formatter),
            Self::Invariant(error) => error.fmt(formatter),
        }
    }
}

impl core::error::Error for EditError {}

impl From<SchemaError> for EditError {
    fn from(value: SchemaError) -> Self {
        Self::Schema(value)
    }
}

impl From<StepError> for EditError {
    fn from(value: StepError) -> Self {
        Self::Step(value)
    }
}

impl From<LimitError> for EditError {
    fn from(value: LimitError) -> Self {
        Self::Limit(value)
    }
}

impl From<ParseError> for EditError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<InvariantError> for EditError {
    fn from(value: InvariantError) -> Self {
        Self::Invariant(value)
    }
}

/// Human-readable error string for adapters, allocating once.
#[must_use]
pub fn display_string(error: &EditError) -> String {
    alloc::format!("{error}")
}
