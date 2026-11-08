//! Limit errors.

use core::fmt;

/// A configured document or operation limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LimitError {
    /// The document exceeds its byte budget.
    DocBytes {
        /// Observed value.
        actual: u64,
        /// Allowed value.
        max: u32,
    },
    /// A block exceeds its text budget.
    BlockTextBytes {
        /// Observed value.
        actual: u64,
        /// Allowed value.
        max: u32,
    },
    /// Structural depth exceeds its budget.
    Depth {
        /// Observed depth.
        actual: u32,
        /// Allowed depth.
        max: u32,
    },
    /// Node count exceeds its budget.
    Nodes {
        /// Observed count.
        actual: u32,
        /// Allowed count.
        max: u32,
    },
    /// An attribute string exceeds its byte budget.
    AttrStringBytes {
        /// Observed value.
        actual: u64,
        /// Allowed value.
        max: u32,
    },
    /// A run exceeds its byte budget.
    RunBytes {
        /// Observed value.
        actual: u64,
        /// Allowed value.
        max: u32,
    },
    /// A node has too many attributes.
    Attrs {
        /// Observed value.
        actual: u32,
        /// Allowed value.
        max: u32,
    },
    /// A mark has too many attributes.
    MarkAttrs {
        /// Observed value.
        actual: u32,
        /// Allowed value.
        max: u32,
    },
    /// A text block has too many runs.
    Runs {
        /// Observed value.
        actual: u32,
        /// Allowed value.
        max: u32,
    },
}

impl fmt::Display for LimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocBytes { actual, max } | Self::BlockTextBytes { actual, max } => {
                write!(formatter, "byte limit exceeded: {actual} > {max}")
            }
            Self::Depth { actual, max } => {
                write!(formatter, "depth limit exceeded: {actual} > {max}")
            }
            Self::Nodes { actual, max } => {
                write!(formatter, "node limit exceeded: {actual} > {max}")
            }
            Self::AttrStringBytes { actual, max } => {
                write!(
                    formatter,
                    "attribute string limit exceeded: {actual} > {max}"
                )
            }
            Self::RunBytes { actual, max } => {
                write!(formatter, "run limit exceeded: {actual} > {max}")
            }
            Self::Attrs { actual, max } => {
                write!(formatter, "attribute count exceeded: {actual} > {max}")
            }
            Self::MarkAttrs { actual, max } => {
                write!(formatter, "mark attribute count exceeded: {actual} > {max}")
            }
            Self::Runs { actual, max } => {
                write!(formatter, "run count exceeded: {actual} > {max}")
            }
        }
    }
}

impl core::error::Error for LimitError {}
