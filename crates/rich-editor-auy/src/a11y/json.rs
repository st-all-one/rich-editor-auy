//! Deterministic JSON projection of the semantic tree.
#![allow(
    clippy::excessive_nesting,
    reason = "one ordered writer keeps the schema explicit"
)]

use core::fmt::{self, Write};

use alloc::string::String;

use crate::model::NodeId;
use crate::schema::Schema;
use crate::wire::write_string;

use super::tree::{A11yNode, A11yTree, CellScope};

impl A11yTree {
    /// Writes the semantic tree as canonical JSON.
    ///
    /// # Errors
    ///
    /// Propagates formatting failures from the sink.
    pub fn to_json_into(&self, schema: &Schema, out: &mut impl Write) -> fmt::Result {
        out.write_str("{\"role\":\"document\"")?;
        write_str_field(out, "lang", self.lang.as_deref())?;
        write_str_field(out, "dir", self.dir.as_deref())?;
        out.write_str(",\"range\":")?;
        write_range(&self.root, out)?;
        out.write_str(",\"children\":[")?;
        for (index, child) in self.root.children.iter().enumerate() {
            if index > 0 {
                out.write_char(',')?;
            }
            write_node(child, schema, out)?;
        }
        out.write_str("]}")
    }

    /// Returns the semantic tree as canonical JSON.
    #[must_use]
    pub fn to_json_string(&self, schema: &Schema) -> String {
        let mut out = String::new();
        if self.to_json_into(schema, &mut out).is_err() {
            return String::new();
        }
        out
    }
}

fn write_node(node: &A11yNode, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    out.write_str("{\"role\":")?;
    write_string(node.role.name(), out)?;
    write_str_field(out, "path", Some(node.path.as_str()))?;
    write_str_field(out, "id", node.id.as_ref().map(NodeId::as_str))?;
    write_str_field(out, "name", node.name.as_deref())?;
    write_str_field(out, "description", node.description.as_deref())?;
    write_str_field(out, "alt", node.alt.as_deref())?;
    write_str_field(out, "lang", node.lang.as_deref())?;
    write_str_field(out, "dir", node.dir.as_deref())?;
    if let Some(level) = node.level {
        write!(out, ",\"level\":{level}")?;
    }
    write_str_field(out, "text", node.text.as_deref())?;
    write_marks(node, schema, out)?;
    write_str_field(out, "href", node.href.as_deref())?;
    write_str_field(out, "src", node.src.as_deref())?;
    write_str_field(out, "caption", node.caption.as_deref())?;
    write_str_field(out, "transcript", node.transcript.as_deref())?;
    if let Some(ordered) = node.ordered {
        write!(out, ",\"ordered\":{ordered}")?;
    }
    if let Some(scope) = node.scope {
        write!(out, ",\"scope\":\"{}\"", scope_name(scope))?;
    }
    if let Some(colspan) = node.colspan {
        write!(out, ",\"colspan\":{colspan}")?;
    }
    if let Some(rowspan) = node.rowspan {
        write!(out, ",\"rowspan\":{rowspan}")?;
    }
    out.write_str(",\"range\":")?;
    write_range(node, out)?;
    out.write_str(",\"children\":[")?;
    for (index, child) in node.children.iter().enumerate() {
        if index > 0 {
            out.write_char(',')?;
        }
        write_node(child, schema, out)?;
    }
    out.write_str("]}")
}

fn write_range(node: &A11yNode, out: &mut impl Write) -> fmt::Result {
    write!(
        out,
        "{{\"from\":{},\"to\":{}}}",
        node.range.start.0, node.range.end.0
    )
}

fn write_str_field(out: &mut impl Write, key: &str, value: Option<&str>) -> fmt::Result {
    let Some(value) = value else {
        return Ok(());
    };
    out.write_char(',')?;
    write_string(key, out)?;
    out.write_char(':')?;
    write_string(value, out)
}

fn write_marks(node: &A11yNode, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    if node.marks.is_empty() {
        return Ok(());
    }
    out.write_str(",\"marks\":[")?;
    let mut first = true;
    for mark in schema.marks() {
        if !node.marks.contains(mark.id) {
            continue;
        }
        if !first {
            out.write_char(',')?;
        }
        first = false;
        write_string(&mark.name, out)?;
    }
    out.write_char(']')
}

const fn scope_name(scope: CellScope) -> &'static str {
    match scope {
        CellScope::Col => "col",
        CellScope::Row => "row",
        CellScope::ColGroup => "col_group",
        CellScope::RowGroup => "row_group",
    }
}
