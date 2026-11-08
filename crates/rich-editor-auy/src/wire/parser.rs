//! Canonical document decoder.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::error::ParseError;
use crate::model::{Attrs, Block, Doc, InlineAtom, InlineItem, Run, TextBlock};
use crate::schema::Schema;

use super::attrs::{decode_attrs, decode_marks};
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
    let attrs = decode_attrs(value.get("attrs"), &schema.doc().attrs, "doc.attrs")?;
    let mut blocks = Vec::new();
    if let Some(content) = value.get("content") {
        let values = content
            .as_array()
            .ok_or_else(|| ParseError::new("content_not_array", 0))?;
        for (index, child) in values.iter().enumerate() {
            blocks.push(alloc::sync::Arc::new(decode_block(
                child,
                schema,
                options,
                &alloc::format!("{index}"),
                0,
            )?));
        }
    }
    let doc = Doc::new(attrs, blocks, &options.limits).map_err(|_error| ParseError {
        code: Box::from("limit"),
        offset: 0,
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
    path: &str,
    depth: u16,
) -> Result<Block, ParseError> {
    if depth > options.limits.max_depth {
        return Err(ParseError {
            code: Box::from("depth_limit"),
            offset: 0,
            path: Box::from(path),
        });
    }
    let node_type = value
        .get("type")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| ParseError {
            code: Box::from("missing_type"),
            offset: 0,
            path: Box::from(path),
        })?;
    let kind = schema.node_kind(node_type).ok_or_else(|| ParseError {
        code: Box::from("unknown_node"),
        offset: 0,
        path: Box::from(path),
    })?;
    let spec = schema.node(kind).ok_or_else(|| ParseError {
        code: Box::from("unknown_node"),
        offset: 0,
        path: Box::from(path),
    })?;
    let attrs = decode_attrs(value.get("attrs"), &spec.attrs, path)?;
    let children = value.get("content").and_then(JsonValue::as_array);
    if spec.atom {
        if children.is_some() {
            return Err(ParseError {
                code: Box::from("atom_with_content"),
                offset: 0,
                path: Box::from(path),
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
    let child_kinds = children
        .iter()
        .map(|child| {
            child
                .get("type")
                .and_then(JsonValue::as_str)
                .and_then(|name| schema.node_kind(name))
                .ok_or_else(|| ParseError {
                    code: Box::from("unknown_node"),
                    offset: 0,
                    path: Box::from(path),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let inline = child_kinds.iter().all(|child| {
        schema
            .node(*child)
            .is_some_and(|child_spec| child_spec.inline)
    }) && schema.content_allows(kind, &child_kinds);
    if inline {
        let mut items = Vec::new();
        for child in children {
            items.push(decode_inline(child, schema, path)?);
        }
        let mut text = TextBlock::new(items);
        text.normalize(options.limits.max_block_text_bytes)
            .map_err(|_| ParseError {
                code: Box::from("run_limit"),
                offset: 0,
                path: Box::from(path),
            })?;
        return Ok(Block::text(kind, attrs, text));
    }
    let mut blocks = Vec::new();
    for (index, child) in children.iter().enumerate() {
        blocks.push(alloc::sync::Arc::new(decode_block(
            child,
            schema,
            options,
            &alloc::format!("{path}.{index}"),
            depth.saturating_add(1),
        )?));
    }
    Ok(Block::container(kind, attrs, blocks))
}

fn decode_inline(value: &JsonValue, schema: &Schema, path: &str) -> Result<InlineItem, ParseError> {
    let node_type = value
        .get("type")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| ParseError {
            code: Box::from("missing_type"),
            offset: 0,
            path: Box::from(path),
        })?;
    if node_type == "text" {
        let text = value
            .get("text")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| ParseError {
                code: Box::from("missing_text"),
                offset: 0,
                path: Box::from(path),
            })?;
        let marks = decode_marks(value.get("marks"), schema)?;
        return Ok(InlineItem::Text(Run::new(String::from(text), marks)));
    }
    let kind = schema.node_kind(node_type).ok_or_else(|| ParseError {
        code: Box::from("unknown_inline"),
        offset: 0,
        path: Box::from(path),
    })?;
    let spec = schema.node(kind).ok_or_else(|| ParseError {
        code: Box::from("unknown_inline"),
        offset: 0,
        path: Box::from(path),
    })?;
    let attrs = decode_attrs(value.get("attrs"), &spec.attrs, path)?;
    Ok(InlineItem::Atom(InlineAtom { kind, attrs }))
}
