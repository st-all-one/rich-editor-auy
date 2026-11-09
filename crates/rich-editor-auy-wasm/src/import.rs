//! Structured import modes over the full schema.

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::model::{Doc, GroupId};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor as CoreEditor, Origin};

use crate::dto::ImportPayload;
use crate::ops;
use crate::problem;
use crate::stage;

/// Applies an import payload to a working editor.
pub(crate) fn run_import(
    editor: &mut CoreEditor,
    schema: &Schema,
    payload: &ImportPayload,
) -> Result<(), ProblemDetails> {
    match payload.mode.as_str() {
        "document" => {
            let value = payload.doc.as_ref().ok_or_else(|| {
                problem::malformed_problem(String::from("document mode requires 'doc'"), "doc")
            })?;
            let doc = ops::decode_doc(schema, value)?;
            apply_replacement(editor, doc)
        }
        "fragment" | "replace" | "insert" => {
            let value = payload.fragment.as_ref().ok_or_else(|| {
                problem::malformed_problem(
                    String::from("fragment mode requires 'fragment'"),
                    "fragment",
                )
            })?;
            let fragment = ops::decode_fragment(schema, value)?;
            let (start, end) = fragment_range(payload)?;
            let replacement = stage::fragment_replacement(
                editor.doc(),
                Range::new(Pos(start), Pos(end)),
                &fragment,
            )?;
            apply_replacement(editor, replacement)
        }
        "ops" => {
            let ops = payload.ops.as_ref().ok_or_else(|| {
                problem::malformed_problem(String::from("ops mode requires 'ops'"), "ops")
            })?;
            stage::run_ops(editor, schema, ops, Origin::Import)?;
            Ok(())
        }
        other => Err(problem::mode_problem(other)),
    }
}

/// Resolves the replacement range of a fragment payload.
fn fragment_range(payload: &ImportPayload) -> Result<(u32, u32), ProblemDetails> {
    if payload.mode == "insert" || (payload.mode == "fragment" && payload.at.is_some()) {
        let at = payload.at.ok_or_else(|| {
            problem::malformed_problem(String::from("insert mode requires 'at'"), "at")
        })?;
        return Ok((at, at));
    }
    let from = payload.from.ok_or_else(|| {
        problem::malformed_problem(String::from("replace mode requires 'from'"), "from")
    })?;
    let to = payload.to.ok_or_else(|| {
        problem::malformed_problem(String::from("replace mode requires 'to'"), "to")
    })?;
    Ok((from, to))
}

/// Replaces the document as one import history group.
pub(crate) fn apply_replacement(editor: &mut CoreEditor, doc: Doc) -> Result<(), ProblemDetails> {
    let mut transaction = editor.transaction();
    transaction.set_origin(Origin::Import);
    transaction.set_group(GroupId(editor.version().saturating_add(1)));
    transaction
        .replace_document(doc)
        .map_err(|error| problem::from_step_error(&error))?;
    editor
        .apply(transaction)
        .map_err(|error| problem::from_edit_error(&error))?;
    Ok(())
}
