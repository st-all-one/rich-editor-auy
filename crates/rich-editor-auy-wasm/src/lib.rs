//! WASM boundary for `rich-editor-auy`.
//!
//! This crate is not the core. It exposes a synchronous string/JSON API and keeps all
//! schema, normalization, and step semantics inside the Rust core. Every failure crosses the
//! boundary as an RFC 9457 [`ProblemDetails`] JSON object rather than an opaque string.
//!
//! The stateful [`Editor`] is the primary surface; the free functions remain as stateless
//! conversions for hosts that only need one-shot canonicalization.

#![allow(
    clippy::excessive_nesting,
    clippy::redundant_pub_crate,
    reason = "explicit operation matching and crate-private helpers keep the boundary visible"
)]

mod attr;
mod convert;
mod dto;
mod editor;
mod import;
mod ops;
mod problem;
mod problem_map;
mod stage;
#[cfg(test)]
mod tests;

pub use editor::Editor;

use std::sync::Arc;

use rich_editor_auy::error::{EditError, ProblemDetails, StepError};
use rich_editor_auy::model::{Doc, Mark};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::pos::Range;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{EditorState, Transaction};
use rich_editor_auy::wire::{Envelope, ParseOptions, from_json as parse_document};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

/// Returns the crate version exposed to JavaScript consumers.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Returns the canonical BLAKE3 fingerprint of the full schema.
///
/// # Errors
///
/// Returns a problem object only if the full preset is inconsistent.
#[wasm_bindgen]
pub fn schema_fingerprint() -> Result<String, JsValue> {
    let schema = full_schema()?;
    Ok(schema.fingerprint().to_owned())
}

/// Canonicalizes a document JSON string.
///
/// # Errors
///
/// Returns a problem object when the input is not a document accepted by the full schema.
#[wasm_bindgen]
pub fn canonicalize(input: &str) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let doc = decode(input, &schema)?;
    Ok(doc.to_json_string(&schema))
}

/// Returns the plain text of a document.
///
/// # Errors
///
/// Returns a problem object when the input is not a document accepted by the full schema.
#[wasm_bindgen]
pub fn plain_text(input: &str) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let doc = decode(input, &schema)?;
    Ok(doc.plain_text())
}

/// Inserts text at a byte position and returns canonical JSON.
///
/// # Errors
///
/// Returns a problem object when the position or input is invalid.
#[wasm_bindgen]
pub fn insert_text(input: &str, position: u32, text: &str) -> Result<String, JsValue> {
    edit(input, |transaction| {
        transaction.insert_text(Pos(position), text)
    })
}

/// Returns stable accessibility error codes, or `ok`.
///
/// # Errors
///
/// Returns a problem object when the input is not a document accepted by the full schema.
#[wasm_bindgen]
pub fn validate_a11y(input: &str) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let doc = decode(input, &schema)?;
    let report = doc.validate_a11y(&schema);
    if report.is_ok() {
        return Ok(String::from("ok"));
    }
    let codes = report
        .errors
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>()
        .join(",");
    Ok(codes)
}

/// Returns the semantic accessibility tree as canonical JSON.
///
/// # Errors
///
/// Returns a problem object when the input is not a document accepted by the full schema.
#[wasm_bindgen]
pub fn a11y_tree(input: &str) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let doc = decode(input, &schema)?;
    Ok(doc.a11y_tree(&schema).to_json_string(&schema))
}

/// Splits the block at a byte position and returns canonical JSON.
///
/// # Errors
///
/// Returns a problem object when the position or input is invalid.
#[wasm_bindgen]
pub fn split_block(input: &str, position: u32) -> Result<String, JsValue> {
    edit(input, |transaction| transaction.split_block(Pos(position)))
}

/// Deletes a byte range and returns canonical JSON.
///
/// # Errors
///
/// Returns a problem object when the range or input is invalid.
#[wasm_bindgen]
pub fn delete_range(input: &str, from: u32, to: u32) -> Result<String, JsValue> {
    edit(input, |transaction| {
        transaction.delete(Range::new(Pos(from), Pos(to)))
    })
}

