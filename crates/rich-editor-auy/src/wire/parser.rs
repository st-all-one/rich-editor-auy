//! Canonical document decoder.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::Limits;
use crate::error::ParseError;
use crate::model::{Attrs, Block, Doc, InlineAtom, InlineItem, Run, TextBlock};
use crate::schema::Schema;

use super::attrs::{DecodePath, decode_attrs};
use super::budget::{DecodeBudget, limit_error};
use super::json::JsonValue;
use super::json_parser::parse_json;
use super::marks::decode_marks;
use super::options::ParseOptions;

/// Decodes a document node.
///
/// # Errors
///
/// Returns a located parse error.
pub fn from_json(input: &str, schema: &Schema, options: &ParseOptions) -> Result<Doc, ParseError> {
    let value = parse_json(input, options.limits.max_depth)?;
    decode_doc(&value, schema, options)
}

/// Parses strict JSON syntax without decoding a document.
///
/// This is the raw parser path used by envelope preflight and by the Serde A/B benchmark.
///
/// # Errors
///
/// Returns a located error for malformed JSON, duplicate keys, floats, or exponents.
pub fn check_json(input: &str, options: &ParseOptions) -> Result<(), ParseError> {
    parse_json(input, options.limits.max_depth).map(|_value| ())
}

pub(super) fn path_string(path: &[u32]) -> Box<str> {
    let mut out = String::new();
    for (index, component) in path.iter().enumerate() {
        if index > 0 {
            out.push('.');
        }
        out.push_str(&component.to_string());
    }
    out.into_boxed_str()
}

pub(super) fn decode_doc(
    value: &JsonValue,
    schema: &Schema,
    options: &ParseOptions,
) -> Result<Doc, ParseError> {
    let node_type = value
        .get("type")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| ParseError::new("missing_type", 0))?;
    if node_type != "doc" {
        return Err(ParseError::new("not_a_document", 0));
    }
    let attrs = decode_attrs(
        value.get("attrs"),
        &schema.doc().attrs,
        DecodePath::DocAttrs,
        &options.limits,
    )?;
    let blocks = match value.get("content") {
        Some(content) => {
            let values = content
                .as_array()
                .ok_or_else(|| ParseError::new("content_not_array", 0))?;
            decode_blocks(values, schema, options)?
        }
        None => Vec::new(),
    };
    check_doc(Doc::from_decoded(attrs, blocks), schema, 0)
}

/// Decodes a block sequence, using array indices as stable error paths.
pub(super) fn decode_blocks(
    values: &[JsonValue],
    schema: &Schema,
    options: &ParseOptions,
) -> Result<Vec<Arc<Block>>, ParseError> {
    let mut blocks = Vec::with_capacity(values.len());
    let mut path = Vec::new();
    let mut budget = DecodeBudget::default();
    for (index, child) in values.iter().enumerate() {
        path.push(u32::try_from(index).unwrap_or(u32::MAX));
        let decoded = decode_block(child, schema, options, &mut path, 0, &mut budget)?;
        budget.count_block(decoded.size().0, &options.limits, child.start(), &path)?;
        path.pop();
        blocks.push(Arc::new(decoded));
    }
    Ok(blocks)
}

/// Validates a constructed document against the schema.
pub(super) fn check_doc(doc: Doc, schema: &Schema, offset: u32) -> Result<Doc, ParseError> {
    let report = doc.validate(schema);
    if let Some(issue) = report.issues.first() {
        return Err(ParseError {
            code: Box::from(issue.invariant),
            offset: issue.position.map_or(offset, |position| position.0),
            path: issue.path.as_str().into(),
        });
    }
    Ok(doc)
}

