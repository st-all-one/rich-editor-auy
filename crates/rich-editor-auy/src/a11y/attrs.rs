//! Attribute identifiers resolved once per tree projection.
//!
//! The projection reads a fixed set of attribute names for every node. Resolving the names to
//! [`AttrId`]s once per `a11y_tree` call turns nine schema scans per node into nine `Attrs::get`
//! probes.
#![allow(
    clippy::redundant_pub_crate,
    reason = "projection helpers are crate-internal"
)]

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::model::{AttrId, AttrValue, Attrs, NodeKind};
use crate::schema::Schema;

/// Attribute identifiers read by name for one node kind.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NodeAttrs {
    /// Link target.
    pub(crate) href: Option<AttrId>,
    /// Media or attachment source.
    pub(crate) src: Option<AttrId>,
    /// Textual alternative.
    pub(crate) alt: Option<AttrId>,
    /// Figure or media caption.
    pub(crate) caption: Option<AttrId>,
    /// Media transcript.
    pub(crate) transcript: Option<AttrId>,
    /// Code language.
    pub(crate) language: Option<AttrId>,
    /// Table cell scope.
    pub(crate) scope: Option<AttrId>,
    /// Table cell column span.
    pub(crate) colspan: Option<AttrId>,
    /// Table cell row span.
    pub(crate) rowspan: Option<AttrId>,
    /// Accessible description.
    pub(crate) description: Option<AttrId>,
}

/// Per-kind attribute identifier cache indexed by [`NodeKind`].
#[derive(Debug)]
pub(crate) struct AttrCache {
    by_kind: Vec<NodeAttrs>,
}

impl AttrCache {
    /// Resolves every projected attribute name for every schema node.
    pub(crate) fn new(schema: &Schema) -> Self {
        let by_kind = schema
            .nodes()
            .iter()
            .map(|spec| NodeAttrs {
                href: attr_id(spec, "href"),
                src: attr_id(spec, "src"),
                alt: attr_id(spec, "alt"),
                caption: attr_id(spec, "caption"),
                transcript: attr_id(spec, "transcript"),
                language: attr_id(spec, "language"),
                scope: attr_id(spec, "scope"),
                colspan: attr_id(spec, "colspan"),
                rowspan: attr_id(spec, "rowspan"),
                description: attr_id(spec, "description"),
            })
            .collect();
        Self { by_kind }
    }

    /// Returns the cached identifiers for one kind, or an empty set.
    pub(crate) fn get(&self, kind: NodeKind) -> NodeAttrs {
        self.by_kind
            .get(usize::from(kind.0))
            .copied()
            .unwrap_or_default()
    }
}

/// Returns the string value of a cached attribute.
pub(crate) fn text(attrs: &Attrs, id: Option<AttrId>) -> Option<Box<str>> {
    match id.and_then(|id| attrs.get(id)) {
        Some(AttrValue::Str(value)) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}

/// Returns the integer value of a cached attribute.
pub(crate) fn integer(attrs: &Attrs, id: Option<AttrId>) -> Option<u16> {
    match id.and_then(|id| attrs.get(id)) {
        Some(AttrValue::Int(value)) => u16::try_from(*value).ok(),
        _ => None,
    }
}

fn attr_id(spec: &crate::schema::NodeSpec, name: &str) -> Option<AttrId> {
    spec.attrs
        .iter()
        .find(|attr| &*attr.name == name)
        .map(|attr| attr.id)
}
