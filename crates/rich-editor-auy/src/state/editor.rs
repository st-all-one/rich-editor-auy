//! Stateful editor wrapper over [`EditorState`].

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::{EditError, SchemaError, SelectionError, StepError};
use crate::model::{Doc, Fragment};
use crate::pos::{Pos, Range};
use crate::schema::Schema;
use crate::wire::{ParseOptions, from_json as parse_document};

use super::{ApplyError, ChangeSet, EditorState, HistoryInfo, Selection, Transaction};

/// Observable result of one editor mutation.
///
/// The document itself is not copied into this value; hosts keep reading it through
/// [`Editor::doc`] and use `version` to decide when to rebuild a view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedInfo {
    /// Monotonic document version after the mutation.
    pub version: u64,
    /// Repaint and announcement information.
    pub changes: ChangeSet,
    /// Selection after the mutation.
    pub selection: Selection,
    /// History state after the mutation.
    pub history: HistoryInfo,
}

/// Construction options for an [`Editor`].
///
/// The default options use the state layer's standard history budgets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorOptions {
    /// Replacement `(max_groups, max_bytes)` history budgets.
    pub history_limits: Option<(usize, usize)>,
}

/// Stateful editor holding a document, selection, and history.
///
/// Every mutation bumps [`Editor::version`] when it changes the document, so hosts can
/// reconcile incrementally from [`AppliedInfo::changes`].
#[derive(Clone, Debug)]
pub struct Editor {
    pub(super) state: EditorState,
    pub(super) version: u64,
}

impl Editor {
    /// Creates an empty editor using the complete schema.
    ///
    /// # Errors
    ///
    /// Returns a schema error only if the full preset is inconsistent.
    pub fn new() -> Result<Self, SchemaError> {
        Self::with_options(EditorOptions::default())
    }

    /// Creates an editor from canonical document JSON using the complete schema.
    ///
    /// # Errors
    ///
    /// Returns a schema or parse error when the preset is inconsistent or the input is not a
    /// canonical document accepted by it.
    pub fn from_json(input: &str) -> Result<Self, EditError> {
        let schema = Arc::new(Schema::full()?);
        let doc = parse_document(input, &schema, &ParseOptions::default())?;
        Ok(Self {
            state: EditorState::from_doc(schema, doc),
            version: 0,
        })
    }

    /// Returns the current document.
    #[must_use]
    pub fn doc(&self) -> &Doc {
        self.state.doc()
    }

    /// Returns the current selection.
    #[must_use]
    pub fn selection(&self) -> Selection {
        self.state.selection()
    }

    /// Returns the current document version.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Returns the observable history state.
    #[must_use]
    pub fn history_info(&self) -> HistoryInfo {
        self.state.history_info()
    }

    /// Starts a transaction against the current document.
    #[must_use]
    pub fn transaction(&self) -> Transaction {
        self.state.transaction()
    }

    /// Sets the selection directly without creating history.
    ///
    /// # Errors
    ///
    /// Returns a selection error when a position lies beyond the document.
    pub fn set_selection(&mut self, selection: Selection) -> Result<(), SelectionError> {
        if selection.is_valid(self.state.doc().size().as_pos()) {
            self.state.selection = selection;
            Ok(())
        } else {
            Err(SelectionError {
                reason: Box::from("selection_out_of_bounds"),
                position: offending_position(selection),
            })
        }
    }

    /// Applies a transaction and returns its observable result.
    ///
    /// On error the editor is left unchanged.
    ///
    /// # Errors
    ///
    /// Propagates the typed error for an invalid step.
    pub fn apply(&mut self, transaction: Transaction) -> Result<AppliedInfo, ApplyError> {
        let state = self.state.clone();
        let applied = state.apply_owned(transaction)?;
        if !applied.state.doc().ptr_eq(self.state.doc()) {
            self.version = self.version.saturating_add(1);
        }
        self.state = applied.state;
        Ok(self.info(applied.changes))
    }

