//! Structured import modes over the full schema.

use rich_editor_auy::Limits;
use rich_editor_auy::error::ProblemDetails;
use std::sync::Arc;

use rich_editor_auy::model::{Block, BlockContent, Doc, GroupId, NodeKind};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor as CoreEditor, Origin};
use rich_editor_auy::wire::ParseOptions;

use crate::batch::{BatchMeta, run_ops_with};
use crate::dto::{BatchLimits, ImportPayload};
use crate::ids::{IdPair, IdPolicy, rewrite_doc, rewrite_fragment};
use crate::ops;
use crate::problem;
use crate::stage;

/// What an accepted payload contributed to the editor.
pub(crate) struct ImportOutcome {
    /// Top-level blocks accepted from the payload.
    pub(crate) accepted_blocks: u32,
    /// Anchor rewrites performed by the `regenerate` policy.
    pub(crate) id_map: Vec<IdPair>,
}

/// Applies an import payload to a working editor.
pub(crate) fn run_import(
    editor: &mut CoreEditor,
    schema: &Schema,
    payload: &ImportPayload,
) -> Result<ImportOutcome, ProblemDetails> {
    let policy = IdPolicy::parse(
        payload.options.id_policy.as_deref(),
        payload.options.id_seed.as_deref(),
    )?;
    let options = parse_options(payload.options.limits.as_ref());
    match payload.mode.as_str() {
        "document" => {
            let value = payload.doc.as_ref().ok_or_else(|| {
                problem::malformed_problem(String::from("document mode requires 'doc'"), "doc")
            })?;
            let decoded = ops::decode_doc_with(schema, value, &options)?;
            let accepted_blocks = top_blocks(decoded.blocks().len());
            let (doc, id_map) = rewrite_doc(&decoded, &policy, schema)?;
            apply_replacement(editor, doc, payload.options.add_to_history)?;
            Ok(ImportOutcome {
                accepted_blocks,
                id_map,
            })
        }
        "fragment" | "replace" | "insert" => {
            let value = payload.fragment.as_ref().ok_or_else(|| {
                problem::malformed_problem(
                    String::from("fragment mode requires 'fragment'"),
                    "fragment",
                )
            })?;
            let decoded = ops::decode_fragment_with(schema, value, &options)?;
            let accepted_blocks = top_blocks(decoded.blocks().len());
            let (fragment, id_map) = rewrite_fragment(&decoded, &policy, schema)?;
            let (start, end) = fragment_range(payload)?;
            let replacement = stage::fragment_replacement(
                editor.doc(),
                Range::new(Pos(start), Pos(end)),
                &fragment,
            )?;
            apply_replacement(editor, replacement, payload.options.add_to_history)?;
            Ok(ImportOutcome {
                accepted_blocks,
                id_map,
            })
        }
        "" | "ops" => {
            let ops = payload.ops.as_ref().ok_or_else(|| {
                problem::malformed_problem(String::from("ops mode requires 'ops'"), "ops")
            })?;
            check_op_limit(ops.len(), payload.options.limits.as_ref())?;
            let meta = BatchMeta {
                origin: Origin::Import,
                group: None,
                add_to_history: payload.options.add_to_history,
            };
            run_ops_with(editor, schema, ops, &meta)?;
            Ok(ImportOutcome {
                accepted_blocks: ops::block_count(editor),
                id_map: Vec::new(),
            })
        }
        other => Err(problem::mode_problem(other)),
    }
}

/// Fails the import when the resulting top-level block count exceeds the payload limit.
///
/// # Errors
///
/// Returns a limit problem naming `maxBlocks`.
pub(crate) fn check_block_limit(
    editor: &CoreEditor,
    limits: Option<&BatchLimits>,
) -> Result<(), ProblemDetails> {
    let Some(max) = limits.and_then(|limits| limits.max_blocks) else {
        return Ok(());
    };
    let blocks = ops::block_count(editor);
    if blocks > max {
        return Err(problem::limit_problem("maxBlocks", "blocks"));
    }
    Ok(())
}

