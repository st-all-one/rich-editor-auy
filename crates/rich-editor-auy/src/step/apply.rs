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
use alloc::string::ToString;
use alloc::sync::Arc;

use crate::error::StepError;
use crate::model::{Attr, AttrId, AttrValue, Block, Doc};
use crate::pos::Pos;
use crate::schema::Schema;

#[cfg(feature = "visits")]
use core::sync::atomic::{AtomicU64, Ordering};

use super::structural::replace_blocks;
use super::text_edit;
use super::{Slice, Step};

/// Located top-level block: index, absolute start, and block.
pub(super) type Located<'a> = (usize, u32, &'a Arc<Block>);

/// Returns the byte length inserted by a slice.
pub(super) fn replace_len(slice: &Slice) -> u32 {
    if !slice.content.is_empty() {
        return slice.content.size().0;
    }
    if let Some(runs) = &slice.runs {
        return runs.len().0;
    }
    u32::try_from(slice.text.len()).unwrap_or(u32::MAX)
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
        Step::SetNodeAttr { pos, attr, value } => set_node_attr(doc, schema, *pos, *attr, value),
        Step::SetDocAttr { attr, value } => set_doc_attr(doc, schema, *attr, value),
    }
}

fn replace(doc: &Doc, from: Pos, to: Pos, slice: &Slice) -> Result<Doc, StepError> {
    if from > to {
        return Err(StepError::OutOfBounds {
            position: from.0,
            size: doc.size().0,
        });
    }
    if !slice.content.is_empty() {
        return replace_blocks(doc, from, to, slice.content.blocks());
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
    let (index, block_start, block) = locate(doc, from)?;
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
    replace_block(
        doc,
        index,
        Block::text(block.kind(), block.attrs().clone(), new_text),
    )
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
    if from >= to {
        return Ok(doc.clone());
    }
    let mut result = doc.clone();
    let mut offset = 0_u32;
    for (index, block) in doc.blocks().iter().enumerate() {
        let block_start = offset;
        let block_end = offset.saturating_add(block.size().0);
        offset = block_end;
        if block_end <= from.0 || block_start >= to.0 {
            continue;
        }
        let Some(text) = block.as_textblock() else {
            return Err(StepError::Unsupported {
                operation: Box::from("mark_outside_textblock"),
            });
        };
        if !schema.mark_allowed(block.kind(), mark.id) {
            return Err(StepError::Unknown {
                path: Box::from("mark"),
            });
        }
        let start = from.0.max(block_start.saturating_add(1));
        let end = to.0.min(block_end.saturating_sub(1));
        if start > end {
            continue;
        }
        let local_start = usize::try_from(start.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let local_end = usize::try_from(end.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let new_text = if add {
            text_edit::add_mark(text, local_start, local_end, mark)?
        } else {
            text_edit::remove_mark(text, local_start, local_end, mark)?
        };
        let new_block = Block::text(block.kind(), block.attrs().clone(), new_text);
        result = replace_block(&result, index, new_block)?;
    }
    Ok(result)
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

fn set_node_attr(
    doc: &Doc,
    schema: &Schema,
    pos: Pos,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    let (index, _start, block) = locate(doc, pos)?;
    let attrs = if matches!(value, AttrValue::Null) {
        block.attrs().without(attr)
    } else {
        block
            .attrs()
            .with(Attr {
                id: attr,
                value: value.clone(),
            })
            .ok_or_else(|| StepError::Unknown {
                path: Box::from("attr"),
            })?
    };
    schema
        .validate_attrs(block.kind(), &attrs)
        .map_err(|error| StepError::Unknown {
            path: Box::from(error.to_string()),
        })?;
    let new_block = match block.as_textblock() {
        Some(text) => Block::text(block.kind(), attrs, text.clone()),
        None => Block::container(
            block.kind(),
            attrs,
            block
                .children()
                .map(<[Arc<Block>]>::to_vec)
                .unwrap_or_default(),
        ),
    };
    replace_block(doc, index, new_block)
}

fn set_doc_attr(
    doc: &Doc,
    schema: &Schema,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    let Some(spec) = schema.doc().attrs.iter().find(|spec| spec.id == attr) else {
        return Err(StepError::Unknown {
            path: Box::from("doc_attr"),
        });
    };
    if !matches!(value, AttrValue::Null) && !Schema::value_matches(&spec.kind, value) {
        return Err(StepError::Unknown {
            path: Box::from("doc_attr_type"),
        });
    }
    let attrs = if matches!(value, AttrValue::Null) {
        doc.attrs().without(attr)
    } else {
        doc.attrs()
            .with(Attr {
                id: attr,
                value: value.clone(),
            })
            .ok_or_else(|| StepError::Unknown {
                path: Box::from("doc_attr"),
            })?
    };
    Doc::new(attrs, doc.blocks().to_vec(), &crate::Limits::default()).map_err(StepError::Limit)
}

fn replace_block(doc: &Doc, index: usize, block: Block) -> Result<Doc, StepError> {
    if index >= doc.blocks().len() {
        return Err(StepError::OutOfBounds {
            position: u32::try_from(index).unwrap_or(u32::MAX),
            size: doc.size().0,
        });
    }
    doc.replacing_block(index, block, &crate::Limits::default())
        .map_err(StepError::Limit)
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
