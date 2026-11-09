//! Mark attribute edits.
#![allow(clippy::excessive_nesting, reason = "explicit edit algebra over runs")]

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{
    Attr, AttrId, AttrValue, Attrs, Block, Doc, InlineItem, Mark, MarkId, Run, TextBlock,
};
use crate::pos::Pos;
use crate::schema::Schema;

/// Previous mark attribute state across the runs touched by a step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MarkAttrOld {
    /// No run in the range carries the mark.
    Absent,
    /// Every run carries the same value.
    Uniform(AttrValue),
    /// Runs disagree about the value.
    Mixed,
}

/// Applies a mark attribute change to every run that carries the mark in a range.
#[allow(
    clippy::too_many_arguments,
    reason = "one step expands to its explicit fields"
)]
pub(super) fn set_mark_attr(
    doc: &Doc,
    schema: &Schema,
    from: Pos,
    to: Pos,
    mark: MarkId,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    set_mark_attr_owned(doc.clone(), schema, from, to, mark, attr, value)
}

/// Applies a mark attribute change, reusing the document when uniquely owned.
#[allow(
    clippy::too_many_arguments,
    reason = "one step expands to its explicit fields"
)]
pub(super) fn set_mark_attr_owned(
    mut doc: Doc,
    schema: &Schema,
    from: Pos,
    to: Pos,
    mark: MarkId,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    crate::step::check_range(&doc, from, to)?;
    if from == to {
        return Ok(doc);
    }
    validate_mark_attr(schema, mark, attr, value)?;
    let block_count = doc.blocks().len();
    let mut offset = 0_u32;
    let mut index = 0_usize;
    while index < block_count {
        let Some(block) = doc.block(u32::try_from(index).unwrap_or(u32::MAX)).cloned() else {
            break;
        };
        let block_start = offset;
        let block_end = offset.saturating_add(block.size().0);
        offset = block_end;
        if block_end <= from.0 || block_start >= to.0 {
            index = index.saturating_add(1);
            continue;
        }
        let Some(text) = block.as_textblock() else {
            return Err(StepError::Unsupported {
                operation: Box::from("mark_attr_outside_textblock"),
            });
        };
        if !schema.mark_allowed(block.kind(), mark) {
            return Err(StepError::Unknown {
                path: Box::from("mark"),
            });
        }
        let start = from.0.max(block_start.saturating_add(1));
        let end = to.0.min(block_end.saturating_sub(1));
        if start >= end {
            index = index.saturating_add(1);
            continue;
        }
        let local_start = usize::try_from(start.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let local_end = usize::try_from(end.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let new_text = rewrite_mark_attr(text, local_start..local_end, mark, attr, value)?;
        let new_block = Block::text(block.kind(), block.attrs().clone(), new_text);
        offset = block_start.saturating_add(new_block.size().0);
        doc = doc.replacing_block_owned(index, new_block, &crate::Limits::default())?;
        index = index.saturating_add(1);
    }
    Ok(doc)
}

/// Returns the previous attribute value across the runs in a range.
pub(super) fn mark_attr_value(
    doc: &Doc,
    from: Pos,
    to: Pos,
    mark: MarkId,
    attr: AttrId,
) -> MarkAttrOld {
    if from >= to {
        return MarkAttrOld::Absent;
    }
    let mut found: Option<AttrValue> = None;
    let mut offset = 0_u32;
    for block in doc.blocks().iter() {
        let block_start = offset;
        let block_end = offset.saturating_add(block.size().0);
        offset = block_end;
        if block_end <= from.0 || block_start >= to.0 {
            continue;
        }
        let Some(text) = block.as_textblock() else {
            continue;
        };
        let start = from.0.max(block_start.saturating_add(1));
        let end = to.0.min(block_end.saturating_sub(1));
        if start >= end {
            continue;
        }
        let local_start = usize::try_from(start.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let local_end = usize::try_from(end.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let mut cursor = 0_usize;
        for item in text.items() {
            let next = cursor.saturating_add(usize::try_from(item.size()).unwrap_or(usize::MAX));
            if let InlineItem::Text(run) = item
                && run.marks.contains(mark)
                && cursor < local_end
                && next > local_start
            {
                let value = run
                    .marks
                    .attrs_for(mark)
                    .and_then(|attrs| attrs.get(attr))
                    .cloned()
                    .unwrap_or(AttrValue::Null);
                match &found {
                    Some(previous) if previous != &value => return MarkAttrOld::Mixed,
                    Some(_) => {}
                    None => found = Some(value),
                }
            }
            cursor = next;
        }
    }
    found.map_or(MarkAttrOld::Absent, MarkAttrOld::Uniform)
}

/// Rewrites the mark attribute on the runs intersecting a content range.
fn rewrite_mark_attr(
    text: &TextBlock,
    content: core::ops::Range<usize>,
    mark: MarkId,
    attr: AttrId,
    value: &AttrValue,
) -> Result<TextBlock, StepError> {
    let mut items = Vec::with_capacity(text.items().len());
    let mut cursor = 0_usize;
    for item in text.items() {
        let next = cursor.saturating_add(usize::try_from(item.size()).unwrap_or(usize::MAX));
        if let InlineItem::Text(run) = item
            && run.marks.contains(mark)
            && cursor < content.end
            && next > content.start
        {
            let attrs = mark_attrs(run.marks.attrs_for(mark), attr, value)?;
            let marks = run.marks.with_mark(Mark { id: mark, attrs });
            items.push(InlineItem::Text(Run::new(run.text.clone(), marks)));
        } else {
            items.push(item.clone());
        }
        cursor = next;
    }
    Ok(TextBlock::new(items))
}

/// Returns mark attributes with one attribute replaced or removed.
fn mark_attrs(
    current: Option<&Attrs>,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Attrs, StepError> {
    let base = current.cloned().unwrap_or_else(Attrs::empty);
    if matches!(value, AttrValue::Null) {
        return Ok(base.without(attr));
    }
    base.with(Attr {
        id: attr,
        value: value.clone(),
    })
    .ok_or_else(|| StepError::Unknown {
        path: Box::from("mark_attr"),
    })
}

/// Validates a mark attribute update against the mark declaration.
fn validate_mark_attr(
    schema: &Schema,
    mark: MarkId,
    attr: AttrId,
    value: &AttrValue,
) -> Result<(), StepError> {
    let spec = schema.mark(mark).ok_or_else(|| StepError::Unknown {
        path: Box::from("mark"),
    })?;
    let Some(declared) = spec.attrs.iter().find(|candidate| candidate.id == attr) else {
        return Err(StepError::Unknown {
            path: Box::from("mark_attr"),
        });
    };
    if matches!(value, AttrValue::Null) {
        if declared.required {
            return Err(StepError::Unknown {
                path: Box::from("mark_attr_required"),
            });
        }
        return Ok(());
    }
    if !Schema::value_matches(&declared.kind, value) {
        return Err(StepError::Unknown {
            path: Box::from("mark_attr_type"),
        });
    }
    Ok(())
}
