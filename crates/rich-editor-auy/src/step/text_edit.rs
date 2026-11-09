//! Byte-range edits inside one text block.
#![allow(
    clippy::redundant_pub_crate,
    reason = "internal algebra is crate-scoped"
)]
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{InlineItem, Mark, MarkSet, Run, TextBlock};

mod insert;
mod slice;
pub(crate) use slice::slice_text;

use insert::insert_into_plain_run;

/// Replaces a byte range in a text block and inserts text at the start.
///
/// # Errors
///
/// Returns an error when the range is out of bounds, not on UTF-8 boundaries, or violates
/// normalization limits.
#[allow(clippy::too_many_lines, reason = "single-pass run split and insert")]
pub(crate) fn replace_text(
    block: &TextBlock,
    start: usize,
    end: usize,
    insert: &str,
) -> Result<TextBlock, StepError> {
    let total = usize::try_from(block.len().0).unwrap_or(usize::MAX);
    if start > end || end > total {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("text_range"),
        });
    }
    if start == end
        && let Some(fast) = insert_into_plain_run(block, start, insert)
    {
        return Ok(fast);
    }
    let mut items = Vec::new();
    let mut cursor = 0_usize;
    let mut inserted = false;
    for item in block.items() {
        match item {
            InlineItem::Text(run) => {
                let run_start = cursor;
                let run_end = cursor.saturating_add(run.text.len());
                cursor = run_end;
                if run_end <= start {
                    items.push(item.clone());
                    continue;
                }
                if run_start >= end {
                    if !inserted {
                        push_text(&mut items, insert);
                        inserted = true;
                    }
                    items.push(item.clone());
                    continue;
                }
                let prefix_len = start.saturating_sub(run_start).min(run.text.len());
                let suffix_start = end.saturating_sub(run_start).min(run.text.len());
                if prefix_len > 0 {
                    push_run(
                        &mut items,
                        run.text.get(..prefix_len).ok_or_else(range_error)?,
                        run.marks.clone(),
                    );
                }
                if !inserted {
                    push_text(&mut items, insert);
                    inserted = true;
                }
                if suffix_start < run.text.len() {
                    push_run(
                        &mut items,
                        run.text.get(suffix_start..).ok_or_else(range_error)?,
                        run.marks.clone(),
                    );
                }
            }
            InlineItem::Atom(atom) => {
                let atom_start = cursor;
                let atom_end = cursor.saturating_add(1);
                cursor = atom_end;
                if atom_end <= start {
                    items.push(InlineItem::Atom(atom.clone()));
                } else if atom_start >= end {
                    if !inserted {
                        push_text(&mut items, insert);
                        inserted = true;
                    }
                    items.push(InlineItem::Atom(atom.clone()));
                } else if !inserted {
                    push_text(&mut items, insert);
                    inserted = true;
                }
            }
        }
    }
    if !inserted {
        push_text(&mut items, insert);
    }
    let mut result = TextBlock::new(items);
    result
        .normalize(u32::try_from(crate::model::MAX_RUN_BYTES).unwrap_or(4096))
        .map_err(StepError::Limit)?;
    Ok(result)
}

/// Adds a mark to a byte range in one text block.
///
/// # Errors
///
/// Returns an error when the range is invalid.
pub(crate) fn add_mark(
    block: &TextBlock,
    start: usize,
    end: usize,
    mark: &Mark,
) -> Result<TextBlock, StepError> {
    map_range_marks(block, start, end, &MarkAction::Add(mark))
}

/// Removes a mark from a byte range in one text block.
///
/// # Errors
///
/// Returns an error when the range is invalid.
pub(crate) fn remove_mark(
    block: &TextBlock,
    start: usize,
    end: usize,
    mark: &Mark,
) -> Result<TextBlock, StepError> {
    map_range_marks(block, start, end, &MarkAction::Remove(mark.id))
}

