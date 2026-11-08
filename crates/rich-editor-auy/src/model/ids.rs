//! Stable identifiers and reference targets.

use alloc::boxed::Box;
use alloc::string::String;

/// Maximum identifier length in bytes.
pub const MAX_ID_BYTES: usize = 128;

/// Maximum version tag length in bytes.
pub const MAX_VERSION_BYTES: usize = 64;

/// Index of a node declaration in a schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeKind(pub u16);

impl NodeKind {
    /// Returns the raw schema index.
    #[must_use]
    pub const fn index(self) -> u16 {
        self.0
    }
}

/// Index of a mark declaration in a schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MarkId(pub u16);

/// Index of an attribute declaration in a node, mark, or document spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttrId(pub u16);

impl AttrId {
    /// Reserved attribute holding a block anchor.
    pub const NODE_ID: Self = Self(0);

    /// Returns the raw declaration index.
    #[must_use]
    pub const fn index(self) -> u16 {
        self.0
    }
}

/// Stable undo group identifier supplied by the host or command layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupId(pub u64);

/// Opaque publication identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocId(Box<str>);

impl DocId {
    /// Creates a publication identifier after ASCII/length validation.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        valid_id(value).then(|| Self(Box::from(value)))
    }

    /// Returns the identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque node anchor identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(Box<str>);

impl NodeId {
    /// Creates a node identifier after ASCII/length validation.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        valid_id(value).then(|| Self(Box::from(value)))
    }

    /// Returns the identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque publication version tag.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionTag(Box<str>);

impl VersionTag {
    /// Creates a version tag after ASCII/length validation.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        (value.len() <= MAX_VERSION_BYTES && value.bytes().all(|byte| byte.is_ascii_graphic()))
            .then(|| Self(Box::from(value)))
    }

    /// Returns the tag text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Typed target of a cross-publication reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefTarget {
    /// Target publication.
    pub doc: DocId,
    /// Optional target node.
    pub node: Option<NodeId>,
    /// Optional fixed version.
    pub version: Option<VersionTag>,
    /// Optional sub-range inside the target node.
    pub fragment: Option<crate::pos::Range>,
}

/// Returns whether an identifier is ASCII graphic, non-empty, and bounded.
#[must_use]
pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_BYTES
        && value.bytes().all(|byte| byte.is_ascii_graphic())
}

/// Returns whether a node identifier is already present in a list.
#[must_use]
pub fn contains_node_id(ids: &[NodeId], candidate: &NodeId) -> bool {
    ids.iter().any(|id| id == candidate)
}

/// Formats a node identifier for diagnostics.
#[must_use]
pub fn id_string(id: &NodeId) -> String {
    String::from(id.as_str())
}
