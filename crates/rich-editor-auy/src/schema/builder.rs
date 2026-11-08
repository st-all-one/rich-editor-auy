//! Schema construction and validation.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::error::SchemaError;
use crate::model::{AttrId, MarkId, NodeKind};

use super::expr;
use super::machine;
use super::{DocSpec, MarkSpec, NodeSpec, Schema};

/// Incremental schema builder.
#[derive(Clone, Debug)]
pub struct SchemaBuilder {
    doc: DocSpec,
    nodes: Vec<NodeSpec>,
    marks: Vec<MarkSpec>,
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaBuilder {
    /// Creates a builder with a default document declaration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            doc: DocSpec::default(),
            nodes: Vec::new(),
            marks: Vec::new(),
        }
    }

    /// Replaces the document declaration.
    #[must_use]
    pub fn doc(mut self, doc: DocSpec) -> Self {
        self.doc = doc;
        self
    }

    /// Adds a node declaration.
    #[must_use]
    pub fn node(mut self, node: NodeSpec) -> Self {
        self.nodes.push(node);
        self
    }

    /// Adds a mark declaration.
    #[must_use]
    pub fn mark(mut self, mark: MarkSpec) -> Self {
        self.marks.push(mark);
        self
    }

    /// Compiles and validates the schema.
    ///
    /// # Errors
    ///
    /// Returns a structured schema error for duplicate names, invalid content, or invalid
    /// accessibility declarations.
    #[allow(clippy::too_many_lines, reason = "single-pass schema compilation")]
    pub fn build(self) -> Result<Schema, SchemaError> {
        if self.marks.len() > 64 {
            return Err(SchemaError::TooManyMarks {
                count: u32::try_from(self.marks.len()).unwrap_or(u32::MAX),
            });
        }

        let mut nodes = self.nodes;
        let mut node_by_name = BTreeMap::new();
        for (index, node) in nodes.iter_mut().enumerate() {
            let kind = NodeKind(u16::try_from(index).map_err(|_| SchemaError::InvalidName {
                name: Box::from("too_many_nodes"),
            })?);
            if node.name.is_empty() || node_by_name.insert(node.name.clone(), kind).is_some() {
                return Err(SchemaError::InvalidName {
                    name: node.name.clone(),
                });
            }
            node.kind = kind;
            validate_attrs(&node.attrs, true)?;
            validate_a11y(node.a11y, &node.attrs)?;
        }

        let mut marks = self.marks;
        let mut mark_by_name = BTreeMap::new();
        for (index, mark) in marks.iter_mut().enumerate() {
            let id = MarkId(u16::try_from(index).map_err(|_| SchemaError::InvalidName {
                name: Box::from("too_many_marks"),
            })?);
            if mark.name.is_empty() || mark_by_name.insert(mark.name.clone(), id).is_some() {
                return Err(SchemaError::InvalidName {
                    name: mark.name.clone(),
                });
            }
            mark.id = id;
            validate_attrs(&mark.attrs, false)?;
            validate_a11y(mark.a11y, &mark.attrs)?;
        }

        let mut groups: BTreeMap<Box<str>, Vec<NodeKind>> = BTreeMap::new();
        for node in &nodes {
            for group in &node.groups {
                groups.entry(group.clone()).or_default().push(node.kind);
            }
        }

        let resolve = |name: &str| -> Option<Vec<NodeKind>> {
            if let Some(kind) = node_by_name.get(name) {
                return Some(Vec::from([*kind]));
            }
            groups.get(name).cloned()
        };

        let doc_expression = expr::parse(&self.doc.content)?;
        let doc_machine = machine::compile(&doc_expression, &resolve)?;
        let mut node_machines = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let machine = match &node.content {
                Some(content) => {
                    let parsed = expr::parse(content)?;
                    Some(machine::compile(&parsed, &resolve)?)
                }
                None => None,
            };
            node_machines.push(machine);
        }

        validate_doc_a11y(self.doc.a11y, &self.doc.attrs)?;
        validate_attrs(&self.doc.attrs, false)?;

        let fingerprint = super::fingerprint::fingerprint(&self.doc, &nodes, &marks);
        Ok(Schema {
            doc: self.doc,
            nodes: nodes.into_boxed_slice(),
            marks: marks.into_boxed_slice(),
            node_by_name,
            mark_by_name,
            groups,
            doc_machine,
            node_machines: node_machines.into_boxed_slice(),
            fingerprint: fingerprint.into_boxed_str(),
        })
    }
}

#[allow(
    clippy::fn_params_excessive_bools,
    reason = "node and document attr tables differ only by the reserved id"
)]
fn validate_attrs(attrs: &[super::AttrSpec], node: bool) -> Result<(), SchemaError> {
    let mut names = BTreeMap::new();
    for attr in attrs {
        if attr.name.is_empty() || names.insert(attr.name.clone(), attr.id).is_some() {
            return Err(SchemaError::InvalidName {
                name: attr.name.clone(),
            });
        }
        if node && attr.id == AttrId::NODE_ID && &*attr.name != "id" {
            return Err(SchemaError::InvalidName {
                name: attr.name.clone(),
            });
        }
        if node && &*attr.name == "id" && attr.id != AttrId::NODE_ID {
            return Err(SchemaError::InvalidName {
                name: attr.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_a11y(
    spec: Option<crate::a11y::A11ySpec>,
    attrs: &[super::AttrSpec],
) -> Result<(), SchemaError> {
    let Some(spec) = spec else {
        return Ok(());
    };
    if let Some(code) = spec.error_code() {
        return Err(SchemaError::A11y { code });
    }
    if let Some(level) = spec.level
        && !attrs.iter().any(|attr| attr.id == level)
    {
        return Err(SchemaError::A11y {
            code: Box::from("heading_attr"),
        });
    }
    Ok(())
}

fn validate_doc_a11y(
    spec: Option<crate::a11y::A11ySpec>,
    attrs: &[super::AttrSpec],
) -> Result<(), SchemaError> {
    let Some(spec) = spec else {
        return Ok(());
    };
    if let Some(code) = spec.error_code() {
        return Err(SchemaError::A11y { code });
    }
    if let Some(level) = spec.level
        && !attrs.iter().any(|attr| attr.id == level)
    {
        return Err(SchemaError::A11y {
            code: Box::from("document_attr"),
        });
    }
    Ok(())
}