/// Toggles a named mark over a byte range and returns canonical JSON.
///
/// # Errors
///
/// Returns a problem object when the mark, range, or input is invalid.
#[wasm_bindgen]
pub fn toggle_mark(input: &str, from: u32, to: u32, mark: &str) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let id = schema
        .mark_id(mark)
        .ok_or_else(|| problem_js(&problem::token_problem(mark, "mark")))?;
    let doc = decode(input, &schema)?;
    let range = Range::new(Pos(from), Pos(to));
    let active = !range.is_empty() && attr::range_has_mark(&doc, range, id);
    let schema = Arc::new(schema);
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    let result = if active {
        transaction.remove_mark(range, Mark::new(id))
    } else {
        transaction.add_mark(range, Mark::new(id))
    };
    result.map_err(|error| step_js(&error))?;
    let applied = state
        .apply_owned(transaction)
        .map_err(|error| edit_js(&error))?;
    Ok(applied.state.doc().to_json_string(&schema))
}

/// Joins the block at a byte boundary with its previous sibling.
///
/// # Errors
///
/// Returns a problem object when the position or input is invalid.
#[wasm_bindgen]
pub fn join_backward(input: &str, position: u32) -> Result<String, JsValue> {
    edit(input, |transaction| {
        transaction.join_backward(Pos(position))
    })
}

/// Validates and canonicalizes a durable envelope, returning its document JSON.
///
/// # Errors
///
/// Returns a problem object when the envelope format, fingerprint, or document is invalid.
#[wasm_bindgen]
pub fn envelope_doc(input: &str) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let envelope = Envelope::from_json(input, &schema, &ParseOptions::default())
        .map_err(|error| edit_js(&error))?;
    Ok(envelope.doc.to_json_string(&schema))
}

/// Validates and canonicalizes a durable envelope, returning canonical envelope JSON.
///
/// # Errors
///
/// Returns a problem object when the envelope format, fingerprint, or document is invalid.
#[wasm_bindgen]
pub fn canonicalize_envelope(input: &str) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let envelope = Envelope::from_json(input, &schema, &ParseOptions::default())
        .map_err(|error| edit_js(&error))?;
    envelope
        .to_json_string(&schema)
        .map_err(|error| problem_js(&problem::malformed_problem(format!("{error}"), "envelope")))
}

/// Applies one transaction and returns canonical document JSON.
fn edit(
    input: &str,
    operation: impl FnOnce(&mut Transaction) -> Result<(), StepError>,
) -> Result<String, JsValue> {
    let schema = full_schema()?;
    let doc = decode(input, &schema)?;
    let schema = Arc::new(schema);
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    operation(&mut transaction).map_err(|error| step_js(&error))?;
    let applied = state
        .apply_owned(transaction)
        .map_err(|error| edit_js(&error))?;
    Ok(applied.state.doc().to_json_string(&schema))
}

/// Builds the single public schema.
fn full_schema() -> Result<Schema, JsValue> {
    Schema::full().map_err(|error| problem_js(&problem::from_schema_error(&error)))
}

/// Decodes a document token tree.
fn decode(input: &str, schema: &Schema) -> Result<Doc, JsValue> {
    parse_document(input, schema, &ParseOptions::default())
        .map_err(|error| problem_js(&problem::from_parse_error(&error)))
}

/// Converts a problem into the rejection value thrown across the WASM boundary.
fn problem_js(problem: &ProblemDetails) -> JsValue {
    JsValue::from_str(&problem::problem_json(problem))
}

/// Converts a step failure into the rejection value.
fn step_js(error: &StepError) -> JsValue {
    problem_js(&problem::from_step_error(error))
}

/// Converts an editing failure into the rejection value.
fn edit_js(error: &EditError) -> JsValue {
    problem_js(&problem::from_edit_error(error))
}
