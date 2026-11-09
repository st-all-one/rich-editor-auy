//! Stateful WASM editor exposing atomic batches and structured import.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "the wasm-bindgen JS class shim expands to Rc and panic!; the core path stays panic-free"
)]

use std::sync::Arc;

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor as CoreEditor, Origin, Selection};
use rich_editor_auy::wire::{Envelope, ParseOptions, from_json as parse_document};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::convert::{
    ImportReport, applied_json, history_value, state_json, undo_json, version_u32,
};
use crate::dto::ImportPayload;
use crate::import;
use crate::ops;
use crate::problem;
use crate::stage;

mod view;

/// Converts a problem into the rejection value thrown across the WASM boundary.
fn problem_js(problem: &ProblemDetails) -> JsValue {
    JsValue::from_str(&problem::problem_json(problem))
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
    /// # Errors
    /// Returns a problem object when the document is rejected by the full schema.
    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json(input: &str) -> Result<Self, JsValue> {
        let schema = full_schema()?;
        let doc = parse_document(input, &schema, &ParseOptions::default())
            .map_err(|error| problem_js(&problem::from_parse_error(&error)))?;
        let inner = CoreEditor::from_doc(doc)
            .map_err(|error| problem_js(&problem::from_schema_error(&error)))?;
        Ok(Self { inner, schema })
    }

    /// Creates an editor from a durable envelope.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the envelope format, fingerprint, or document is invalid.
    #[wasm_bindgen(js_name = fromEnvelope)]
    pub fn from_envelope(input: &str) -> Result<Self, JsValue> {
        let schema = full_schema()?;
        let envelope = Envelope::from_json(input, &schema, &ParseOptions::default())
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        let inner = CoreEditor::from_doc(envelope.doc)
            .map_err(|error| problem_js(&problem::from_schema_error(&error)))?;
        Ok(Self { inner, schema })
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
        let mut working = self.inner.clone();
        match stage::run_ops(&mut working, &self.schema, &batch, Origin::Api) {
            Ok(info) => {
                self.inner = working;
                Ok(match info {
                    Some(info) => applied_json(&self.doc_json(), &info),
                    None => self.state_json(),
                })
            }
            Err(problem) => Err(problem_js(&problem)),
        }
    }

    /// Validates `{"ops": [...]}` without mutating the editor.
    ///
    /// # Errors
    ///
    /// Returns a problem object only when the payload is not valid JSON.
    #[wasm_bindgen(js_name = dryRun)]
    pub fn dry_run(&self, payload: &str) -> Result<String, JsValue> {
        let batch = ops::parse_batch(payload).map_err(|problem| problem_js(&problem))?;
        let mut working = self.inner.clone();
        let report = match stage::run_ops(&mut working, &self.schema, &batch, Origin::Api) {
            Ok(_) => ImportReport {
                accepted: true,
                blocks: ops::block_count(&working),
                version: self.version(),
                problems: Vec::new(),
            },
            Err(problem) => ImportReport {
                accepted: false,
                blocks: ops::block_count(&self.inner),
                version: self.version(),
                problems: vec![problem],
            },
        };
        let (doc_json, selection) = if report.accepted {
            (
                working.doc().to_json_string(&self.schema),
                working.selection(),
            )
        } else {
            (self.doc_json(), self.inner.selection())
        };
        Ok(report.to_json(&doc_json, selection))
    }

    /// Imports a document, fragment, or operation batch atomically.
    ///
    /// The returned report carries `accepted`, the resulting block count, the editor version,
    /// every `ProblemDetails`, and the current state.
    #[must_use]
    pub fn import(&mut self, payload: &str) -> String {
        let parsed = match serde_json::from_str::<ImportPayload>(payload) {
            Ok(parsed) => parsed,
            Err(error) => {
                let problem = problem::malformed_problem(format!("{error}"), "import");
                return self.import_failure(&problem);
            }
        };
        if let Some(schema) = &parsed.schema
            && schema.as_str() != self.schema.fingerprint()
        {
            let problem = problem::schema_mismatch_problem(self.schema.fingerprint(), schema);
            return self.import_failure(&problem);
        }
        if parsed.mode == "ops"
            && let Some(base) = parsed.base_version
            && base != self.version()
        {
            let problem = problem::stale_problem(base, self.version());
            return self.import_failure(&problem);
        }
        let mut working = self.inner.clone();
        match import::run_import(&mut working, &self.schema, &parsed) {
            Ok(()) => {
                self.inner = working;
                let report = ImportReport {
                    accepted: true,
                    blocks: ops::block_count(&self.inner),
                    version: self.version(),
                    problems: Vec::new(),
                };
                report.to_json(&self.doc_json(), self.inner.selection())
            }
            Err(problem) => self.import_failure(&problem),
        }
    }

    /// Undoes the latest group.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the inverse step no longer applies.
    pub fn undo(&mut self) -> Result<String, JsValue> {
        let applied = self
            .inner
            .undo()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(undo_json(
            &self.doc_json(),
            applied.as_ref(),
            self.inner.selection(),
            self.version(),
        ))
    }

    /// Redoes the latest undone group.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the step no longer applies.
    pub fn redo(&mut self) -> Result<String, JsValue> {
        let applied = self
            .inner
            .redo()
            .map_err(|error| problem_js(&problem::from_edit_error(&error)))?;
        Ok(undo_json(
            &self.doc_json(),
            applied.as_ref(),
            self.inner.selection(),
            self.version(),
        ))
    }

    /// Sets the selection without creating history.
    ///
    /// # Errors
    ///
    /// Returns a problem object when a position lies beyond the document.
    #[wasm_bindgen(js_name = setSelection)]
    pub fn set_selection(&mut self, from: u32, to: u32) -> Result<String, JsValue> {
        let selection = if from == to {
            Selection::caret(Pos(from))
        } else {
            Selection::text(Pos(from), Pos(to))
        };
        self.inner
            .set_selection(selection)
            .map_err(|error| problem_js(&problem::from_selection_error(&error)))?;
        Ok(self.state_json())
    }

    /// Discards every undo and redo group.
    #[wasm_bindgen(js_name = clearHistory)]
    pub fn clear_history(&mut self) {
        self.inner.clear_history();
    }

    /// Builds a rejected import report without mutating the editor.
    fn import_failure(&self, problem: &ProblemDetails) -> String {
        let report = ImportReport {
            accepted: false,
            blocks: ops::block_count(&self.inner),
            version: self.version(),
            problems: vec![problem.clone()],
        };
        report.to_json(&self.doc_json(), self.inner.selection())
    }
}
