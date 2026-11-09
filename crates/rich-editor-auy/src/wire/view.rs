//! Render-oriented `DocView` projection.
#![allow(
    clippy::excessive_nesting,
    reason = "writer mirrors the nested block grammar"
)]

use alloc::string::String;
use core::fmt::{self, Write};

use alloc::sync::Arc;

use crate::model::{Block, BlockContent, Doc, InlineItem, TextBlock};
use crate::schema::Schema;

use super::writer::{write_attrs, write_marks, write_string};

/// Writes the render projection for a document.
///
/// # Errors
///
/// Returns a formatting error when the sink fails or a block kind is absent from `schema`.
pub fn write_view_into(
    doc: &Doc,
    version: u64,
    schema: &Schema,
    out: &mut impl Write,
) -> fmt::Result {
    out.write_str("{\"version\":")?;
    write!(out, "{version}")?;
    write_attrs_field(doc.attrs(), &schema.doc().attrs, out)?;
    out.write_str(",\"blocks\":[")?;
    for (index, block) in doc.blocks().iter().enumerate() {
        if index > 0 {
            out.write_char(',')?;
        }
        let index = u32::try_from(index).map_err(|_| fmt::Error)?;
        let start = doc.block_range(index).map_or(0, |range| range.start.0);
        write_block(block, index, start, schema, out)?;
    }
    out.write_str("]}")
}

/// Returns the render projection as a string.
///
/// # Errors
///
/// Propagates formatting failures from the sink.
pub fn view_json(doc: &Doc, version: u64, schema: &Schema) -> Result<String, fmt::Error> {
    let mut out = String::new();
    write_view_into(doc, version, schema, &mut out)?;
    Ok(out)
}

fn write_block(
    block: &Block,
    index: u32,
    start: u32,
    schema: &Schema,
    out: &mut impl Write,
) -> fmt::Result {
    let Some(spec) = schema.node(block.kind()) else {
        return Err(fmt::Error);
    };
    out.write_str("{\"index\":")?;
    write!(out, "{index}")?;
    out.write_str(",\"kind\":")?;
    write_string(&spec.name, out)?;
    if let Some(id) = block.id() {
        out.write_str(",\"id\":")?;
        write_string(id.as_str(), out)?;
    }
    write_attrs_field(block.attrs(), &spec.attrs, out)?;
    write_range(start, start.saturating_add(block.size().0), out)?;
    match block.content() {
        BlockContent::Text(text) => write_text(text, start, schema, out)?,
        BlockContent::Blocks(children) => write_children(children, start, schema, out)?,
        BlockContent::Atom(_) => {}
    }
    out.write_char('}')
}

fn write_text(text: &TextBlock, start: u32, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    let mut value = String::new();
    text.text_into(&mut value);
    out.write_str(",\"text\":")?;
    write_string(&value, out)?;
    out.write_str(",\"runs\":[")?;
    write_runs(text, start, schema, out)?;
    out.write_str("],\"atoms\":[")?;
    write_atoms(text, start, schema, out)?;
    out.write_char(']')
}

fn write_runs(text: &TextBlock, start: u32, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    let mut offset = start;
    let mut first = true;
    for item in text.items() {
        if let InlineItem::Text(run) = item {
            if !first {
                out.write_char(',')?;
            }
            first = false;
            let end = offset.saturating_add(InlineItem::size(item));
            out.write_str("{\"text\":")?;
            write_string(&run.text, out)?;
            out.write_str(",\"range\":[")?;
            write!(out, "{offset},{end}")?;
            out.write_char(']')?;
            write_marks(&run.marks, schema, out)?;
            out.write_char('}')?;
        }
        offset = offset.saturating_add(InlineItem::size(item));
    }
    Ok(())
}

fn write_atoms(text: &TextBlock, start: u32, schema: &Schema, out: &mut impl Write) -> fmt::Result {
    let mut offset = start;
    let mut first = true;
    for item in text.items() {
        if let InlineItem::Atom(atom) = item {
            if !first {
                out.write_char(',')?;
            }
            first = false;
            let Some(spec) = schema.node(atom.kind) else {
                return Err(fmt::Error);
            };
            out.write_str("{\"kind\":")?;
            write_string(&spec.name, out)?;
            write_attrs_field(&atom.attrs, &spec.attrs, out)?;
            write_range(offset, offset.saturating_add(1), out)?;
            out.write_char('}')?;
        }
        offset = offset.saturating_add(InlineItem::size(item));
    }
    Ok(())
}

fn write_children(
    children: &[Arc<Block>],
    start: u32,
    schema: &Schema,
    out: &mut impl Write,
) -> fmt::Result {
    out.write_str(",\"children\":[")?;
    let mut offset = start.saturating_add(1);
    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            out.write_char(',')?;
        }
        let index = u32::try_from(index).map_err(|_| fmt::Error)?;
        write_block(child, index, offset, schema, out)?;
        offset = offset.saturating_add(child.size().0);
    }
    out.write_char(']')
}

fn write_range(start: u32, end: u32, out: &mut impl Write) -> fmt::Result {
    out.write_str(",\"range\":[")?;
    write!(out, "{start},{end}")?;
    out.write_char(']')
}

fn write_attrs_field(
    attrs: &crate::model::Attrs,
    specs: &[crate::schema::AttrSpec],
    out: &mut impl Write,
) -> fmt::Result {
    if attrs.is_empty() {
        return out.write_str(",\"attrs\":{}");
    }
    write_attrs(attrs, specs, out)
}
