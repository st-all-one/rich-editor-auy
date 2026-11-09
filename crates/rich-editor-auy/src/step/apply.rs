//! Step application and inversion.
#![allow(
    clippy::redundant_pub_crate,
    reason = "internal algebra is crate-scoped"
)]
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::error::StepError;
use crate::model::{AttrId, Block, Doc};
use crate::pos::Pos;
use crate::schema::Schema;

#[cfg(feature = "visits")]
use core::sync::atomic::{AtomicU64, Ordering};

use super::structural::replace_blocks;
use super::{inline_attr, mark_attr, node_attr};

mod mark_owned;

use super::text_edit;
use super::{Slice, Step};
use mark_owned::mark_range_owned;
use node_attr::replace_block_owned;

/// Located top-level block: index, absolute start, and block.
pub(super) type Located<'a> = (usize, u32, &'a Arc<Block>);

/// Returns the byte length inserted by a slice.
pub(super) fn replace_len(slice: &Slice) -> u32 {
    slice.size()
}

/// Splits the text block that contains a position.
///
/// # Errors
///
/// Returns an unsupported error for a structural position and an out-of-bounds error when the
/// position is outside the document.
pub(crate) fn split_at(doc: &Doc, at: Pos) -> Result<Doc, StepError> {
    split_block(doc, at, 0)
}

/// Applies one step.
///
/// # Errors
///
/// Returns a typed error for any invalid position, content, or attribute.
pub fn apply_step(doc: &Doc, schema: &Schema, step: &Step) -> Result<Doc, StepError> {
    match step {
        Step::Replace { from, to, slice } => replace(doc, *from, *to, slice),
        Step::ReplaceAround {
            from,
            to,
            gap_from,
            gap_to,
            insert,
            ..
        } => {
            if from != to || gap_from != gap_to {
                return Err(StepError::Unsupported {
                    operation: Box::from("replace_around"),
                });
            }
            split_block(doc, *from, *insert)
        }
        Step::AddMark { from, to, mark } => mark_range(doc, schema, *from, *to, mark, true),
        Step::RemoveMark { from, to, mark } => mark_range(doc, schema, *from, *to, mark, false),
        Step::SetNodeAttr { pos, attr, value } => {
            node_attr::set_node_attr(doc, schema, *pos, *attr, value)
        }
        Step::SetDocAttr { attr, value } => node_attr::set_doc_attr(doc, schema, *attr, value),
        Step::ReplaceDoc {
            doc: replacement, ..
        } => Ok(replacement.clone()),
        Step::SetInlineAttr {
            from,
            to,
            attr,
            value,
        } => inline_attr::set_inline_attr(doc, schema, *from, *to, *attr, value),
        Step::SetMarkAttr {
            from,
            to,
            mark,
            attr,
            value,
        } => mark_attr::set_mark_attr(doc, schema, *from, *to, *mark, *attr, value),
    }
}

/// Applies one step, mutating the document in place when it is uniquely owned.
pub(crate) fn apply_step_owned(doc: Doc, schema: &Schema, step: &Step) -> Result<Doc, StepError> {
    match step {
        Step::Replace { from, to, slice } => replace_owned(doc, *from, *to, slice),
        Step::ReplaceAround { .. } | Step::SetDocAttr { .. } => apply_step(&doc, schema, step),
        Step::AddMark { from, to, mark } => mark_range_owned(doc, schema, *from, *to, mark, true),
        Step::RemoveMark { from, to, mark } => {
            mark_range_owned(doc, schema, *from, *to, mark, false)
        }
        Step::SetNodeAttr { pos, attr, value } => {
            node_attr::set_node_attr_owned(doc, schema, *pos, *attr, value)
        }
        Step::ReplaceDoc {
            doc: replacement, ..
        } => Ok(replacement.clone()),
        Step::SetInlineAttr {
            from,
            to,
            attr,
            value,
        } => inline_attr::set_inline_attr_owned(doc, schema, *from, *to, *attr, value),
        Step::SetMarkAttr {
            from,
            to,
            mark,
            attr,
            value,
        } => mark_attr::set_mark_attr_owned(doc, schema, *from, *to, *mark, *attr, value),
    }
}

fn replace(doc: &Doc, from: Pos, to: Pos, slice: &Slice) -> Result<Doc, StepError> {
    replace_owned(doc.clone(), from, to, slice)
}

