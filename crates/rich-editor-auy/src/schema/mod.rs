//! Runtime schema: declarations, content machines, and fingerprint.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

mod builder;
mod expr;
mod fingerprint;
mod full;
mod machine;
mod mark;
mod preset;
mod spec;

pub use builder::SchemaBuilder;
pub use mark::MarkSpec;
pub use spec::{AttrSpec, AttrType, DocSpec, MarkMode, NodeSpec};

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::error::SchemaError;
use crate::model::{AttrId, AttrValue, Attrs, MarkId, NodeKind};

use machine::ContentMachine;

/// Compiled, immutable schema.
#[derive(Clone, Debug)]
pub struct Schema {
    doc: DocSpec,
    nodes: Box<[NodeSpec]>,
    marks: Box<[MarkSpec]>,
    node_by_name: BTreeMap<Box<str>, NodeKind>,
    mark_by_name: BTreeMap<Box<str>, MarkId>,
    groups: BTreeMap<Box<str>, Vec<NodeKind>>,
    doc_machine: ContentMachine,
    node_machines: Box<[Option<ContentMachine>]>,
    fingerprint: Box<str>,
}

impl Schema {
    /// Starts a schema builder.
    #[must_use]
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    /// Returns the document declaration.
    #[must_use]
    pub const fn doc(&self) -> &DocSpec {
        &self.doc
    }

    /// Returns all node declarations in declaration order.
    #[must_use]
    pub fn nodes(&self) -> &[NodeSpec] {
        &self.nodes
    }

    /// Returns all marks in declaration order.
    #[must_use]
    pub fn marks(&self) -> &[MarkSpec] {
        &self.marks
    }

    /// Returns the node kinds in one group, in declaration order.
    #[must_use]
    pub fn group_kinds(&self, name: &str) -> Option<&[NodeKind]> {
        self.groups.get(name).map(Vec::as_slice)
    }

    /// Returns a node declaration.
    #[must_use]
    pub fn node(&self, kind: NodeKind) -> Option<&NodeSpec> {
        self.nodes.get(usize::from(kind.0))
    }

    /// Resolves a node name.
    #[must_use]
    pub fn node_kind(&self, name: &str) -> Option<NodeKind> {
        self.node_by_name.get(name).copied()
    }

    /// Resolves a mark name.
    #[must_use]
    pub fn mark_id(&self, name: &str) -> Option<MarkId> {
        self.mark_by_name.get(name).copied()
    }

    /// Returns a mark declaration.
    #[must_use]
    pub fn mark(&self, id: MarkId) -> Option<&MarkSpec> {
        self.marks.get(usize::from(id.0))
    }

    /// Returns the canonical BLAKE3 fingerprint as 64 lowercase hex characters.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Returns whether a child sequence satisfies a node's content expression.
    #[must_use]
    pub fn content_allows(&self, kind: NodeKind, children: &[NodeKind]) -> bool {
        self.node_machines
            .get(usize::from(kind.0))
            .and_then(Option::as_ref)
            .is_some_and(|machine| machine.allows(children))
    }

    /// Returns whether the document root accepts a child sequence.
    #[must_use]
    pub fn doc_allows(&self, children: &[NodeKind]) -> bool {
        self.doc_machine.allows(children)
    }

    /// Returns whether a node may be empty.
    #[must_use]
    pub fn content_can_be_empty(&self, kind: NodeKind) -> bool {
        self.node_machines
            .get(usize::from(kind.0))
            .and_then(Option::as_ref)
            .is_none_or(ContentMachine::can_be_empty)
    }

    /// Returns the first deterministic filler kind for a node.
    #[must_use]
    pub fn filler(&self, kind: NodeKind) -> Option<NodeKind> {
        self.node_machines
            .get(usize::from(kind.0))
            .and_then(Option::as_ref)
            .and_then(ContentMachine::first_kind)
    }

    /// Returns whether a mark is allowed on a node.
    #[must_use]
    pub fn mark_allowed(&self, kind: NodeKind, mark: MarkId) -> bool {
        let Some(node) = self.node(kind) else {
            return false;
        };
        match node.marks {
            MarkMode::None => false,
            MarkMode::All => self.mark(mark).is_some(),
        }
    }

    /// Returns the declared attribute with a given name.
    #[must_use]
    pub fn node_attr(&self, kind: NodeKind, name: &str) -> Option<&AttrSpec> {
        self.node(kind)?
            .attrs
            .iter()
            .find(|attr| &*attr.name == name)
    }

    /// Validates attribute values for a node.
    ///
    /// # Errors
    ///
    /// Returns an error when an attribute is undeclared, missing, or has the wrong type.
    pub fn validate_attrs(&self, kind: NodeKind, attrs: &Attrs) -> Result<(), SchemaError> {
        let Some(node) = self.node(kind) else {
            return Err(SchemaError::UnknownName {
                name: Box::from("node"),
            });
        };
        for pair in attrs {
            let Some(spec) = node.attrs.iter().find(|attr| attr.id == pair.id) else {
                return Err(SchemaError::A11y {
                    code: Box::from("unknown_attr"),
                });
            };
            validate_value(&spec.kind, &pair.value)?;
            if is_url_attr(&spec.name)
                && let AttrValue::Str(url) = &pair.value
                && is_unsafe_url(url)
            {
                return Err(SchemaError::A11y {
                    code: Box::from("unsafe_url"),
                });
            }
        }
        for spec in &node.attrs {
            if spec.required && attrs.get(spec.id).is_none() && spec.default.is_none() {
                return Err(SchemaError::A11y {
                    code: Box::from("required_attr"),
                });
            }
        }
        Ok(())
    }

    /// Returns the reserved node-id attribute index.
    #[must_use]
    pub const fn node_id_attr() -> AttrId {
        AttrId::NODE_ID
    }

    /// Returns whether a canonical value has the declared type.
    #[must_use]
    pub fn value_matches(kind: &AttrType, value: &AttrValue) -> bool {
        validate_value(kind, value).is_ok()
    }
}

/// Returns whether an attribute name carries a navigable URL.
#[must_use]
pub(crate) fn is_url_attr(name: &str) -> bool {
    matches!(name, "src" | "href" | "poster" | "canonical_url")
}

/// Returns whether a URL uses a scheme forbidden by the ingestion contract.
#[must_use]
pub(crate) fn is_unsafe_url(url: &str) -> bool {
    let url = url.trim_start();
    url.get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        || url
            .get(..11)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("javascript:"))
}

fn validate_value(kind: &AttrType, value: &AttrValue) -> Result<(), SchemaError> {
    let matches = match (kind, value) {
        (AttrType::Any, _)
        | (AttrType::Str, AttrValue::Str(_))
        | (AttrType::Bool, AttrValue::Bool(_))
        | (AttrType::Ref, AttrValue::Ref(_)) => true,
        (AttrType::Int { min, max }, AttrValue::Int(number)) => *min <= *number && *number <= *max,
        (AttrType::Enum(allowed), AttrValue::Str(text)) => {
            allowed.iter().any(|value| value.as_ref() == text.as_ref())
        }
        (AttrType::List(inner), AttrValue::Array(items)) => {
            items.iter().all(|item| validate_value(inner, item).is_ok())
        }
        (AttrType::Map(inner), AttrValue::Map(entries)) => {
            let canonical = entries.windows(2).all(|pair| match pair {
                [left, right] => left.0 < right.0,
                _ => true,
            });
            canonical
                && entries
                    .iter()
                    .all(|(_, value)| validate_value(inner, value).is_ok())
        }
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(SchemaError::A11y {
            code: Box::from("attr_type"),
        })
    }
}
