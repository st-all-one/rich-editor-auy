//! Stateful WASM editor exposing atomic batches and structured import.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "the wasm-bindgen JS class shim expands to Rc and panic!; the core path stays panic-free"
)]

use std::sync::Arc;

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor as CoreEditor, EditorOptions, Origin};
use rich_editor_auy::wire::{Envelope, ParseOptions, from_json as parse_document};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::batch;
use crate::convert::{applied_json, empty_applied_json, history_value, state_json, version_u32};
use crate::dto::EditorOptionsJson;
use crate::ops;
use crate::problem;

mod commands;
mod history;
mod import;
mod methods;
mod positions;
mod tx;

/// Converts a problem into the rejection value thrown across the WASM boundary.
fn problem_js(problem: &ProblemDetails) -> JsValue {
    JsValue::from_str(&problem::problem_json(problem))
}

/// Maps a history origin name to the core enum.
fn batch_origin(name: Option<&str>) -> Result<Origin, JsValue> {
    match name {
        None | Some("api") => Ok(Origin::Api),
        Some("user") => Ok(Origin::User),
        Some("paste") => Ok(Origin::Paste),
        Some("remote") => Ok(Origin::Remote),
        Some("import") => Ok(Origin::Import),
        Some(other) => Err(problem_js(&problem::unknown_origin(other))),
    }
}

/// Parses construction options from JSON text.
fn editor_options(options: Option<String>) -> Result<EditorOptions, JsValue> {
    let Some(options) = options else {
        return Ok(EditorOptions::default());
    };
    let parsed: EditorOptionsJson = serde_json::from_str(&options)
        .map_err(|error| problem_js(&problem::malformed_problem(format!("{error}"), "options")))?;
    Ok(EditorOptions {
        history_limits: parsed.history_limits,
    })
}

/// Builds the single public schema.
fn full_schema() -> Result<Arc<Schema>, JsValue> {
    Schema::full()
        .map(Arc::new)
        .map_err(|error| problem_js(&problem::from_schema_error(&error)))
}

/// Stateful editor exposed to JavaScript.
///
/// Every mutation is atomic: the batch is applied to a clone and committed only when every
/// operation succeeds, so a rejected batch leaves the editor byte-for-byte unchanged.
#[wasm_bindgen]
pub struct Editor {
    inner: CoreEditor,
    schema: Arc<Schema>,
}

#[wasm_bindgen]
impl Editor {
    /// Creates an empty editor using the complete schema.
    ///
    /// # Errors
    /// Returns a problem object only if the full preset is inconsistent.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        let schema = full_schema()?;
        let inner =
            CoreEditor::new().map_err(|error| problem_js(&problem::from_schema_error(&error)))?;
        Ok(Self { inner, schema })
    }

    /// Creates an editor from canonical document JSON.
    ///
    /// `options` is JSON with optional `historyLimits: [maxGroups, maxBytes]`.
    ///
    /// # Errors
    /// Returns a problem object when the document or the options are rejected.
    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json(input: &str, options: Option<String>) -> Result<Self, JsValue> {
        let schema = full_schema()?;
        let doc = parse_document(input, &schema, &ParseOptions::default())
            .map_err(|error| problem_js(&problem::from_parse_error(&error)))?;
        let inner = CoreEditor::from_doc_with_options(doc, editor_options(options)?)
            .map_err(|error| problem_js(&problem::from_schema_error(&error)))?;
        Ok(Self { inner, schema })
    }

    /// Creates an editor from a durable envelope.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the envelope format, fingerprint, or document is invalid.
    #[wasm_bindgen(js_name = fromEnvelope)]
    pub fn from_envelope(input: &str, options: Option<String>) -> Result<Self, JsValue> {
        let schema = full_schema()?;
        let envelope = Envelope::from_json(input, &schema, &ParseOptions::default())
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        let inner = CoreEditor::from_doc_with_options(envelope.doc, editor_options(options)?)
            .map_err(|error| problem_js(&problem::from_schema_error(&error)))?;
        Ok(Self { inner, schema })
    }

    /// Rejects a batch computed against another document version.
    fn check_base(&self, base: Option<u32>) -> Result<(), JsValue> {
        match base {
            Some(base) if base != self.version() => {
                Err(problem_js(&problem::stale_problem(base, self.version())))
            }
            _ => Ok(()),
        }
    }

    /// Returns the canonical document JSON.
    #[wasm_bindgen(js_name = docJSON)]
    #[must_use]
    pub fn doc_json(&self) -> String {
        self.inner.doc().to_json_string(&self.schema)
    }

    /// Returns `{doc, selection, version}`.
    #[wasm_bindgen(js_name = stateJSON)]
    #[must_use]
    pub fn state_json(&self) -> String {
        state_json(&self.doc_json(), self.inner.selection(), self.version())
    }

    /// Returns the current document version.
    #[must_use]
    pub fn version(&self) -> u32 {
        version_u32(self.inner.version())
    }

    /// Returns the active schema fingerprint.
    #[wasm_bindgen(js_name = schemaFingerprint)]
    #[must_use]
    pub fn schema_fingerprint(&self) -> String {
        self.schema.fingerprint().to_owned()
    }

    /// Returns the observable history state.
    #[wasm_bindgen(js_name = historyJSON)]
    #[must_use]
    pub fn history_json(&self) -> String {
        history_value(self.inner.history_info()).to_string()
    }

    /// Returns the number of top-level blocks.
    #[must_use]
    pub fn blocks(&self) -> u32 {
        ops::block_count(&self.inner)
    }

    /// Applies `{"ops": [...]}` atomically.
    ///
    /// # Errors
    ///
    /// Returns a problem object when any operation is rejected.
    pub fn apply(&mut self, payload: &str) -> Result<String, JsValue> {
        let batch = ops::parse_batch(payload).map_err(|problem| problem_js(&problem))?;
        self.check_base(batch.base_version)?;
        let origin = batch_origin(batch.origin.as_deref())?;
        let mut working = self.inner.clone();
        let meta = batch::BatchMeta {
            origin,
            group: batch.group,
            add_to_history: batch.add_to_history,
        };
        match batch::run_ops_with(&mut working, &self.schema, &batch.ops, &meta) {
            Ok(info) => {
                self.inner = working;
                Ok(match info {
                    Some(info) => applied_json(&info),
                    None => empty_applied_json(
                        self.inner.selection(),
                        self.version(),
                        self.inner.history_info(),
                    ),
                })
            }
            Err(problem) => Err(problem_js(&problem)),
        }
    }
}
