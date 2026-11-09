//! Step staging, fragment replacement, and batch execution.

use std::sync::Arc;

use rich_editor_auy::error::{ProblemDetails, StepError};
use rich_editor_auy::model::{Attr, Attrs, Block, Doc, Fragment, Mark};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Selection, Transaction};
use rich_editor_auy::wire::{ParseOptions, from_slice_json};

use crate::attr;
use crate::dto::Op;
use crate::ops::{decode_doc, decode_fragment, selection_of};
use crate::problem;
use serde_json::Value;

/// Returns the top-level boundary index for a position.
fn boundary_index(doc: &Doc, pos: Pos) -> Result<usize, ProblemDetails> {
    let size = doc.size().0;
    if pos.0 > size {
        return Err(problem::from_step_error(&StepError::OutOfBounds {
            position: pos.0,
            size,
        }));
    }
    let mut offset = 0_u32;
    for (index, block) in doc.blocks().iter().enumerate() {
        if offset == pos.0 {
            return Ok(index);
        }
        let end = offset.saturating_add(block.size().0);
        if end == pos.0 {
            return Ok(index.saturating_add(1));
        }
        offset = end;
    }
    if offset == pos.0 {
        Ok(doc.blocks().len())
    } else {
        Err(problem::from_step_error(&StepError::Unsupported {
            operation: Box::from("fragment_not_on_block_boundary"),
        }))
    }
}

/// Builds the replacement document for a fragment range.
pub(crate) fn fragment_replacement(
    doc: &Doc,
    range: Range,
    fragment: &Fragment,
) -> Result<Doc, ProblemDetails> {
    if range.start > range.end {
        return Err(problem::from_step_error(&StepError::OutOfBounds {
            position: range.start.0,
            size: doc.size().0,
        }));
    }
    let start = boundary_index(doc, range.start)?;
    let end = boundary_index(doc, range.end)?;
    let mut blocks: Vec<Arc<Block>> = Vec::new();
    blocks.extend(doc.blocks().iter().take(start).cloned());
    blocks.extend_from_slice(fragment.blocks());
    blocks.extend(doc.blocks().iter().skip(end).cloned());
    Doc::from_blocks(doc.attrs().clone(), blocks).map_err(|error| problem::from_limit_error(&error))
}

