//! Step inversion.
#![allow(
    clippy::excessive_nesting,
    reason = "inverse reconstruction walks run boundaries"
)]

use alloc::boxed::Box;

use crate::error::StepError;
use alloc::sync::Arc;

use crate::model::{AttrId, AttrValue, Block, Doc, Fragment};
use crate::pos::Pos;
use crate::step::Slice;
use crate::step::Step;

use super::apply::{locate, replace_len};
use super::inline_attr;
use super::mark_attr::{self, MarkAttrOld};
use super::structural::block_index_at;
use super::text_edit;

/// Produces the inverse step against the document before application.
///
/// # Errors
///
/// Returns an error when the step cannot be inverted by the MVP algebra.
#[allow(
    clippy::too_many_lines,
    reason = "closed step algebra keeps one exhaustive match"
)]
pub fn invert_step(doc: &Doc, step: &Step) -> Result<Step, StepError> {
    match step {
        Step::Replace { from, to, slice } => {
            if !slice.content.is_empty() {
                let old = blocks_between(doc, *from, *to)?;
                let new_to = from.checked_add(slice.content.size().0).ok_or_else(|| {
                    StepError::OutOfBounds {
                        position: from.0,
                        size: doc.size().0,
                    }
                })?;
                return Ok(Step::Replace {
                    from: *from,
                    to: new_to,
                    slice: Slice::from_fragment(Fragment::new(old)),
                });
            }
            let old = text_between_block(doc, *from, *to)?;
            let new_to =
                from.checked_add(replace_len(slice))
                    .ok_or_else(|| StepError::OutOfBounds {
                        position: from.0,
                        size: doc.size().0,
                    })?;
            Ok(Step::Replace {
                from: *from,
                to: new_to,
                slice: Slice::from_textblock(old),
            })
        }
        Step::AddMark { from, to, mark } => Ok(Step::RemoveMark {
            from: *from,
            to: *to,
            mark: mark.clone(),
        }),
        Step::RemoveMark { from, to, mark } => Ok(Step::AddMark {
            from: *from,
            to: *to,
            mark: mark.clone(),
        }),
        Step::SetNodeAttr { pos, attr, .. } => {
            let value = node_attr(doc, *pos, *attr).unwrap_or(AttrValue::Null);
            Ok(Step::SetNodeAttr {
                pos: *pos,
                attr: *attr,
                value,
            })
        }
        Step::SetDocAttr { attr, .. } => {
            let value = doc.attrs().get(*attr).cloned().unwrap_or(AttrValue::Null);
            Ok(Step::SetDocAttr { attr: *attr, value })
        }
        Step::ReplaceDoc { doc: replaced, .. } => Ok(Step::ReplaceDoc {
            doc: doc.clone(),
            old_size: replaced.size().0,
        }),
        Step::SetInlineAttr { from, to, attr, .. } => {
            let value = inline_attr::inline_attr_value(doc, *from, *to, *attr)?;
            Ok(Step::SetInlineAttr {
                from: *from,
                to: *to,
                attr: *attr,
                value,
            })
        }
        Step::SetMarkAttr {
            from,
            to,
            mark,
            attr,
            ..
        } => {
            let value = match mark_attr::mark_attr_value(doc, *from, *to, *mark, *attr) {
                MarkAttrOld::Absent => AttrValue::Null,
                MarkAttrOld::Uniform(value) => value,
                MarkAttrOld::Mixed => {
                    return Ok(Step::ReplaceDoc {
                        doc: doc.clone(),
                        old_size: doc.size().0,
                    });
                }
            };
            Ok(Step::SetMarkAttr {
                from: *from,
                to: *to,
                mark: *mark,
                attr: *attr,
                value,
            })
        }
        Step::ReplaceAround {
            from,
            to,
            gap_from,
            gap_to,
            ..
        } => {
            if from != to || gap_from != gap_to {
                return Err(StepError::Unsupported {
                    operation: Box::from("invert_replace_around"),
                });
            }
            let (_, start, block) = locate(doc, *from)?;
            let text = block.as_textblock().ok_or_else(|| StepError::Unsupported {
                operation: Box::from("invert_split_outside_textblock"),
            })?;
            let local = usize::try_from(from.0.saturating_sub(start)).unwrap_or(usize::MAX);
            if local == 0 {
                return Err(StepError::Unsupported {
                    operation: Box::from("invert_split_token"),
                });
            }
            let (left, right) = text_edit::split_text(text, local.saturating_sub(1))?;
            let new_to = start
                .checked_add(left.size().0)
                .and_then(|value| value.checked_add(right.size().0))
                .ok_or_else(|| StepError::OutOfBounds {
                    position: start,
                    size: doc.size().0,
                })?;
            Ok(Step::Replace {
                from: Pos(start),
                to: Pos(new_to),
                slice: Slice::from_fragment(Fragment::new(alloc::vec![Arc::new(
                    (**block).clone()
                )])),
            })
        }
    }
}

fn text_between_block(doc: &Doc, from: Pos, to: Pos) -> Result<crate::model::TextBlock, StepError> {
    if from > to {
        return Err(StepError::OutOfBounds {
            position: from.0,
            size: doc.size().0,
        });
    }
    let (index, start, block) = locate(doc, from)?;
    let Some(text) = block.as_textblock() else {
        return Err(StepError::Unsupported {
            operation: Box::from("text_between"),
        });
    };
    let end_block = locate(doc, to)?;
    if end_block.0 != index {
        return Err(StepError::Unsupported {
            operation: Box::from("text_across_blocks"),
        });
    }
    let local_start =
        usize::try_from(from.0.saturating_sub(start.saturating_add(1))).unwrap_or(usize::MAX);
    let local_end =
        usize::try_from(to.0.saturating_sub(start.saturating_add(1))).unwrap_or(usize::MAX);
    text_edit::slice_text(text, local_start, local_end)
}

pub(super) fn blocks_between(
    doc: &Doc,
    from: Pos,
    to: Pos,
) -> Result<alloc::vec::Vec<Arc<Block>>, StepError> {
    if from >= to {
        return Err(StepError::OutOfBounds {
            position: from.0,
            size: doc.size().0,
        });
    }
    let start = block_index_at(doc, from, true).ok_or_else(|| StepError::OutOfBounds {
        position: from.0,
        size: doc.size().0,
    })?;
    let end = block_index_at(doc, to, false).ok_or_else(|| StepError::OutOfBounds {
        position: to.0,
        size: doc.size().0,
    })?;
    Ok(doc
        .blocks()
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .cloned()
        .collect())
}

fn node_attr(doc: &Doc, pos: Pos, attr: AttrId) -> Option<AttrValue> {
    let (_, _, block) = locate(doc, pos).ok()?;
    block.attrs().get(attr).cloned()
}