    /// Undoes the latest group.
    ///
    /// # Errors
    ///
    /// Propagates the typed error when an inverse step no longer applies.
    pub fn undo(&mut self) -> Result<Option<AppliedInfo>, ApplyError> {
        let Some(applied) = self.state.undo()? else {
            return Ok(None);
        };
        self.state = applied.state;
        self.version = self.version.saturating_add(1);
        Ok(Some(self.info(applied.changes)))
    }

    /// Redoes the latest undone group.
    ///
    /// # Errors
    ///
    /// Propagates the typed error when a step no longer applies.
    pub fn redo(&mut self) -> Result<Option<AppliedInfo>, ApplyError> {
        let Some(applied) = self.state.redo()? else {
            return Ok(None);
        };
        self.state = applied.state;
        self.version = self.version.saturating_add(1);
        Ok(Some(self.info(applied.changes)))
    }

    /// Discards every undo and redo group.
    pub fn clear_history(&mut self) {
        self.state = self.state.clear_history();
    }

    /// Replaces the whole document through a single history entry.
    ///
    /// # Errors
    ///
    /// Propagates the typed error for an invalid replacement.
    pub fn replace_document(&mut self, doc: Doc) -> Result<AppliedInfo, ApplyError> {
        let mut transaction = self.transaction();
        transaction.replace_document(doc)?;
        self.apply(transaction)
    }

    /// Replaces the top-level blocks covered by `range` with a fragment.
    ///
    /// Both endpoints must lie on top-level block boundaries; an empty range inserts the
    /// fragment at that boundary. The replacement is recorded as one document step.
    ///
    /// # Errors
    ///
    /// Returns an out-of-bounds or unsupported step error when the range is inverted or does
    /// not align with block boundaries, and propagates application errors otherwise.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "fragment values are consumed by the document replacement"
    )]
    pub fn replace_fragment(
        &mut self,
        range: Range,
        fragment: Fragment,
    ) -> Result<AppliedInfo, ApplyError> {
        let doc = self.state.doc();
        if range.start > range.end {
            return Err(StepError::OutOfBounds {
                position: range.start.0,
                size: doc.size().0,
            }
            .into());
        }
        let start = boundary_index(doc, range.start)?;
        let end = boundary_index(doc, range.end)?;
        let mut blocks = Vec::new();
        blocks.extend(doc.blocks().iter().take(start).cloned());
        blocks.extend_from_slice(fragment.blocks());
        blocks.extend(doc.blocks().iter().skip(end).cloned());
        let replacement = Doc::from_blocks(doc.attrs().clone(), blocks)?;
        self.replace_document(replacement)
    }

    pub(super) fn info(&self, changes: ChangeSet) -> AppliedInfo {
        AppliedInfo {
            version: self.version,
            changes,
            selection: self.state.selection(),
            history: self.state.history_info(),
        }
    }
}

/// Returns the furthest position a selection touches.
fn offending_position(selection: Selection) -> u32 {
    match selection {
        Selection::None | Selection::Table { .. } => 0,
        Selection::Caret { pos, .. } | Selection::Node { pos } => pos.0,
        Selection::Text { anchor, head, .. } => anchor.0.max(head.0),
    }
}

/// Returns the block boundary index for a position.
pub(super) fn boundary_index(doc: &Doc, pos: Pos) -> Result<usize, StepError> {
    let size = doc.size().0;
    if pos.0 > size {
        return Err(StepError::OutOfBounds {
            position: pos.0,
            size,
        });
    }
    let mut offset = 0_u32;
    for (index, block) in doc.blocks().iter().enumerate() {
        if offset == pos.0 {
            return Ok(index);
        }
        let end = offset.saturating_add(block.size().0);
        if end == pos.0 {
            return Ok(index.saturating_add(1));
        }
        offset = end;
    }
    if offset == pos.0 {
        Ok(doc.blocks().len())
    } else {
        Err(StepError::Unsupported {
            operation: Box::from("fragment_not_on_block_boundary"),
        })
    }
}
