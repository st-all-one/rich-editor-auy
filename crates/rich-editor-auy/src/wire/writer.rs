//! Canonical JSON writer. No intermediate tree and no whitespace.
#![allow(
    clippy::excessive_nesting,
    reason = "canonical writer mirrors the wire grammar"
)]

use core::fmt::{self, Write};

use crate::model::{AttrValue, Attrs, Block, BlockContent, Doc, InlineItem, MarkSet};
use crate::schema::{AttrSpec, Schema};

/// Writes a document as canonical JSON.
///
/// # Errors
///
/// Propagates formatting failures from the sink.
pub fn write_doc_into(doc: &Doc, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    out.write_str("{\"type\":\"doc\"")?;
    write_attrs(doc.attrs(), &schema.doc().attrs, out)?;
    out.write_str(",\"content\":[")?;
    for (index, block) in doc.blocks().iter().enumerate() {
        if index > 0 {
            out.write_char(',')?;
        }
        write_block(block, schema, out)?;
    }
    out.write_str("]}")
}

fn write_block(block: &Block, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    let Some(spec) = schema.node(block.kind()) else {
        return Err(fmt::Error);
    };
    out.write_str("{\"type\":")?;
    write_string(&spec.name, out)?;
    write_attrs(block.attrs(), &spec.attrs, out)?;
    match block.content() {
        BlockContent::Text(text) => {
            out.write_str(",\"content\":[")?;
            let mut first = true;
            for item in text.items() {
                match item {
                    InlineItem::Text(run) => {
                        if !first {
                            out.write_char(',')?;
                        }
                        first = false;
                        out.write_str("{\"type\":\"text\",\"text\":")?;
                        write_string(&run.text, out)?;
                        write_marks(&run.marks, schema, out)?;
                        out.write_char('}')?;
                    }
                    InlineItem::Atom(atom) => {
                        if !first {
                            out.write_char(',')?;
                        }
                        first = false;
                        write_inline_atom(atom, schema, out)?;
                    }
                }
            }
            out.write_char(']')?;
        }
        BlockContent::Blocks(children) => {
            out.write_str(",\"content\":[")?;
            for (index, child) in children.iter().enumerate() {
                if index > 0 {
                    out.write_char(',')?;
                }
                write_block(child, schema, out)?;
            }
            out.write_char(']')?;
        }
        BlockContent::Atom(_) => {}
    }
    out.write_char('}')
}

fn write_inline_atom(
    atom: &crate::model::InlineAtom,
    schema: &Schema,
    out: &mut impl Write,
) -> fmt::Result {
    let Some(spec) = schema.node(atom.kind) else {
        return Err(fmt::Error);
    };
    out.write_str("{\"type\":")?;
    write_string(&spec.name, out)?;
    write_attrs(&atom.attrs, &spec.attrs, out)?;
    out.write_char('}')
}

fn write_attrs(attrs: &Attrs, specs: &[AttrSpec], out: &mut impl Write) -> fmt::Result {
    if attrs.is_empty() {
        return Ok(());
    }
    out.write_str(",\"attrs\":{")?;
    let mut first = true;
    for attr in attrs {
        let Some(spec) = specs.iter().find(|spec| spec.id == attr.id) else {
            return Err(fmt::Error);
        };
        let name = &spec.name;
        if !first {
            out.write_char(',')?;
        }
        first = false;
        write_string(name, out)?;
        out.write_char(':')?;
        write_value(&attr.value, out)?;
    }
    out.write_char('}')
}

fn write_value(value: &AttrValue, out: &mut impl Write) -> fmt::Result {
    match value {
        AttrValue::Null => out.write_str("null"),
        AttrValue::Bool(value) => out.write_str(if *value { "true" } else { "false" }),
        AttrValue::Int(value) => write!(out, "{value}"),
        AttrValue::Str(value) => write_string(value, out),
        AttrValue::Array(values) => {
            out.write_char('[')?;
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.write_char(',')?;
                }
                write_value(value, out)?;
            }
            out.write_char(']')
        }
        AttrValue::Ref(target) => {
            out.write_str("{\"doc\":")?;
            write_string(target.doc.as_str(), out)?;
            if let Some(node) = &target.node {
                out.write_str(",\"node\":")?;
                write_string(node.as_str(), out)?;
            }
            if let Some(version) = &target.version {
                out.write_str(",\"version\":")?;
                write_string(version.as_str(), out)?;
            }
            if let Some(fragment) = &target.fragment {
                out.write_str(",\"fragment\":{\"start\":")?;
                write!(out, "{}", fragment.start.0)?;
                out.write_str(",\"end\":")?;
                write!(out, "{}", fragment.end.0)?;
                out.write_char('}')?;
            }
            out.write_char('}')
        }
    }
}

fn write_marks(marks: &MarkSet, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    if marks.is_empty() {
        return Ok(());
    }
    out.write_str(",\"marks\":[")?;
    let mut first = true;
    for mark in schema.marks() {
        if !marks.contains(mark.id) {
            continue;
        }
        if !first {
            out.write_char(',')?;
        }
        first = false;
        out.write_str("{\"type\":")?;
        write_string(&mark.name, out)?;
        if let Some(attrs) = marks.attrs_for(mark.id) {
            write_attrs(attrs, &mark.attrs, out)?;
        }
        out.write_char('}')?;
    }
    out.write_char(']')
}

#[allow(
    clippy::redundant_pub_crate,
    reason = "re-exported for the a11y JSON writer"
)]
pub(crate) fn write_string(value: &str, out: &mut impl Write) -> fmt::Result {
    out.write_char('"')?;
    for character in value.chars() {
        match character {
            '"' => out.write_str("\\\"")?,
            '\\' => out.write_str("\\\\")?,
            '\u{0008}' => out.write_str("\\b")?,
            '\u{000c}' => out.write_str("\\f")?,
            '\n' => out.write_str("\\n")?,
            '\r' => out.write_str("\\r")?,
            '\t' => out.write_str("\\t")?,
            value if u32::from(value) < 0x20 => {
                write!(out, "\\u{:04x}", u32::from(value))?;
            }
            value => out.write_char(value)?,
        }
    }
    out.write_char('"')
}
