//! Canonical document decoder.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::ParseError;
use crate::model::{Attrs, Block, Doc, InlineAtom, InlineItem, Run, TextBlock};
use crate::schema::Schema;

use super::attrs::{DecodePath, decode_attrs, decode_marks};
use super::json::JsonValue;
use super::json_parser::parse_json;
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

fn path_string(path: &[u32]) -> Box<str> {
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
    )?;
    let mut blocks = Vec::new();
    let mut path = Vec::new();
    if let Some(content) = value.get("content") {
        let values = content
            .as_array()
            .ok_or_else(|| ParseError::new("content_not_array", 0))?;
        for (index, child) in values.iter().enumerate() {
            path.push(u32::try_from(index).unwrap_or(u32::MAX));
            blocks.push(alloc::sync::Arc::new(decode_block(
                child, schema, options, &mut path, 0,
            )?));
            path.pop();
        }
    }
    let doc = Doc::new(attrs, blocks, &options.limits).map_err(|_error| ParseError {
        code: Box::from("limit"),
        offset: value.start(),
        path: Box::from("doc"),
    })?;
    let report = doc.validate(schema);
    if let Some(issue) = report.issues.first() {
        return Err(ParseError {
            code: Box::from(issue.invariant),
            offset: issue.position.map_or(0, |position| position.0),
            path: issue.path.as_str().into(),
        });
    }
    Ok(doc)
}

#[allow(clippy::too_many_lines, reason = "single-pass block decoder")]
fn decode_block(
    value: &JsonValue,
    schema: &Schema,
    options: &ParseOptions,
    path: &mut Vec<u32>,
    depth: u16,
) -> Result<Block, ParseError> {
    if depth > options.limits.max_depth {
        return Err(ParseError {
            code: Box::from("depth_limit"),
            offset: value.start(),
            path: path_string(path),
        });
    }
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
    let attrs = decode_attrs(value.get("attrs"), &spec.attrs, DecodePath::Node(path))?;
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
    let inline = if children.len() == 1 {
        let Some(child) = children.first() else {
            return Ok(Block::text(kind, attrs, TextBlock::empty()));
        };
        let child_kind = child
            .get("type")
            .and_then(JsonValue::as_str)
            .and_then(|name| schema.node_kind(name))
            .ok_or_else(|| ParseError {
                code: Box::from("unknown_node"),
                offset: child.start(),
                path: path_string(path),
            })?;
        schema
            .node(child_kind)
            .is_some_and(|child_spec| child_spec.inline)
            && schema.content_allows(kind, &[child_kind])
    } else {
        let child_kinds = children
            .iter()
            .map(|child| {
                child
                    .get("type")
                    .and_then(JsonValue::as_str)
                    .and_then(|name| schema.node_kind(name))
                    .ok_or_else(|| ParseError {
                        code: Box::from("unknown_node"),
                        offset: child.start(),
                        path: path_string(path),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        child_kinds.iter().all(|child| {
            schema
                .node(*child)
                .is_some_and(|child_spec| child_spec.inline)
        }) && schema.content_allows(kind, &child_kinds)
    };
    if inline {
        let mut items = Vec::new();
        for child in children {
            items.push(decode_inline(child, schema, path)?);
        }
        let mut text = TextBlock::new(items);
        text.normalize(options.limits.max_block_text_bytes)
            .map_err(|_| ParseError {
                code: Box::from("run_limit"),
                offset: value.start(),
                path: path_string(path),
            })?;
        return Ok(Block::text(kind, attrs, text));
    }
    let mut blocks = Vec::new();
    for (index, child) in children.iter().enumerate() {
        path.push(u32::try_from(index).unwrap_or(u32::MAX));
        let decoded = decode_block(child, schema, options, path, depth.saturating_add(1))?;
        path.pop();
        blocks.push(alloc::sync::Arc::new(decoded));
    }
    Ok(Block::container(kind, attrs, blocks))
}

fn decode_inline(
    value: &JsonValue,
    schema: &Schema,
    path: &[u32],
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
        let marks = decode_marks(value.get("marks"), schema)?;
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
    let attrs = decode_attrs(value.get("attrs"), &spec.attrs, DecodePath::Node(path))?;
    Ok(InlineItem::Atom(InlineAtom { kind, attrs }))
}
