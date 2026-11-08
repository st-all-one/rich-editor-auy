//! Built-in commands expressed as transactions.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::boxed::Box;

use crate::error::StepError;
use crate::model::Mark;
use crate::pos::{Pos, Range};
use crate::state::{EditorState, Selection, Transaction};

/// Inserts text at the selection, replacing a non-empty selection first.
#[must_use]
pub fn insert_text(state: &EditorState, text: &str) -> Option<Transaction> {
    let range = state.selection().range()?;
    let mut transaction = state.transaction();
    if !range.is_empty() && transaction.delete(range).is_err() {
        return None;
    }
    if transaction.insert_text(range.start, text).is_err() {
        return None;
    }
    let end = range
        .start
        .checked_add(u32::try_from(text.len()).unwrap_or(u32::MAX))
        .unwrap_or(range.start);
    transaction.set_selection(Selection::caret(end));
    Some(transaction)
}

/// Deletes the selected range.
#[must_use]
pub fn delete_selection(state: &EditorState) -> Option<Transaction> {
    let range = state.selection().range()?;
    if range.is_empty() {
        return None;
    }
    let mut transaction = state.transaction();
    if transaction.delete(range).is_err() {
        return None;
    }
    transaction.set_selection(Selection::caret(range.start));
    Some(transaction)
}

/// Toggles a mark on the selection or in the stored marks.
#[must_use]
pub fn toggle_mark(state: &EditorState, mark: Mark) -> Option<Transaction> {
    let mut transaction = state.transaction();
    let selection = state.selection().range();
    let active = match selection.filter(|range| !range.is_empty()) {
        Some(range) => range_has_mark(state, range, mark.id),
        None => state.stored_marks().contains(mark.id),
    };
    if let Some(range) = selection.filter(|range| !range.is_empty()) {
        let result = if active {
            transaction.remove_mark(range, mark)
        } else {
            transaction.add_mark(range, mark)
        };
        if result.is_err() {
            return None;
        }
        return Some(transaction);
    }
    if active {
        transaction
            .remove_mark(Range::new(Pos::ZERO, Pos::ZERO), mark)
            .ok()?;
    } else {
        transaction.set_stored_mark(mark);
    }
    Some(transaction)
}

/// Splits the current block.
#[must_use]
pub fn split_block(state: &EditorState) -> Option<Transaction> {
    let head = state.selection().head()?;
    let mut transaction = state.transaction();
    transaction.split_block(head).ok()?;
    Some(transaction)
}

/// Selects the whole document.
#[must_use]
#[allow(
    clippy::unnecessary_wraps,
    reason = "command API returns Option for uniform dispatch"
)]
pub fn select_all(state: &EditorState) -> Option<Transaction> {
    let end = Pos(state.doc().size().0);
    let mut transaction = state.transaction();
    transaction.set_selection(Selection::text(Pos::ZERO, end));
    Some(transaction)
}

/// Returns a typed error for commands that the MVP algebra cannot express.
#[must_use]
pub fn unsupported(name: &str) -> StepError {
    StepError::Unsupported {
        operation: Box::from(name),
    }
}

fn range_has_mark(state: &EditorState, range: Range, id: crate::model::MarkId) -> bool {
    let mut found = false;
    for (index, block) in state.doc().blocks().iter().enumerate() {
        let Some(block_range) = state
            .doc()
            .block_range(u32::try_from(index).unwrap_or(u32::MAX))
        else {
            continue;
        };
        if block_range.end.0 <= range.start.0 || block_range.start.0 >= range.end.0 {
            continue;
        }
        let Some(text) = block.as_textblock() else {
            continue;
        };
        let content_start = block_range.start.0.saturating_add(1);
        let local_start = range.start.0.saturating_sub(content_start);
        let local_end = range.end.0.saturating_sub(content_start).min(text.len().0);
        let mut cursor = 0_u32;
        for item in text.items() {
            let size = match item {
                crate::model::InlineItem::Text(run) => {
                    u32::try_from(run.text.len()).unwrap_or(u32::MAX)
                }
                crate::model::InlineItem::Atom(_) => 1,
            };
            let item_end = cursor.saturating_add(size);
            if item_end > local_start && cursor < local_end {
                let crate::model::InlineItem::Text(run) = item else {
                    return false;
                };
                if !run.marks.contains(id) {
                    return false;
                }
                found = true;
            }
            cursor = item_end;
        }
    }
    found
}
