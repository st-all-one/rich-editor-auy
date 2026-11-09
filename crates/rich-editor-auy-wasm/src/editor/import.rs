//! Structured import and dry-run validation.

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::state::Editor as CoreEditor;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::convert::ImportReport;
use crate::dto::ImportPayload;
use crate::import;
use crate::ops;
use crate::problem;

use super::{Editor, problem_js};

/// Host labels echoed back in every import report.
struct AuditLabels {
    request_id: Option<String>,
    source: Option<String>,
    destination: Option<String>,
}

impl AuditLabels {
    fn of(payload: &ImportPayload) -> Self {
        Self {
            request_id: payload.request_id.clone(),
            source: payload.source.clone(),
            destination: payload.destination.clone(),
        }
    }

    fn none() -> Self {
        Self {
            request_id: None,
            source: None,
            destination: None,
        }
    }
}

#[wasm_bindgen]
impl Editor {
    /// Validates an import payload without mutating the editor.
    ///
    /// Accepts the same envelope as `import`, including a bare `{"ops": [...]}` batch. A
    /// malformed envelope or a payload-level rejection (schema mismatch, stale version, byte
    /// limit) throws; content problems come back in the report with `accepted: false`.
    ///
    /// # Errors
    ///
    /// Returns a problem object when the envelope is not valid JSON or fails a precheck.
    #[wasm_bindgen(js_name = dryRun)]
    pub fn dry_run(&self, payload: &str) -> Result<String, JsValue> {
        let parsed = serde_json::from_str::<ImportPayload>(payload).map_err(|error| {
            problem_js(&problem::malformed_problem(format!("{error}"), "import"))
        })?;
        let audit = AuditLabels::of(&parsed);
        self.precheck_import(payload, &parsed)
            .map_err(|problem| problem_js(&problem))?;
        let mut working = self.inner.clone();
        match import::run_import(&mut working, &self.schema, &parsed) {
            Ok(outcome) => {
                if let Err(problem) =
                    import::check_limits(&working, &self.schema, parsed.options.limits.as_ref())
                {
                    return Ok(self.rejected_report(&problem, &audit));
                }
                Ok(self.accepted_report(&working, outcome, &audit))
            }
            Err(problem) => Ok(self.rejected_report(&problem, &audit)),
        }
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
                return self.rejected_report(&problem, &AuditLabels::none());
            }
        };
        let audit = AuditLabels::of(&parsed);
        if let Err(problem) = self.precheck_import(payload, &parsed) {
            return self.rejected_report(&problem, &audit);
        }
        let mut working = self.inner.clone();
        match import::run_import(&mut working, &self.schema, &parsed) {
            Ok(outcome) => {
                if let Err(problem) =
                    import::check_limits(&working, &self.schema, parsed.options.limits.as_ref())
                {
                    return self.rejected_report(&problem, &audit);
                }
                self.inner = working;
                self.accepted_report(&self.inner, outcome, &audit)
            }
            Err(problem) => self.rejected_report(&problem, &audit),
        }
    }

    /// Validates payload-level limits and version checks before applying anything.
    fn precheck_import(&self, payload: &str, parsed: &ImportPayload) -> Result<(), ProblemDetails> {
        import::check_byte_limit(payload, parsed.options.limits.as_ref())?;
        if let Some(schema) = &parsed.schema
            && schema.as_str() != self.schema.fingerprint()
        {
            return Err(problem::schema_mismatch_problem(
                self.schema.fingerprint(),
                schema,
            ));
        }
        if matches!(parsed.mode.as_str(), "" | "ops")
            && let Some(base) = parsed.base_version
            && base != self.version()
        {
            return Err(problem::stale_problem(base, self.version()));
        }
        Ok(())
    }

    /// Builds an accepted report from the working editor.
    fn accepted_report(
        &self,
        editor: &CoreEditor,
        outcome: import::ImportOutcome,
        audit: &AuditLabels,
    ) -> String {
        let report = ImportReport {
            accepted: true,
            blocks: ops::block_count(editor),
            accepted_blocks: outcome.accepted_blocks,
            version: self.version(),
            schema: String::from(self.schema.fingerprint()),
            request_id: audit.request_id.clone(),
            source: audit.source.clone(),
            destination: audit.destination.clone(),
            id_map: outcome.id_map,
            problems: Vec::new(),
        };
        report.to_json(
            &editor.doc().to_json_string(&self.schema),
            editor.selection(),
        )
    }

    /// Builds a rejected report without mutating the editor.
    fn rejected_report(&self, problem: &ProblemDetails, audit: &AuditLabels) -> String {
        let mut report = ImportReport {
            accepted: false,
            blocks: ops::block_count(&self.inner),
            accepted_blocks: 0,
            version: self.version(),
            schema: String::from(self.schema.fingerprint()),
            request_id: audit.request_id.clone(),
            source: audit.source.clone(),
            destination: audit.destination.clone(),
            id_map: Vec::new(),
            problems: Vec::new(),
        };
        report.problems.push(problem.clone());
        report.to_json(&self.doc_json(), self.inner.selection())
    }
}