#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "single-pass block decoder threading decode state"
)]
fn decode_block(
    value: &JsonValue,
    schema: &Schema,
    options: &ParseOptions,
    path: &mut Vec<u32>,
    depth: u16,
    budget: &mut DecodeBudget,
) -> Result<Block, ParseError> {
    if depth > options.limits.max_depth {
        return Err(ParseError {
            code: Box::from("depth_limit"),
            offset: value.start(),
            path: path_string(path),
        });
    }
    budget.count_node(&options.limits, value.start(), path)?;
    let node_type = value
        .get("type")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| ParseError {
            code: Box::from("missing_type"),
            offset: value.start(),
            path: path_string(path),
        })?;
    let kind = schema.node_kind(node_type).ok_or_else(|| ParseError {
        code: Box::from("unknown_node"),
        offset: value.start(),
        path: path_string(path),
    })?;
    let spec = schema.node(kind).ok_or_else(|| ParseError {
        code: Box::from("unknown_node"),
        offset: value.start(),
        path: path_string(path),
    })?;
    let attrs = decode_attrs(
        value.get("attrs"),
        &spec.attrs,
        DecodePath::Node(path),
        &options.limits,
    )?;
    let children = value.get("content").and_then(JsonValue::as_array);
    if spec.atom {
        if children.is_some() {
            return Err(ParseError {
                code: Box::from("atom_with_content"),
                offset: value.start(),
                path: path_string(path),
            });
        }
        return Ok(Block::atom(
            kind,
            attrs,
            crate::model::AtomData {
                attrs: Attrs::empty(),
            },
        ));
    }
    let Some(children) = children else {
        return Ok(Block::text(kind, attrs, TextBlock::empty()));
    };
    // The content model itself is checked by `Doc::validate` (I2); the decoder only needs to
    // know whether the children are inline to pick the representation.
    let mut inline = true;
    for child in children {
        let child_kind = child
            .get("type")
            .and_then(JsonValue::as_str)
            .and_then(|name| schema.node_kind(name))
            .ok_or_else(|| ParseError {
                code: Box::from("unknown_node"),
                offset: child.start(),
                path: path_string(path),
            })?;
        inline = inline
            && schema
                .node(child_kind)
                .is_some_and(|child_spec| child_spec.inline);
    }
    if inline {
        let mut items = Vec::with_capacity(children.len());
        let mut text_bytes = 0_u64;
        for child in children {
            let item = decode_inline(child, schema, path, &options.limits)?;
            match &item {
                InlineItem::Atom(_) => {
                    budget.count_node(&options.limits, child.start(), path)?;
                }
                InlineItem::Text(run) => {
                    text_bytes = text_bytes
                        .saturating_add(u64::try_from(run.text.len()).unwrap_or(u64::MAX));
                }
            }
            items.push(item);
        }
        if text_bytes > u64::from(options.limits.max_block_text_bytes) {
            return Err(limit_error(value.start(), path));
        }
        if u32::try_from(items.len()).unwrap_or(u32::MAX) > options.limits.max_runs_per_block {
            return Err(limit_error(value.start(), path));
        }
        let ceiling = u32::try_from(crate::model::MAX_RUN_BYTES)
            .unwrap_or(4096)
            .min(options.limits.max_block_text_bytes);
        let mut text = TextBlock::new(items);
        text.normalize(ceiling).map_err(|_| ParseError {
            code: Box::from("run_limit"),
            offset: value.start(),
            path: path_string(path),
        })?;
        return Ok(Block::text(kind, attrs, text));
    }
    let mut blocks = Vec::with_capacity(children.len());
    for (index, child) in children.iter().enumerate() {
        path.push(u32::try_from(index).unwrap_or(u32::MAX));
        let decoded = decode_block(
            child,
            schema,
            options,
            path,
            depth.saturating_add(1),
            budget,
        )?;
        path.pop();
        blocks.push(Arc::new(decoded));
    }
    Ok(Block::container(kind, attrs, blocks))
}

pub(super) fn decode_inline(
    value: &JsonValue,
    schema: &Schema,
    path: &[u32],
    limits: &Limits,
) -> Result<InlineItem, ParseError> {
    let node_type = value
        .get("type")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| ParseError {
            code: Box::from("missing_type"),
            offset: value.start(),
            path: path_string(path),
        })?;
    if node_type == "text" {
        let text = value
            .get("text")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| ParseError {
                code: Box::from("missing_text"),
                offset: value.start(),
                path: path_string(path),
            })?;
        if u32::try_from(text.len()).unwrap_or(u32::MAX) > limits.max_block_text_bytes {
            return Err(limit_error(value.start(), path));
        }
        let marks = decode_marks(value.get("marks"), schema, limits)?;
        return Ok(InlineItem::Text(Run::new(String::from(text), marks)));
    }
    let kind = schema.node_kind(node_type).ok_or_else(|| ParseError {
        code: Box::from("unknown_inline"),
        offset: value.start(),
        path: path_string(path),
    })?;
    let spec = schema.node(kind).ok_or_else(|| ParseError {
        code: Box::from("unknown_inline"),
        offset: value.start(),
        path: path_string(path),
    })?;
    let attrs = decode_attrs(
        value.get("attrs"),
        &spec.attrs,
        DecodePath::Node(path),
        limits,
    )?;
    Ok(InlineItem::Atom(InlineAtom { kind, attrs }))
}
