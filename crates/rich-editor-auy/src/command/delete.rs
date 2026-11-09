//! Character-oriented deletion commands.

use alloc::string::String;

use crate::model::Doc;
use crate::pos::{Pos, Range};
use crate::state::{EditorState, Selection, Transaction};

use super::delete_selection;

/// Deletes the scalar character or the selection before the caret.
///
/// At the start of a block it joins the block with its previous sibling.
#[must_use]
pub fn delete_backward(state: &EditorState) -> Option<Transaction> {
    let range = state.selection().range()?;
    if !range.is_empty() {
        return delete_selection(state);
    }
    let head = range.start;
    let mut transaction = state.transaction();
    match scalar_before(state.doc(), head) {
        Some(previous) => {
            transaction.delete(Range::new(previous, head)).ok()?;
            transaction.set_selection(Selection::caret(previous));
        }
        None => transaction
            .join_backward(block_start(state.doc(), head)?)
            .ok()?,
    }
    Some(transaction)
}

/// Deletes the scalar character or the selection after the caret.
///
/// At the end of a block it joins the next block into the current one.
#[must_use]
pub fn delete_forward(state: &EditorState) -> Option<Transaction> {
    let range = state.selection().range()?;
    if !range.is_empty() {
        return delete_selection(state);
    }
    let head = range.start;
    let mut transaction = state.transaction();
    if let Some(next) = scalar_after(state.doc(), head) {
        transaction.delete(Range::new(head, next)).ok()?;
        transaction.set_selection(Selection::caret(head));
    } else {
        let (index, _) = state.doc().block_at(head)?;
        let next = u32::try_from(index).ok()?.checked_add(1)?;
        let next_range = state.doc().block_range(next)?;
        transaction.join_backward(next_range.start).ok()?;
    }
    Some(transaction)
}

/// Returns the start position of the block containing a position.
fn block_start(doc: &Doc, pos: Pos) -> Option<Pos> {
    let (index, _) = doc.block_at(pos)?;
    let range = doc.block_range(u32::try_from(index).ok()?)?;
    Some(range.start)
}

/// Returns the previous scalar boundary inside the block, if any.
fn scalar_before(doc: &Doc, pos: Pos) -> Option<Pos> {
    let (index, block) = doc.block_at(pos)?;
    let range = doc.block_range(u32::try_from(index).ok()?)?;
    let text_start = range.start.checked_add(1)?;
    if pos <= text_start {
        return None;
    }
    let content = text_of(block.as_textblock()?);
    let local = usize::try_from(pos.0.checked_sub(text_start.0)?).ok()?;
    let local = local.min(content.len());
    let mut boundary = local.saturating_sub(1);
    while boundary > 0 && !content.is_char_boundary(boundary) {
        boundary = boundary.saturating_sub(1);
    }
    text_start.checked_add(u32::try_from(boundary).ok()?)
}

/// Returns the next scalar boundary inside the block, if any.
fn scalar_after(doc: &Doc, pos: Pos) -> Option<Pos> {
    let (index, block) = doc.block_at(pos)?;
    let range = doc.block_range(u32::try_from(index).ok()?)?;
    let text_start = range.start.checked_add(1)?;
    let content = text_of(block.as_textblock()?);
    let text_end = text_start.checked_add(u32::try_from(content.len()).ok()?)?;
    if pos >= text_end {
        return None;
    }
    let local = usize::try_from(pos.0.checked_sub(text_start.0)?)
        .ok()?
        .min(content.len());
    let mut boundary = local.saturating_add(1);
    while boundary < content.len() && !content.is_char_boundary(boundary) {
        boundary = boundary.saturating_add(1);
    }
    text_start.checked_add(u32::try_from(boundary).ok()?)
}

/// Returns the plain text of a text block.
fn text_of(block: &crate::model::TextBlock) -> String {
    let mut text = String::new();
    block.text_into(&mut text);
    text
}
