//! WASM boundary for `rich-editor-auy`.
//!
//! This crate is not the core. It exposes a small synchronous string/JSON API and keeps all
//! schema, normalization, and step semantics inside the Rust core.

use core::fmt::Display;
use std::sync::Arc;

use rich_editor_auy::error::StepError;
use rich_editor_auy::model::{Doc, Mark};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::pos::Range;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{EditorState, Transaction};
use rich_editor_auy::wire::{Envelope, ParseOptions, from_json};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

/// Returns the crate version exposed to JavaScript consumers.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Returns the canonical BLAKE3 schema fingerprint.
#[wasm_bindgen]
pub fn schema_fingerprint() -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    Ok(schema.fingerprint().to_owned())
}

/// Canonicalizes a document JSON string.
#[wasm_bindgen]
pub fn canonicalize(input: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let doc = decode(input, &schema)?;
    Ok(doc.to_json_string(&schema))
}

/// Returns the plain text of a document.
#[wasm_bindgen]
pub fn plain_text(input: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let doc = decode(input, &schema)?;
    Ok(doc.plain_text())
}

/// Inserts text at a byte position and returns canonical JSON.
#[wasm_bindgen]
pub fn insert_text(input: &str, position: u32, text: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let doc = decode(input, &schema)?;
    let schema = Arc::new(schema);
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    transaction
        .insert_text(Pos(position), text)
        .map_err(error)?;
    let applied = state.apply(transaction).map_err(error)?;
    Ok(applied.state.doc().to_json_string(&schema))
}

/// Returns stable accessibility error codes, or `ok`.
#[wasm_bindgen]
pub fn validate_a11y(input: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
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
#[wasm_bindgen]
pub fn a11y_tree(input: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let doc = decode(input, &schema)?;
    Ok(doc.a11y_tree(&schema).to_json_string(&schema))
}

/// Splits the block at a byte position and returns canonical JSON.
#[wasm_bindgen]
pub fn split_block(input: &str, position: u32) -> Result<String, JsValue> {
    edit(input, |transaction| transaction.split_block(Pos(position)))
}

/// Deletes a byte range and returns canonical JSON.
#[wasm_bindgen]
pub fn delete_range(input: &str, from: u32, to: u32) -> Result<String, JsValue> {
    edit(input, |transaction| {
        transaction.delete(Range::new(Pos(from), Pos(to)))
    })
}

/// Toggles a named mark over a byte range and returns canonical JSON.
#[wasm_bindgen]
pub fn toggle_mark(input: &str, from: u32, to: u32, mark: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let id = schema.mark_id(mark).ok_or_else(|| error("unknown mark"))?;
    let doc = decode(input, &schema)?;
    let schema = Arc::new(schema);
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    transaction
        .add_mark(Range::new(Pos(from), Pos(to)), Mark::new(id))
        .map_err(error)?;
    let applied = state.apply(transaction).map_err(error)?;
    Ok(applied.state.doc().to_json_string(&schema))
}

/// Joins the block at a byte boundary with its previous sibling.
#[wasm_bindgen]
pub fn join_backward(input: &str, position: u32) -> Result<String, JsValue> {
    edit(input, |transaction| {
        transaction.join_backward(Pos(position))
    })
}

fn edit(
    input: &str,
    operation: impl FnOnce(&mut Transaction) -> Result<(), StepError>,
) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let doc = decode(input, &schema)?;
    let schema = Arc::new(schema);
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    operation(&mut transaction).map_err(error)?;
    let applied = state.apply(transaction).map_err(error)?;
    Ok(applied.state.doc().to_json_string(&schema))
}

/// Validates and canonicalizes a durable envelope, returning its document JSON.
#[wasm_bindgen]
pub fn envelope_doc(input: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let envelope = Envelope::from_json(input, &schema, &ParseOptions::default()).map_err(error)?;
    Ok(envelope.doc.to_json_string(&schema))
}

/// Validates and canonicalizes a durable envelope, returning canonical envelope JSON.
#[wasm_bindgen]
pub fn canonicalize_envelope(input: &str) -> Result<String, JsValue> {
    let schema = Schema::basic().map_err(error)?;
    let envelope = Envelope::from_json(input, &schema, &ParseOptions::default()).map_err(error)?;
    envelope.to_json_string(&schema).map_err(error)
}

fn decode(input: &str, schema: &Schema) -> Result<Doc, JsValue> {
    from_json(input, schema, &ParseOptions::default()).map_err(error)
}

fn error(value: impl Display) -> JsValue {
    JsValue::from_str(&value.to_string())
}
