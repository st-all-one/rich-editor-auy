//! Fast insertion into a single unmarked run.
#![allow(
    clippy::redundant_pub_crate,
    reason = "internal algebra is crate-scoped"
)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::model::{InlineItem, MarkSet, Run, TextBlock};

/// Inserts `insert` inside a single unmarked run.
///
/// Returns `None` when the offset is not inside an unmarked run, when the result would exceed
/// [`crate::model::MAX_RUN_BYTES`], or when the offset is not a UTF-8 boundary, so every other
/// case keeps the generic run-splitting path unchanged.
pub(super) fn insert_into_plain_run(
    block: &TextBlock,
    at: usize,
    insert: &str,
) -> Option<TextBlock> {
    if insert.is_empty() {
        return None;
    }
    let mut cursor = 0_usize;
    let mut target = None;
    for (index, item) in block.items().iter().enumerate() {
        match item {
            InlineItem::Text(run) => {
                let run_start = cursor;
                cursor = cursor.checked_add(run.text.len())?;
                if at >= run_start && at < cursor && run.marks.is_empty() {
                    target = Some((index, at.saturating_sub(run_start)));
                    break;
                }
            }
            InlineItem::Atom(_) => cursor = cursor.checked_add(1)?,
        }
    }
    let (index, local) = target?;
    let run = match block.items().get(index)? {
        InlineItem::Text(run) => run,
        InlineItem::Atom(_) => return None,
    };
    if run.text.len().saturating_add(insert.len()) > crate::model::MAX_RUN_BYTES {
        return None;
    }
    let prefix = run.text.get(..local)?;
    let suffix = run.text.get(local..)?;
    let mut text = String::with_capacity(run.text.len().saturating_add(insert.len()));
    text.push_str(prefix);
    text.push_str(insert);
    text.push_str(suffix);
    let mut items = Vec::with_capacity(block.items().len());
    for (position, item) in block.items().iter().enumerate() {
        if position == index {
            items.push(InlineItem::Text(Run::new(
                core::mem::take(&mut text),
                MarkSet::empty(),
            )));
        } else {
            items.push(item.clone());
        }
    }
    Some(TextBlock::new(items))
}
