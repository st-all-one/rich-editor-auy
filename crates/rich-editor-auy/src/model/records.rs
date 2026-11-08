//! Document-derived records.

use super::ids::{NodeId, RefTarget};
use crate::pos::Range;

/// A node anchor and its absolute range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    /// Stable identifier.
    pub id: NodeId,
    /// Absolute range.
    pub range: Range,
}

/// A cross-publication reference and its source range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    /// Source range.
    pub source: Range,
    /// Target.
    pub target: RefTarget,
}
