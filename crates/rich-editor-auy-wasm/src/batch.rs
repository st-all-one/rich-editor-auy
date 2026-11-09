//! Batch execution with history metadata.

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::model::GroupId;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{AppliedInfo, Editor as CoreEditor, Origin};

use crate::dto::Op;
use crate::problem;
use crate::stage;

/// Applies operations to a working editor, returning the last applied result.
/// History metadata for one batch.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BatchMeta {
    /// History origin.
    pub(crate) origin: Origin,
    /// Explicit history group.
    pub(crate) group: Option<u32>,
    /// Whether the batch enters history.
    pub(crate) add_to_history: Option<bool>,
}

/// Applies operations with explicit history metadata.
pub(crate) fn run_ops_with(
    editor: &mut CoreEditor,
    schema: &Schema,
    ops: &[Op],
    meta: &BatchMeta,
) -> Result<Option<AppliedInfo>, ProblemDetails> {
    let origin = meta.origin;
    let add_to_history = meta.add_to_history;
    let group = GroupId(u64::from(meta.group.unwrap_or_else(|| {
        u32::try_from(editor.version().saturating_add(1)).unwrap_or(u32::MAX)
    })));
    let mut last = None;
    for op in ops {
        let mut transaction = editor.transaction();
        transaction.set_origin(origin);
        transaction.set_group(group);
        if let Some(add) = add_to_history {
            transaction.set_add_to_history(add);
        }
        stage::stage(schema, editor.doc(), &mut transaction, op)?;
        let info = editor
            .apply(transaction)
            .map_err(|error| problem::from_edit_error(&error))?;
        last = Some(info);
    }
    Ok(last)
}
