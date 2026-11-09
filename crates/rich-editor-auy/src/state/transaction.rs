//! Explicit edit transactions.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{AttrId, AttrValue, Doc, GroupId, Mark, MarkSet};
use crate::pos::{Pos, Range};
use crate::step::{Slice, Step};

use super::composition::CompositionId;
use super::history::Origin;
use super::selection::Selection;

/// A sequence of steps plus selection/history metadata.
#[derive(Clone, Debug)]
pub struct Transaction {
    pub(super) steps: Vec<Step>,
    pub(super) selection: Option<Selection>,
    pub(super) stored_marks: Option<MarkSet>,
    pub(super) origin: Origin,
    pub(super) group: Option<GroupId>,
    pub(super) add_to_history: bool,
    pub(super) composition: Option<CompositionId>,
    pub(super) composition_range: Option<Range>,
    pub(super) composition_step_start: usize,
    pub(super) composition_selection: Option<Selection>,
    pub(super) base: Option<Doc>,
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "constructors share a fallible step API"
)]
impl Transaction {
    /// Creates an empty transaction.
    #[must_use]
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            selection: None,
            stored_marks: None,
            origin: Origin::User,
            group: None,
            add_to_history: true,
            composition: None,
            composition_range: None,
            composition_step_start: 0,
            composition_selection: None,
            base: None,
        }
    }

    pub(crate) fn with_base(doc: Doc) -> Self {
        let mut transaction = Self::new();
        transaction.base = Some(doc);
        transaction
    }

    /// Inserts text at a position.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is invalid for the current step algebra.
    pub fn insert_text(&mut self, at: Pos, text: &str) -> Result<(), StepError> {
        self.steps.push(Step::Replace {
            from: at,
            to: at,
            slice: Slice::plain_text(text),
        });
        Ok(())
    }

    /// Deletes a range.
    ///
    /// # Errors
    ///
    /// Returns an error when the range is inverted.
    pub fn delete(&mut self, range: Range) -> Result<(), StepError> {
        if range.start > range.end {
            return Err(StepError::OutOfBounds {
                position: range.start.0,
                size: range.end.0,
            });
        }
        self.steps.push(Step::Replace {
            from: range.start,
            to: range.end,
            slice: Slice::plain_text(""),
        });
        Ok(())
    }

    /// Replaces a range with a slice.
    ///
    /// # Errors
    ///
    /// Returns an error when the range is inverted.
    pub fn replace(&mut self, range: Range, slice: Slice) -> Result<(), StepError> {
        if range.start > range.end {
            return Err(StepError::OutOfBounds {
                position: range.start.0,
                size: range.end.0,
            });
        }
        self.steps.push(Step::Replace {
            from: range.start,
            to: range.end,
            slice,
        });
        Ok(())
    }

    /// Splits the block containing a position.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is semantic.
    pub fn split_block(&mut self, at: Pos) -> Result<(), StepError> {
        self.steps.push(Step::ReplaceAround {
            from: at,
            to: at,
            gap_from: at,
            gap_to: at,
            insert: 2,
            slice: Slice::plain_text(""),
        });
        Ok(())
    }

    /// Adds a mark to a range, or stores it for the next insertion when the range is empty.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid ranges.
    pub fn add_mark(&mut self, range: Range, mark: Mark) -> Result<(), StepError> {
        if range.is_empty() {
            self.set_stored_mark(mark);
            return Ok(());
        }
        self.steps.push(Step::AddMark {
            from: range.start,
            to: range.end,
            mark,
        });
        Ok(())
    }

    /// Removes a mark from a range.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid ranges.
    pub fn remove_mark(&mut self, range: Range, mark: Mark) -> Result<(), StepError> {
        if range.is_empty() {
            let current = self.stored_marks.take().unwrap_or_else(MarkSet::empty);
            self.stored_marks = Some(current.without_mark(mark.id));
            return Ok(());
        }
        self.steps.push(Step::RemoveMark {
            from: range.start,
            to: range.end,
            mark,
        });
        Ok(())
    }

    /// Sets the stored mark set.
    pub fn set_stored_marks(&mut self, marks: MarkSet) {
        self.stored_marks = Some(marks);
    }

    /// Stores one mark for the next insertion.
    pub fn set_stored_mark(&mut self, mark: Mark) {
        let current = self.stored_marks.take().unwrap_or_else(MarkSet::empty);
        self.stored_marks = Some(current.with_mark(mark));
    }

    /// Sets a node attribute.
    ///
    /// # Errors
    ///
    /// This constructor is infallible; validation happens in `apply`.
    pub fn set_node_attr(
        &mut self,
        at: Pos,
        attr: AttrId,
        value: AttrValue,
    ) -> Result<(), StepError> {
        self.steps.push(Step::SetNodeAttr {
            pos: at,
            attr,
            value,
        });
        Ok(())
    }

    /// Sets the reserved node id at a position.
    ///
    /// # Errors
    ///
    /// Returns an error when the id is not valid.
    pub fn set_node_id(&mut self, at: Pos, id: Option<&str>) -> Result<(), StepError> {
        let value = match id {
            Some(id) => AttrValue::Str(alloc::boxed::Box::from(id)),
            None => AttrValue::Null,
        };
        self.set_node_attr(at, AttrId::NODE_ID, value)
    }

    /// Sets a document-root attribute.
    ///
    /// # Errors
    ///
    /// This constructor is infallible; validation happens in `apply`.
    pub fn set_doc_attr(&mut self, attr: AttrId, value: AttrValue) -> Result<(), StepError> {
        self.steps.push(Step::SetDocAttr { attr, value });
        Ok(())
    }

    /// Returns the accumulated steps.
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Returns the explicit selection.
    #[must_use]
    pub fn selection(&self) -> Option<Selection> {
        self.selection
    }

    /// Returns the stored mark override.
    #[must_use]
    pub fn stored_marks(&self) -> Option<&MarkSet> {
        self.stored_marks.as_ref()
    }

    /// Returns the origin.
    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// Returns the group.
    #[must_use]
    pub fn group(&self) -> Option<GroupId> {
        self.group
    }

    /// Returns whether the transaction enters history.
    #[must_use]
    pub fn adds_to_history(&self) -> bool {
        self.add_to_history
    }

    /// Returns the composition id.
    #[must_use]
    pub fn composition(&self) -> Option<CompositionId> {
        self.composition
    }

    /// Returns whether the transaction is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}
