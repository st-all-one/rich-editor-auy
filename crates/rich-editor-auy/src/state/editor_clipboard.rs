//! Editor-level clipboard and range operations.

use crate::error::StepError;
use crate::model::{Doc, MarkSet};
use crate::pos::{Pos, Range};
use crate::step::Slice;

use super::{
    AppliedInfo, ApplyError, ChangeSet, CompositionEvent, CompositionId, Editor, Selection,
};

impl Editor {
    /// Returns the schema this editor enforces.
    #[must_use]
    pub fn schema(&self) -> &crate::schema::Schema {
        self.state.schema()
    }

    /// Copies a range into a slice.
    ///
    /// # Errors
    ///
    /// Propagates the typed error for an invalid range or an edge that is not copyable.
    pub fn slice(&self, range: Range) -> Result<Slice, StepError> {
        self.doc().slice(range)
    }

    /// Deletes a range as one history entry and puts the caret where the content was.
    ///
    /// # Errors
    ///
    /// Propagates the typed error for an invalid range and application errors otherwise.
    pub fn delete_range(&mut self, range: Range) -> Result<AppliedInfo, ApplyError> {
        let doc = self.state.doc();
        let next = doc.delete_range(self.state.schema(), range)?;
        let caret = doc.caret_after_delete(range);
        let mut transaction = self.transaction();
        transaction.replace_document(next)?;
        if let Some(pos) = caret {
            transaction.set_selection(Selection::caret(pos));
        }
        self.apply(transaction)
    }

    /// Inserts a slice, replacing the range when one is given.
    ///
    /// A text range that a block slice cannot address directly deletes the range first, so a
    /// paste over a partial selection never leaves the two halves unmerged.
    ///
    /// # Errors
    ///
    /// Propagates the typed error for an invalid range or slice and application errors otherwise.
    pub fn insert_slice(&mut self, at: Pos, slice: &Slice) -> Result<AppliedInfo, ApplyError> {
        self.replace_slice(Range::new(at, at), slice)
    }

    /// Replaces a range with a slice as one history entry.
    ///
    /// # Errors
    ///
    /// Propagates the typed error for an invalid range or slice and application errors otherwise.
    pub fn replace_slice(
        &mut self,
        range: Range,
        slice: &Slice,
    ) -> Result<AppliedInfo, ApplyError> {
        let doc = self.state.doc();
        if range.start == range.end && slice.is_empty() {
            return Ok(self.info(ChangeSet::default()));
        }
        let next = if range.start == range.end {
            doc.insert_slice(self.state.schema(), range.start, slice)?
        } else if doc.same_text_block(range) || slice.content.is_empty() {
            doc.replace_slice(self.state.schema(), range, slice)?
        } else {
            let deleted = doc.delete_range(self.state.schema(), range)?;
            deleted.insert_slice(self.state.schema(), range.start, slice)?
        };
        let caret = caret_after_insert(&next, range.start, slice);
        let mut transaction = self.transaction();
        transaction.replace_document(next)?;
        transaction.set_selection(Selection::caret(caret));
        self.apply(transaction)
    }

    /// Returns the stored marks applied to the next insertion.
    #[must_use]
    pub fn stored_marks(&self) -> &MarkSet {
        self.state.stored_marks()
    }

    /// Returns the running composition id, when an IME composition is open.
    #[must_use]
    pub fn composition_id(&self) -> Option<CompositionId> {
        self.state.composition_id()
    }

    /// Returns whether an IME composition is open.
    #[must_use]
    pub fn is_composing(&self) -> bool {
        self.state.is_composing()
    }

    /// Applies one IME composition event as one history entry.
    ///
    /// # Errors
    ///
    /// Propagates the typed error for an out-of-order or oversized composition.
    pub fn apply_composition(
        &mut self,
        event: CompositionEvent,
    ) -> Result<AppliedInfo, ApplyError> {
        let applied = self.state.apply_composition(event)?;
        if !applied.state.doc().ptr_eq(self.state.doc()) {
            self.version = self.version.saturating_add(1);
        }
        self.state = applied.state;
        Ok(self.info(applied.changes))
    }
}

/// Returns the caret after inserting a slice at a position.
fn caret_after_insert(doc: &Doc, at: Pos, slice: &Slice) -> Pos {
    let size = doc.size().0;
    let next = at.0.saturating_add(slice.size());
    let pos = if next <= size { next } else { size };
    Pos(pos)
}
