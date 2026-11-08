//! Immutable editor state, transactions, and history.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

mod apply_owned;
mod change;
mod composition;
mod history;
mod selection;
mod transaction;
mod transaction_join;
mod transaction_meta;

pub use change::{ChangeSet, RangeChange};
pub use composition::{CompositionEvent, CompositionId};
pub use history::{History, Origin};
pub use selection::Selection;
pub use transaction::Transaction;

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::EditError;
use crate::model::{Attrs, Block, Doc, MarkSet, TextBlock};
use crate::pos::{Pos, Range};
use crate::schema::Schema;
use crate::step::{Mapping, Step, apply_step};

use history::Group;

/// Error returned by state application.
pub type ApplyError = EditError;

/// New state plus the changes that produced it.
#[derive(Clone, Debug)]
pub struct Applied {
    /// New state.
    pub state: EditorState,
    /// Repaint and announcement information.
    pub changes: ChangeSet,
}

/// Immutable editor state.
#[derive(Clone, Debug)]
pub struct EditorState {
    doc: Doc,
    selection: Selection,
    stored_marks: MarkSet,
    schema: Arc<Schema>,
    history: History,
    composition: Option<CompositionId>,
}

impl EditorState {
    /// Creates an empty state for a schema.
    ///
    /// # Errors
    ///
    /// Returns an error only if the empty document violates configured limits.
    pub fn new(schema: Arc<Schema>) -> Result<Self, crate::error::SchemaError> {
        let mut blocks = Vec::new();
        if let Some(kind) = schema
            .group_kinds("block")
            .and_then(|kinds| kinds.first())
            .copied()
        {
            blocks.push(Arc::new(Block::text(
                kind,
                Attrs::empty(),
                TextBlock::empty(),
            )));
        }
        let doc = Doc::from_blocks(Attrs::empty(), blocks).map_err(|_| {
            crate::error::SchemaError::A11y {
                code: alloc::boxed::Box::from("initial_doc_limit"),
            }
        })?;
        Ok(Self {
            doc,
            selection: Selection::None,
            stored_marks: MarkSet::empty(),
            schema,
            history: History::default(),
            composition: None,
        })
    }

    /// Creates a state around an existing document.
    #[must_use]
    pub fn from_doc(schema: Arc<Schema>, doc: Doc) -> Self {
        Self {
            doc,
            selection: Selection::None,
            stored_marks: MarkSet::empty(),
            schema,
            history: History::default(),
            composition: None,
        }
    }

    /// Returns the document.
    #[must_use]
    pub fn doc(&self) -> &Doc {
        &self.doc
    }

    /// Returns the selection.
    #[must_use]
    pub const fn selection(&self) -> Selection {
        self.selection
    }

    /// Returns the stored marks.
    #[must_use]
    pub const fn stored_marks(&self) -> &MarkSet {
        &self.stored_marks
    }

    /// Returns the schema.
    #[must_use]
    pub const fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    /// Starts a transaction.
    #[must_use]
    #[allow(clippy::unused_self, reason = "transaction is part of the state API")]
    pub fn transaction(&self) -> Transaction {
        Transaction::with_base(self.doc.clone())
    }

    /// Applies a transaction without mutating the original state.
    ///
    /// # Errors
    ///
    /// Returns a typed error and leaves the original state intact.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "transactions are consumed by design"
    )]
    pub fn apply(&self, transaction: Transaction) -> Result<Applied, ApplyError> {
        let mut doc = self.doc.clone();
        let mut inverse = Vec::new();
        let mut mapping = Mapping::identity();
        for step in transaction.steps() {
            if step.is_noop() {
                continue;
            }
            inverse.push(step.invert(&doc)?);
            doc = apply_step(&doc, &self.schema, step)?;
            mapping = mapping.with_map(step.step_map());
        }
        let selection = transaction
            .selection()
            .unwrap_or_else(|| self.selection.map(&mapping));
        let stored_marks = transaction
            .stored_marks()
            .cloned()
            .unwrap_or_else(|| self.stored_marks.clone());
        let mut state = Self {
            doc,
            selection,
            stored_marks,
            schema: Arc::clone(&self.schema),
            history: self.history.clone(),
            composition: transaction.composition().or(self.composition),
        };
        if transaction.adds_to_history() && !inverse.is_empty() {
            let bytes = step_bytes(&inverse);
            state.history.record(Group {
                steps: transaction.steps().to_vec(),
                inverse,
                selection_before: self.selection,
                selection_after: selection,
                origin: transaction.origin(),
                bytes,
            });
        }
        let changes =
            ChangeSet::from_steps(&state.doc, transaction.steps(), selection != self.selection);
        Ok(Applied { state, changes })
    }

    /// Returns whether undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Returns whether redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Undoes the latest group.
    ///
    /// # Errors
    ///
    /// Returns an error when an inverse step no longer applies.
    pub fn undo(&self) -> Result<Option<Applied>, ApplyError> {
        let mut history = self.history.clone();
        let Some((doc, selection, steps)) = history.undo(&self.doc, &self.schema)? else {
            return Ok(None);
        };
        let mut state = self.clone();
        state.doc = doc;
        state.selection = selection;
        state.history = history;
        let changes = ChangeSet::from_steps(&state.doc, &steps, true);
        Ok(Some(Applied { state, changes }))
    }

    /// Redoes the latest undone group.
    ///
    /// # Errors
    ///
    /// Returns an error when a step no longer applies.
    pub fn redo(&self) -> Result<Option<Applied>, ApplyError> {
        let mut history = self.history.clone();
        let Some((doc, selection, steps)) = history.redo(&self.doc, &self.schema)? else {
            return Ok(None);
        };
        let mut state = self.clone();
        state.doc = doc;
        state.selection = selection;
        state.history = history;
        let changes = ChangeSet::from_steps(&state.doc, &steps, true);
        Ok(Some(Applied { state, changes }))
    }

    /// Maps a selection through a range.
    #[must_use]
    pub fn map_selection(&self, mapping: &Mapping) -> Selection {
        self.selection.map(mapping)
    }

    /// Clamps a range to the document size.
    #[must_use]
    pub fn clamp(&self, range: Range) -> Range {
        range.clamp(Pos(self.doc.size().0))
    }
}

fn step_bytes(steps: &[Step]) -> usize {
    let mut total = 0_usize;
    for step in steps {
        total = total.saturating_add(match step {
            Step::Replace { slice, .. } | Step::ReplaceAround { slice, .. } => slice
                .text
                .len()
                .saturating_add(usize::try_from(slice.content.size().0).unwrap_or(usize::MAX)),
            Step::AddMark { .. }
            | Step::RemoveMark { .. }
            | Step::SetNodeAttr { .. }
            | Step::SetDocAttr { .. } => 0,
        });
    }
    total
}