/// Stages one operation on a transaction.
#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive match keeps every operation shape visible"
)]
pub(crate) fn stage(
    schema: &Schema,
    doc: &Doc,
    transaction: &mut Transaction,
    op: &Op,
) -> Result<(), ProblemDetails> {
    match op {
        Op::InsertText { at, text } => {
            transaction
                .insert_text(Pos(*at), text)
                .map_err(|error| problem::from_step_error(&error))?;
            let length = u32::try_from(text.len()).unwrap_or(u32::MAX);
            transaction.set_selection(Selection::caret(Pos(at.saturating_add(length))));
        }
        Op::DeleteRange { from, to } => {
            transaction
                .delete(Range::new(Pos(*from), Pos(*to)))
                .map_err(|error| problem::from_step_error(&error))?;
            transaction.set_selection(Selection::caret(Pos(*from)));
        }
        Op::SplitBlock { at } => {
            transaction
                .split_block(Pos(*at))
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::JoinBackward { at } => {
            transaction
                .join_backward(Pos(*at))
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::ToggleMark {
            from,
            to,
            mark,
            active,
        } => {
            let id = schema
                .mark_id(mark)
                .ok_or_else(|| problem::token_problem(mark, "mark"))?;
            let range = Range::new(Pos(*from), Pos(*to));
            let active =
                active.unwrap_or_else(|| !range.is_empty() && attr::range_has_mark(doc, range, id));
            let result = if active {
                transaction.remove_mark(range, Mark::new(id))
            } else {
                transaction.add_mark(range, Mark::new(id))
            };
            result.map_err(|error| problem::from_step_error(&error))?;
        }
        Op::SetBlockAttr { at, name, value } => {
            let size = doc.size().0;
            let kind = attr::block_kind_at(doc, Pos(*at)).ok_or_else(|| {
                problem::from_step_error(&StepError::OutOfBounds {
                    position: *at,
                    size,
                })
            })?;
            let spec = schema
                .node_attr(kind, name)
                .ok_or_else(|| problem::attr_problem(name, "node"))?;
            let decoded = attr::value_from_json(&spec.kind, value, name)?;
            transaction
                .set_node_attr(Pos(*at), spec.id, decoded)
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::SetDocAttr { name, value } => {
            let spec = schema
                .doc()
                .attrs
                .iter()
                .find(|spec| &*spec.name == name)
                .ok_or_else(|| problem::attr_problem(name, "doc"))?;
            let decoded = attr::value_from_json(&spec.kind, value, name)?;
            transaction
                .set_doc_attr(spec.id, decoded)
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::SetInlineAttr {
            from,
            to,
            name,
            value,
        } => {
            let kind = attr::atom_kind_at(doc, Pos(*from), Pos(*to)).ok_or_else(|| {
                problem::from_step_error(&StepError::Unsupported {
                    operation: Box::from("inline_attr_not_atom"),
                })
            })?;
            let spec = schema
                .node_attr(kind, name)
                .ok_or_else(|| problem::attr_problem(name, "inline"))?;
            let decoded = attr::value_from_json(&spec.kind, value, name)?;
            transaction
                .set_inline_attr(Range::new(Pos(*from), Pos(*to)), spec.id, decoded)
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::SetMarkAttr {
            from,
            to,
            mark,
            name,
            value,
        } => {
            let id = schema
                .mark_id(mark)
                .ok_or_else(|| problem::token_problem(mark, "mark"))?;
            let spec = schema
                .mark(id)
                .and_then(|spec| spec.attrs.iter().find(|attr| &*attr.name == name))
                .ok_or_else(|| problem::attr_problem(name, "mark"))?;
            let decoded = attr::value_from_json(&spec.kind, value, name)?;
            transaction
                .set_mark_attr(Range::new(Pos(*from), Pos(*to)), id, spec.id, decoded)
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::ReplaceDocument { doc: value } => {
            let replacement = decode_doc(schema, value)?;
            transaction
                .replace_document(replacement)
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::ReplaceFragment {
            from,
            to,
            at,
            fragment,
        } => {
            let fragment = decode_fragment(schema, fragment)?;
            let start = (*from).or(*at).unwrap_or(0);
            let end = (*to).or(*at).unwrap_or(start);
            let replacement =
                fragment_replacement(doc, Range::new(Pos(start), Pos(end)), &fragment)?;
            transaction
                .replace_document(replacement)
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::SetSelection { selection } => {
            transaction.set_selection(selection_of(*selection));
        }
        Op::SetNodeId { at, id } => {
            transaction
                .set_node_id(Pos(*at), id.as_deref())
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::SetBlockType { at, kind, attrs } => {
            let kind = schema
                .node_kind(kind)
                .ok_or_else(|| problem::token_problem(kind, "node"))?;
            let mut decoded = Attrs::empty();
            if let Some(Value::Object(map)) = attrs {
                for (name, value) in map {
                    let spec = schema
                        .node_attr(kind, name)
                        .ok_or_else(|| problem::attr_problem(name, "node"))?;
                    let value = attr::value_from_json(&spec.kind, value, name)?;
                    decoded = decoded
                        .with(Attr { id: spec.id, value })
                        .ok_or_else(|| problem::invalid_attr_problem(name, "node"))?;
                }
            }
            transaction
                .set_block_type(schema, Pos(*at), kind, decoded)
                .map_err(|error| problem::from_step_error(&error))?;
        }
        Op::ReplaceSlice { from, to, slice } => {
            let text = slice.to_string();
            let decoded = from_slice_json(&text, schema, &ParseOptions::default())
                .map_err(|error| problem::from_parse_error(&error))?;
            transaction
                .replace(Range::new(Pos(*from), Pos(*to)), decoded)
                .map_err(|error| problem::from_step_error(&error))?;
        }
    }
    Ok(())
}
