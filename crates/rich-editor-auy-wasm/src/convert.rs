//! Serialization of editor state, history, change sets, and import reports.

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::model::CellRef;
use rich_editor_auy::pos::Range;
use rich_editor_auy::state::{AppliedInfo, ChangeSet, HistoryInfo, Origin, Selection};
use serde_json::{Value, json};

use crate::ids::IdPair;
use crate::problem::problem_value;

/// Saturates a durable version into the JavaScript-safe `u32` range.
pub(crate) fn version_u32(version: u64) -> u32 {
    u32::try_from(version).unwrap_or(u32::MAX)
}

/// Returns the JSON range `{from, to}`.
pub(crate) fn range_value(range: Range) -> Value {
    json!({"from": range.start.0, "to": range.end.0})
}

/// Returns the serialized selection.
pub(crate) fn selection_value(selection: Selection) -> Value {
    match selection {
        Selection::None => json!({"type": "none"}),
        Selection::Caret { pos, assoc } => {
            json!({"type": "caret", "pos": pos.0, "assoc": assoc.sign()})
        }
        Selection::Text {
            anchor,
            head,
            assoc,
        } => json!({
            "type": "text",
            "anchor": anchor.0,
            "head": head.0,
            "assoc": assoc.sign(),
        }),
        Selection::Node { pos } => json!({"type": "node", "pos": pos.0}),
        Selection::Table {
            table,
            anchor,
            head,
        } => json!({
            "type": "table",
            "table": table,
            "anchor": cell_value(anchor),
            "head": cell_value(head),
        }),
    }
}

/// Returns the serialized table cell reference.
fn cell_value(cell: CellRef) -> Value {
    json!({"table": cell.table, "row": cell.row, "column": cell.column})
}

/// Returns the serialized history state.
pub(crate) fn history_value(info: HistoryInfo) -> Value {
    json!({
        "can_undo": info.can_undo,
        "can_redo": info.can_redo,
        "undo_depth": info.undo_depth,
        "redo_depth": info.redo_depth,
        "bytes": info.bytes,
        "last_origin": info.last_origin.map(origin_name),
    })
}

/// Returns the serialized change set.
pub(crate) fn changes_value(changes: &ChangeSet) -> Value {
    let ranges = changes
        .ranges
        .iter()
        .map(|change| {
            json!({
                "old": range_value(change.old),
                "new": range_value(change.new),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "blocks": {
            "from": changes.blocks.start,
            "to": changes.blocks.end,
        },
        "ranges": ranges,
        "selection": changes.selection,
        "full": changes.full,
        "insertedBlocks": changes.inserted_blocks,
        "removedBlocks": changes.removed_blocks,
    })
}

/// Returns the stable origin name.
fn origin_name(origin: Origin) -> &'static str {
    match origin {
        Origin::User => "user",
        Origin::Paste => "paste",
        Origin::Composition(_) => "composition",
        Origin::Remote => "remote",
        Origin::Api => "api",
        Origin::Import => "import",
    }
}

/// Writes `{doc, selection, version}`.
pub(crate) fn state_json(doc_json: &str, selection: Selection, version: u32) -> String {
    format!(
        "{{\"doc\":{doc_json},\"selection\":{},\"version\":{version}}}",
        selection_value(selection)
    )
}

/// Writes an applied result including changes and history.
pub(crate) fn applied_json(info: &AppliedInfo) -> String {
    format!(
        "{{\"selection\":{},\"version\":{},\"changes\":{},\"history\":{}}}",
        selection_value(info.selection),
        version_u32(info.version),
        changes_value(&info.changes),
        history_value(info.history),
    )
}

/// Writes an applied result that changed no document content.
pub(crate) fn empty_applied_json(
    selection: Selection,
    version: u32,
    history: HistoryInfo,
) -> String {
    format!(
        "{{\"selection\":{},\"version\":{version},\"changes\":{},\"history\":{}}}",
        selection_value(selection),
        changes_value(&ChangeSet::empty()),
        history_value(history),
    )
}

/// Writes an undo or redo result with an explicit `applied` flag.
pub(crate) fn undo_json(info: Option<&AppliedInfo>, selection: Selection, version: u32) -> String {
    match info {
        Some(info) => format!(
            "{{\"applied\":true,\"selection\":{},\"version\":{},\
             \"changes\":{},\"history\":{}}}",
            selection_value(info.selection),
            version_u32(info.version),
            changes_value(&info.changes),
            history_value(info.history),
        ),
        None => format!(
            "{{\"applied\":false,\"selection\":{},\"version\":{version}}}",
            selection_value(selection)
        ),
    }
}

/// Result of a validating batch or import.
pub(crate) struct ImportReport {
    /// Whether the batch was accepted.
    pub(crate) accepted: bool,
    /// Top-level block count after the attempt.
    pub(crate) blocks: u32,
    /// Top-level blocks accepted from the payload.
    pub(crate) accepted_blocks: u32,
    /// Document version after the attempt.
    pub(crate) version: u32,
    /// Schema fingerprint the payload was applied against.
    pub(crate) schema: String,
    /// Host correlation identifier echoed from the payload.
    pub(crate) request_id: Option<String>,
    /// Free-form origin label echoed from the payload.
    pub(crate) source: Option<String>,
    /// Free-form destination label echoed from the payload.
    pub(crate) destination: Option<String>,
    /// Anchor rewrites performed by the `regenerate` policy.
    pub(crate) id_map: Vec<IdPair>,
    /// Every rejection, in evaluation order.
    pub(crate) problems: Vec<ProblemDetails>,
}

impl ImportReport {
    /// Writes the report and the resulting state.
    pub(crate) fn to_json(&self, doc_json: &str, selection: Selection) -> String {
        let problems = self
            .problems
            .iter()
            .map(problem_value)
            .collect::<Vec<Value>>();
        let id_map = self
            .id_map
            .iter()
            .map(|pair| json!({"from": pair.from, "to": pair.to}))
            .collect::<Vec<Value>>();
        let request_id = self
            .request_id
            .as_ref()
            .map_or(Value::Null, |value| Value::String(value.clone()));
        let accepted = self.accepted;
        let blocks = self.blocks;
        let accepted_blocks = self.accepted_blocks;
        let version = self.version;
        let schema = Value::String(self.schema.clone());
        let id_map = Value::Array(id_map);
        let problems = Value::Array(problems);
        let state = state_json(doc_json, selection, version);
        let source = self
            .source
            .as_ref()
            .map_or(Value::Null, |value| Value::String(value.clone()));
        let destination = self
            .destination
            .as_ref()
            .map_or(Value::Null, |value| Value::String(value.clone()));
        format!(
            "{{\"accepted\":{accepted},\"blocks\":{blocks},\"acceptedBlocks\":{accepted_blocks},\"version\":{version},\"schema\":{schema},\"requestId\":{request_id},\"source\":{source},\"destination\":{destination},\"idMap\":{id_map},\"problems\":{problems},\"state\":{state}}}"
        )
    }
}