/// Fails the import when one batch carries more operations than `maxOps`.
///
/// # Errors
///
/// Returns a limit problem naming `maxOps`.
fn check_op_limit(count: usize, limits: Option<&BatchLimits>) -> Result<(), ProblemDetails> {
    let Some(max) = limits.and_then(|limits| limits.max_ops) else {
        return Ok(());
    };
    let count = u32::try_from(count).unwrap_or(u32::MAX);
    if count > max {
        return Err(problem::limit_problem("maxOps", "ops"));
    }
    Ok(())
}

/// Fails the import when the document holds more table cells than `maxTableCells`.
///
/// # Errors
///
/// Returns a limit problem naming `maxTableCells`.
pub(crate) fn check_table_limit(
    editor: &CoreEditor,
    schema: &Schema,
    limits: Option<&BatchLimits>,
) -> Result<(), ProblemDetails> {
    let Some(max) = limits.and_then(|limits| limits.max_table_cells) else {
        return Ok(());
    };
    let Some(cell) = schema.node_kind("table_cell") else {
        return Ok(());
    };
    let header = schema.node_kind("table_header");
    let blocks = editor.doc().blocks().to_vec();
    let mut total = 0_u32;
    for block in &blocks {
        total = total.saturating_add(count_cells(block, cell, header));
    }
    if total > max {
        return Err(problem::limit_problem("maxTableCells", "tables"));
    }
    Ok(())
}

/// Applies every payload-level limit that needs the resulting document.
///
/// # Errors
///
/// Returns the first limit problem found.
pub(crate) fn check_limits(
    editor: &CoreEditor,
    schema: &Schema,
    limits: Option<&BatchLimits>,
) -> Result<(), ProblemDetails> {
    check_block_limit(editor, limits)?;
    check_table_limit(editor, schema, limits)
}

fn count_cells(block: &Arc<Block>, cell: NodeKind, header: Option<NodeKind>) -> u32 {
    let mut total = u32::from(block.kind() == cell || Some(block.kind()) == header);
    if let BlockContent::Blocks(children) = block.content() {
        for child in children {
            total = total.saturating_add(count_cells(child, cell, header));
        }
    }
    total
}

/// Fails the import when the raw payload is larger than `maxBytes`.
///
/// # Errors
///
/// Returns a limit problem naming `maxBytes`.
pub(crate) fn check_byte_limit(
    payload: &str,
    limits: Option<&BatchLimits>,
) -> Result<(), ProblemDetails> {
    let Some(max) = limits.and_then(|limits| limits.max_bytes) else {
        return Ok(());
    };
    let bytes = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    if bytes > max {
        return Err(problem::limit_problem("maxBytes", "payload"));
    }
    Ok(())
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
pub(crate) fn apply_replacement(
    editor: &mut CoreEditor,
    doc: Doc,
    add_to_history: Option<bool>,
) -> Result<(), ProblemDetails> {
    let mut transaction = editor.transaction();
    transaction.set_origin(Origin::Import);
    transaction.set_group(GroupId(editor.version().saturating_add(1)));
    if let Some(add) = add_to_history {
        transaction.set_add_to_history(add);
    }
    transaction
        .replace_document(doc)
        .map_err(|error| problem::from_step_error(&error))?;
    editor
        .apply(transaction)
        .map_err(|error| problem::from_edit_error(&error))?;
    Ok(())
}

/// Maps tightening overrides onto decoder options; defaults are never widened.
fn parse_options(limits: Option<&BatchLimits>) -> ParseOptions {
    let mut core = Limits::default();
    if let Some(limits) = limits {
        if let Some(value) = limits.max_depth {
            core.max_depth = core.max_depth.min(value);
        }
        if let Some(value) = limits.max_nodes {
            core.max_nodes = core.max_nodes.min(value);
        }
        if let Some(value) = limits.max_runs_per_block {
            core.max_runs_per_block = core.max_runs_per_block.min(value);
        }
    }
    ParseOptions { limits: core }
}

fn top_blocks(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}