/// Splits one text block at a content offset.
///
/// # Errors
///
/// Returns an error when the offset is not a UTF-8 boundary.
pub(crate) fn split_text(
    block: &TextBlock,
    at: usize,
) -> Result<(TextBlock, TextBlock), StepError> {
    let total = usize::try_from(block.len().0).unwrap_or(usize::MAX);
    if at > total {
        return Err(range_error());
    }
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut cursor = 0_usize;
    let mut split = false;
    for item in block.items() {
        match item {
            InlineItem::Text(run) => {
                let run_start = cursor;
                let run_end = cursor.saturating_add(run.text.len());
                if run_end <= at {
                    left.push(item.clone());
                } else if run_start >= at {
                    right.push(item.clone());
                } else {
                    let offset = at.saturating_sub(run_start);
                    push_run(
                        &mut left,
                        run.text.get(..offset).ok_or_else(range_error)?,
                        run.marks.clone(),
                    );
                    push_run(
                        &mut right,
                        run.text.get(offset..).ok_or_else(range_error)?,
                        run.marks.clone(),
                    );
                    split = true;
                }
                cursor = run_end;
            }
            InlineItem::Atom(atom) => {
                let run_end = cursor.saturating_add(1);
                if run_end <= at {
                    left.push(InlineItem::Atom(atom.clone()));
                } else if cursor >= at {
                    right.push(InlineItem::Atom(atom.clone()));
                } else {
                    split = true;
                }
                cursor = run_end;
            }
        }
    }
    if !split {
        // Empty half is still a valid canonical block.
    }
    let mut left_block = TextBlock::new(left);
    let mut right_block = TextBlock::new(right);
    left_block
        .normalize(u32::try_from(crate::model::MAX_RUN_BYTES).unwrap_or(4096))
        .map_err(StepError::Limit)?;
    right_block
        .normalize(u32::try_from(crate::model::MAX_RUN_BYTES).unwrap_or(4096))
        .map_err(StepError::Limit)?;
    Ok((left_block, right_block))
}

#[allow(
    variant_size_differences,
    reason = "mark actions are cold edit metadata"
)]
enum MarkAction<'a> {
    Add(&'a Mark),
    Remove(crate::model::MarkId),
}

fn map_range_marks(
    block: &TextBlock,
    start: usize,
    end: usize,
    action: &MarkAction<'_>,
) -> Result<TextBlock, StepError> {
    let total = usize::try_from(block.len().0).unwrap_or(usize::MAX);
    if start > end || end > total || start == end {
        return Ok(block.clone());
    }
    let mut items = Vec::new();
    let mut cursor = 0_usize;
    for item in block.items() {
        let InlineItem::Text(run) = item else {
            items.push(item.clone());
            continue;
        };
        let run_start = cursor;
        let run_end = cursor.saturating_add(run.text.len());
        cursor = run_end;
        if run_end <= start || run_start >= end {
            items.push(item.clone());
            continue;
        }
        let prefix_len = start.saturating_sub(run_start).min(run.text.len());
        let suffix_start = end.saturating_sub(run_start).min(run.text.len());
        if prefix_len > 0 {
            push_run(
                &mut items,
                run.text.get(..prefix_len).ok_or_else(range_error)?,
                run.marks.clone(),
            );
        }
        let middle = run
            .text
            .get(prefix_len..suffix_start)
            .ok_or_else(range_error)?;
        let marks = match action {
            MarkAction::Add(mark) => run.marks.with_mark((*mark).clone()),
            MarkAction::Remove(id) => run.marks.without_mark(*id),
        };
        push_run(&mut items, middle, marks);
        if suffix_start < run.text.len() {
            push_run(
                &mut items,
                run.text.get(suffix_start..).ok_or_else(range_error)?,
                run.marks.clone(),
            );
        }
    }
    let mut result = TextBlock::new(items);
    result
        .normalize(u32::try_from(crate::model::MAX_RUN_BYTES).unwrap_or(4096))
        .map_err(StepError::Limit)?;
    Ok(result)
}

fn push_text(items: &mut Vec<InlineItem>, text: &str) {
    if !text.is_empty() {
        items.push(InlineItem::Text(Run::new(
            String::from(text),
            MarkSet::empty(),
        )));
    }
}

fn push_run(items: &mut Vec<InlineItem>, text: &str, marks: MarkSet) {
    if !text.is_empty() {
        items.push(InlineItem::Text(Run::new(String::from(text), marks)));
    }
}

fn range_error() -> StepError {
    StepError::Unsupported {
        operation: alloc::boxed::Box::from("text_boundary"),
    }
}
