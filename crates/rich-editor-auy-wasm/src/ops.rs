//! Batch envelope parsing, selection decoding, and token-tree decoding.

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::model::{CellRef, Doc, Fragment};
use rich_editor_auy::pos::{Assoc, Pos};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor as CoreEditor, Selection};
use rich_editor_auy::wire::{ParseOptions, from_fragment_json, from_json as parse_document};
use serde_json::Value;

use crate::dto::{BatchPayload, CellRefJson, Op, SelectionJson};
use crate::problem;

/// Parses a batch envelope.
pub(crate) fn parse_batch(payload: &str) -> Result<Vec<Op>, ProblemDetails> {
    serde_json::from_str::<BatchPayload>(payload)
        .map(|batch| batch.ops)
        .map_err(|error| problem::malformed_problem(format!("{error}"), "ops"))
}

/// Returns the block count of an editor as a JavaScript-safe integer.
pub(crate) fn block_count(editor: &CoreEditor) -> u32 {
    u32::try_from(editor.doc().blocks().len()).unwrap_or(u32::MAX)
}

/// Converts a selection DTO into a logical selection.
pub(crate) fn selection_of(selection: SelectionJson) -> Selection {
    match selection {
        SelectionJson::None => Selection::None,
        SelectionJson::Caret { pos, assoc } => Selection::Caret {
            pos: Pos(pos),
            assoc: assoc_of(assoc),
        },
        SelectionJson::Text {
            anchor,
            head,
            assoc,
        } => Selection::Text {
            anchor: Pos(anchor),
            head: Pos(head),
            assoc: assoc_of(assoc),
        },
        SelectionJson::Node { pos } => Selection::Node { pos: Pos(pos) },
        SelectionJson::Table {
            table,
            anchor,
            head,
        } => Selection::Table {
            table,
            anchor: cell_ref(anchor),
            head: cell_ref(head),
        },
    }
}

/// Converts a JSON association sign into a boundary association.
fn assoc_of(sign: i8) -> Assoc {
    if sign < 0 {
        Assoc::Before
    } else {
        Assoc::After
    }
}

/// Converts a serialized table cell reference.
fn cell_ref(cell: CellRefJson) -> CellRef {
    CellRef {
        table: cell.table,
        row: cell.row,
        column: cell.column,
    }
}

/// Decodes a document token tree.
pub(crate) fn decode_doc(schema: &Schema, value: &Value) -> Result<Doc, ProblemDetails> {
    let text = serde_json::to_string(value)
        .map_err(|error| problem::malformed_problem(format!("{error}"), "doc"))?;
    parse_document(&text, schema, &ParseOptions::default())
        .map_err(|error| problem::from_parse_error(&error))
}

/// Decodes a fragment token tree.
pub(crate) fn decode_fragment(schema: &Schema, value: &Value) -> Result<Fragment, ProblemDetails> {
    let text = serde_json::to_string(value)
        .map_err(|error| problem::malformed_problem(format!("{error}"), "fragment"))?;
    from_fragment_json(&text, schema, &ParseOptions::default())
        .map_err(|error| problem::from_parse_error(&error))
}
