//! Schema errors.

use alloc::boxed::Box;
use core::fmt;

/// A schema declaration or compilation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaError {
    /// A node name is empty or duplicated.
    InvalidName {
        /// Offending name.
        name: Box<str>,
    },
    /// A referenced node or group does not exist.
    UnknownName {
        /// Offending name.
        name: Box<str>,
    },
    /// Content expression is malformed.
    InvalidContent {
        /// Offending expression.
        expression: Box<str>,
        /// Byte offset of the failure.
        offset: u32,
    },
    /// Content grammar accepts the empty string where that is impossible.
    NonProductive {
        /// Node whose declaration is non-productive.
        name: Box<str>,
    },
    /// More marks were declared than the mask supports.
    TooManyMarks {
        /// Number of declared marks.
        count: u32,
    },
    /// An accessibility declaration is invalid.
    A11y {
        /// Human-readable, stable reason code.
        code: Box<str>,
    },
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { name } => write!(formatter, "invalid schema name: {name}"),
            Self::UnknownName { name } => write!(formatter, "unknown schema name: {name}"),
            Self::InvalidContent { expression, offset } => {
                write!(
                    formatter,
                    "invalid content expression at {offset}: {expression}"
                )
            }
            Self::NonProductive { name } => write!(formatter, "non-productive node: {name}"),
            Self::TooManyMarks { count } => write!(formatter, "too many marks: {count}"),
            Self::A11y { code } => write!(formatter, "invalid accessibility declaration: {code}"),
        }
    }
}

impl core::error::Error for SchemaError {}
