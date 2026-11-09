//! Position conversion, block lookup, and validation reports.

use rich_editor_auy::a11y::{A11yIssue, A11ySeverity};
use rich_editor_auy::motion;
use rich_editor_auy::pos::Pos;
use serde_json::{Value, json};
use wasm_bindgen::prelude::wasm_bindgen;

use super::Editor;

/// Serializes one accessibility issue.
fn a11y_issue_value(issue: &A11yIssue) -> Value {
    json!({
        "code": issue.code.as_str(),
        "severity": match issue.severity {
            A11ySeverity::Error => "error",
            A11ySeverity::Warning => "warning",
        },
        "path": issue.path.0.as_ref(),
        "range": {
            "from": issue.range.start.0,
            "to": issue.range.end.0,
        },
    })
}

#[wasm_bindgen]
impl Editor {
    /// Converts a document position to UTF-16 code units inside its text block.
    #[wasm_bindgen(js_name = posToUtf16)]
    pub fn pos_to_utf16(&self, pos: u32) -> Option<u32> {
        motion::pos_to_utf16(self.inner.doc(), Pos(pos))
    }

    /// Converts a UTF-16 offset inside a block to a document position.
    #[wasm_bindgen(js_name = utf16ToPos)]
    pub fn utf16_to_pos(&self, block: u32, utf16: u32) -> Option<u32> {
        motion::utf16_to_pos(self.inner.doc(), block, utf16).map(|pos| pos.0)
    }

    /// Snaps a position to the previous scalar boundary.
    #[wasm_bindgen(js_name = snapToScalar)]
    pub fn snap_to_scalar(&self, pos: u32) -> u32 {
        motion::snap_to_scalar(self.inner.doc(), Pos(pos)).0
    }

    /// Returns `{index, from, to}` for the block containing a position.
    #[wasm_bindgen(js_name = blockAt)]
    pub fn block_at(&self, pos: u32) -> Option<String> {
        let (index, _) = self.inner.doc().block_at(Pos(pos))?;
        let index = u32::try_from(index).ok()?;
        let range = self.inner.doc().block_range(index)?;
        Some(
            json!({
                "index": index,
                "from": range.start.0,
                "to": range.end.0,
            })
            .to_string(),
        )
    }

    /// Returns `{from, to}` for one top-level block.
    #[wasm_bindgen(js_name = blockRange)]
    pub fn block_range(&self, index: u32) -> Option<String> {
        let range = self.inner.doc().block_range(index)?;
        Some(
            json!({
                "from": range.start.0,
                "to": range.end.0,
            })
            .to_string(),
        )
    }

    /// Returns the structural validation report.
    #[wasm_bindgen(js_name = validate)]
    pub fn validate(&self) -> String {
        let report = self.inner.doc().validate(&self.schema);
        let issues = report
            .issues
            .iter()
            .map(|issue| {
                let mut value = json!({
                    "invariant": issue.invariant,
                    "path": issue.path.0.as_ref(),
                });
                if let Some(position) = issue.position
                    && let Some(object) = value.as_object_mut()
                {
                    object.insert(String::from("position"), json!(position.0));
                }
                value
            })
            .collect::<Vec<_>>();
        json!({ "ok": report.is_ok(), "issues": issues }).to_string()
    }

    /// Returns the accessibility report.
    #[wasm_bindgen(js_name = validateA11y)]
    pub fn validate_a11y(&self) -> String {
        let report = self.inner.doc().validate_a11y(&self.schema);
        json!({
            "ok": report.is_ok(),
            "errors": report.errors.iter().map(a11y_issue_value).collect::<Vec<_>>(),
            "warnings": report.warnings.iter().map(a11y_issue_value).collect::<Vec<_>>(),
        })
        .to_string()
    }
}
