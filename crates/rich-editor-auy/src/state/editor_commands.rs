//! Selection-oriented commands on the stateful editor.

use crate::command;
use crate::error::{EditError, StepError};
use crate::model::{Attrs, Mark, NodeKind};

use super::{AppliedInfo, Editor};

/// Builds the error returned when a command has no applicable selection.
fn no_selection(operation: &str) -> EditError {
    EditError::Step(StepError::Unsupported {
        operation: alloc::boxed::Box::from(operation),
    })
}

impl Editor {
    /// Inserts text at the selection, replacing it first.
    ///
    /// # Errors
    ///
    /// Returns an error when the selection is missing or the insertion is invalid.
    pub fn insert_text(&mut self, text: &str) -> Result<AppliedInfo, EditError> {
        let transaction =
            command::insert_text(&self.state, text).ok_or_else(|| no_selection("insert_text"))?;
        self.apply(transaction)
    }

    /// Deletes one scalar before the caret or the whole selection.
    ///
    /// # Errors
    ///
    /// Returns an error when the deletion or the join is invalid.
    pub fn delete_backward(&mut self) -> Result<AppliedInfo, EditError> {
        let transaction =
            command::delete_backward(&self.state).ok_or_else(|| no_selection("delete_backward"))?;
        self.apply(transaction)
    }

    /// Deletes one scalar after the caret or the whole selection.
    ///
    /// # Errors
    ///
    /// Returns an error when the deletion or the join is invalid.
    pub fn delete_forward(&mut self) -> Result<AppliedInfo, EditError> {
        let transaction =
            command::delete_forward(&self.state).ok_or_else(|| no_selection("delete_forward"))?;
        self.apply(transaction)
    }

    /// Deletes the selected range.
    ///
    /// # Errors
    ///
    /// Returns an error when the selection is empty or the deletion is invalid.
    pub fn delete_selection(&mut self) -> Result<AppliedInfo, EditError> {
        let transaction = command::delete_selection(&self.state)
            .ok_or_else(|| no_selection("delete_selection"))?;
        self.apply(transaction)
    }

    /// Splits the block containing the selection head.
    ///
    /// # Errors
    ///
    /// Returns an error when the head is not inside a text block.
    pub fn split_block(&mut self) -> Result<AppliedInfo, EditError> {
        let transaction =
            command::split_block(&self.state).ok_or_else(|| no_selection("split_block"))?;
        self.apply(transaction)
    }

    /// Inserts a hard break at the selection.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema has no `hard_break` node or the insertion is invalid.
    pub fn insert_hard_break(&mut self) -> Result<AppliedInfo, EditError> {
        let transaction =
            command::insert_hard_break(&self.state).ok_or_else(|| no_selection("hard_break"))?;
        self.apply(transaction)
    }

    /// Changes the type of the block containing the selection head.
    ///
    /// # Errors
    ///
    /// Returns an error when the target cannot hold the block content.
    pub fn set_block_type(
        &mut self,
        kind: NodeKind,
        attrs: Attrs,
    ) -> Result<AppliedInfo, EditError> {
        let transaction = command::set_block_type(&self.state, kind, attrs)
            .ok_or_else(|| no_selection("set_block_type"))?;
        self.apply(transaction)
    }

    /// Toggles a mark on the selection or in the stored marks.
    ///
    /// # Errors
    ///
    /// Returns an error when the mark is not declared or the step is invalid.
    pub fn toggle_mark(&mut self, mark: Mark) -> Result<AppliedInfo, EditError> {
        let transaction =
            command::toggle_mark(&self.state, mark).ok_or_else(|| no_selection("toggle_mark"))?;
        self.apply(transaction)
    }
}