fn replace_owned(doc: Doc, from: Pos, to: Pos, slice: &Slice) -> Result<Doc, StepError> {
    if from > to {
        return Err(StepError::OutOfBounds {
            position: from.0,
            size: doc.size().0,
        });
    }
    if slice.open_start > 0 || slice.open_end > 0 {
        return Err(StepError::Unsupported {
            operation: Box::from("open_slice"),
        });
    }
    if !slice.content.is_empty() {
        return replace_blocks(&doc, from, to, slice.content.blocks());
    }
    if slice.runs.is_some()
        && !slice.text.is_empty()
        && slice.text.len()
            != usize::try_from(slice.runs.as_ref().map_or(0, |runs| runs.len().0)).unwrap_or(0)
    {
        return Err(StepError::Unsupported {
            operation: Box::from("slice_text_mismatch"),
        });
    }
    let (index, block_start, block) = locate(&doc, from)?;
    let text = block.as_textblock().ok_or_else(|| StepError::Unsupported {
        operation: Box::from("replace_outside_textblock"),
    })?;
    let local_start = usize::try_from(from.0.saturating_sub(block_start)).unwrap_or(usize::MAX);
    let local_end = usize::try_from(to.0.saturating_sub(block_start)).unwrap_or(usize::MAX);
    if local_start == 0 || local_end == 0 {
        return Err(StepError::Unsupported {
            operation: Box::from("replace_structural_token"),
        });
    }
    let content_start = local_start.saturating_sub(1);
    let content_end = local_end.saturating_sub(1);
    let new_text = if let Some(runs) = &slice.runs {
        super::text_splice::replace_textblock(text, content_start, content_end, runs)?
    } else {
        text_edit::replace_text(text, content_start, content_end, &slice.text)?
    };
    let new_block = Block::text(block.kind(), block.attrs().clone(), new_text);
    replace_block_owned(doc, index, new_block)
}

#[allow(
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools,
    reason = "mark operation shares one implementation"
)]
fn mark_range(
    doc: &Doc,
    schema: &Schema,
    from: Pos,
    to: Pos,
    mark: &crate::model::Mark,
    add: bool,
) -> Result<Doc, StepError> {
    mark_range_owned(doc.clone(), schema, from, to, mark, add)
}

fn split_block(doc: &Doc, at: Pos, _insert: u32) -> Result<Doc, StepError> {
    let (index, block_start, block) = locate(doc, at)?;
    let text = block.as_textblock().ok_or_else(|| StepError::Unsupported {
        operation: Box::from("split_outside_textblock"),
    })?;
    let local = usize::try_from(at.0.saturating_sub(block_start)).unwrap_or(usize::MAX);
    if local == 0 {
        return Err(StepError::Unsupported {
            operation: Box::from("split_token"),
        });
    }
    let (left, right) = text_edit::split_text(text, local.saturating_sub(1))?;
    let left_block = Block::text(block.kind(), block.attrs().clone(), left);
    let right_attrs = block.attrs().without(AttrId::NODE_ID);
    let right_block = Block::text(block.kind(), right_attrs, right);
    let mut blocks = doc.blocks().to_vec();
    if let Some(slot) = blocks.get_mut(index) {
        *slot = Arc::new(left_block);
    }
    blocks.insert(index.saturating_add(1), Arc::new(right_block));
    Doc::new(doc.attrs().clone(), blocks, &crate::Limits::default()).map_err(StepError::Limit)
}

#[cfg(feature = "visits")]
static LOCATE_VISITS: AtomicU64 = AtomicU64::new(0);

/// Returns and clears the number of block lookups since the last call.
#[cfg(feature = "visits")]
pub fn take_locate_visits() -> u64 {
    LOCATE_VISITS.swap(0, Ordering::Relaxed)
}

pub(super) fn locate(doc: &Doc, pos: Pos) -> Result<Located<'_>, StepError> {
    #[cfg(feature = "visits")]
    LOCATE_VISITS.fetch_add(1, Ordering::Relaxed);
    let (index, block) = doc.block_at(pos).ok_or_else(|| StepError::OutOfBounds {
        position: pos.0,
        size: doc.size().0,
    })?;
    let start = doc
        .block_range(u32::try_from(index).unwrap_or(u32::MAX))
        .map_or(0, |range| range.start.0);
    Ok((index, start, block))
}
