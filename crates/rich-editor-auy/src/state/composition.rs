//! IME composition values and state-level composition operations.
#![allow(
    clippy::redundant_pub_crate,
    reason = "composition snapshot is crate-scoped"
)]

use alloc::boxed::Box;

use crate::error::{EditError, StepError};
use crate::pos::{Pos, Range};
use crate::step::{Slice, Step};

use super::history::{Group, Origin};
use super::selection::Selection;
use super::{Applied, ApplyError, ChangeSet, EditorState, Transaction, step_bytes};

/// One IME composition identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositionId(pub u64);

/// Composition events supplied by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompositionEvent {
    /// Start at a range.
    Start {
        /// Replaced range.
        at: Range,
    },
    /// Update the composed text.
    Update {
        /// Current composed text.
        text: Box<str>,
    },
    /// Commit the composition as one undo group.
    Commit,
    /// Cancel and restore the original text.
    Cancel,
}

/// Active composition snapshot.
#[derive(Clone, Debug)]
pub(crate) struct Composition {
    pub(crate) id: CompositionId,
    pub(crate) range: Range,
    pub(crate) original: Slice,
    pub(crate) current: Box<str>,
    pub(crate) current_end: Pos,
    pub(crate) selection_before: Selection,
    pub(crate) changed: bool,
}

impl Composition {
    pub(crate) const fn current_range(&self) -> Range {
        Range::new(self.range.start, self.current_end)
    }
}

fn not_started() -> EditError {
    EditError::Step(StepError::Unsupported {
        operation: Box::from("composition_not_started"),
    })
}

fn unsupported(operation: &str) -> EditError {
    EditError::Step(StepError::Unsupported {
        operation: Box::from(operation),
    })
}

impl EditorState {
    /// Applies one host composition event.
    ///
    /// # Errors
    ///
    /// Returns an error when the range is invalid or no composition is active.
    pub fn apply_composition(&self, event: CompositionEvent) -> Result<Applied, ApplyError> {
        match event {
            CompositionEvent::Start { at } => self.begin_composition(at),
            CompositionEvent::Update { text } => self.update_composition(&text),
            CompositionEvent::Commit => self.commit_composition(),
            CompositionEvent::Cancel => self.cancel_composition(),
        }
    }

    /// Starts a composition, capturing the original range and its content.
    ///
    /// # Errors
    ///
    /// Returns an error when a composition is already active or the range is not a text range
    /// inside one text block.
    pub fn begin_composition(&self, at: Range) -> Result<Applied, ApplyError> {
        if self.composition.is_some() {
            return Err(unsupported("composition_already_started"));
        }
        if !at.is_ordered() || at.end.0 > self.doc.size().0 {
            return Err(EditError::Step(StepError::OutOfBounds {
                position: at.end.0,
                size: self.doc.size().0,
            }));
        }
        let Some((index, block)) = self.doc.block_at(at.start) else {
            return Err(EditError::Step(StepError::OutOfBounds {
                position: at.start.0,
                size: self.doc.size().0,
            }));
        };
        let Some(block_range) = self
            .doc
            .block_range(u32::try_from(index).unwrap_or(u32::MAX))
        else {
            return Err(unsupported("composition_block"));
        };
        let Some(text) = block.as_textblock() else {
            return Err(unsupported("composition_block"));
        };
        let content_start = block_range.start.0.saturating_add(1);
        let content_end = content_start.saturating_add(text.len().0);
        if at.start.0 < content_start || at.end.0 > content_end {
            return Err(unsupported("composition_range"));
        }
        let deletion = Step::Replace {
            from: at.start,
            to: at.end,
            slice: Slice::plain_text(""),
        };
        let inverse = deletion.invert(&self.doc)?;
        let Step::Replace {
            slice: original, ..
        } = inverse
        else {
            return Err(unsupported("composition_snapshot"));
        };
        let id = CompositionId(self.next_composition);
        let mut state = self.clone();
        state.next_composition = self.next_composition.saturating_add(1);
        state.composition = Some(Composition {
            id,
            range: at,
            original,
            current: Box::from(""),
            current_end: at.end,
            selection_before: self.selection,
            changed: false,
        });
        Ok(Applied {
            state,
            changes: ChangeSet::empty(),
        })
    }

    /// Replaces the active composed text without recording history.
    ///
    /// # Errors
    ///
    /// Returns an error when no composition is active or the text is too large.
    pub fn update_composition(&self, text: &str) -> Result<Applied, ApplyError> {
        let Some(composition) = self.composition.clone() else {
            return Err(not_started());
        };
        let Ok(length) = u32::try_from(text.len()) else {
            return Err(unsupported("composition_text_limit"));
        };
        let Some(end) = composition.range.start.checked_add(length) else {
            return Err(unsupported("composition_text_limit"));
        };
        let mut transaction = Transaction::with_base(self.doc.clone());
        transaction.replace(composition.current_range(), Slice::plain_text(text))?;
        transaction.set_selection(Selection::caret(end));
        transaction.set_origin(Origin::Composition(composition.id));
        transaction.set_add_to_history(false);
        transaction.composition = Some(composition.id);
        let mut applied = self.apply(transaction)?;
        applied.state.composition = Some(Composition {
            current: Box::from(text),
            current_end: end,
            changed: true,
            ..composition
        });
        Ok(applied)
    }

    /// Commits the composition as one undo entry.
    ///
    /// # Errors
    ///
    /// Returns an error when no composition is active.
    pub fn commit_composition(&self) -> Result<Applied, ApplyError> {
        let Some(composition) = self.composition.clone() else {
            return Err(not_started());
        };
        let selection_after = Selection::caret(composition.current_end);
        let mut state = self.clone();
        state.composition = None;
        if composition.changed {
            let forward = Step::Replace {
                from: composition.range.start,
                to: composition.range.end,
                slice: Slice::plain_text(&composition.current),
            };
            let inverse = Step::Replace {
                from: composition.range.start,
                to: composition.current_end,
                slice: composition.original,
            };
            let bytes = step_bytes(core::slice::from_ref(&forward))
                .saturating_add(step_bytes(core::slice::from_ref(&inverse)));
            state.history.record(Group {
                id: None,
                steps: alloc::vec![forward],
                inverse: alloc::vec![inverse],
                selection_before: composition.selection_before,
                selection_after,
                origin: Origin::Composition(composition.id),
                bytes,
            });
        }
        let selection = state.selection != selection_after;
        state.selection = selection_after;
        Ok(Applied {
            state,
            changes: ChangeSet {
                selection,
                ..ChangeSet::empty()
            },
        })
    }

    /// Cancels the composition, restoring the original text and selection.
    ///
    /// # Errors
    ///
    /// Returns an error when no composition is active or the restore step fails.
    pub fn cancel_composition(&self) -> Result<Applied, ApplyError> {
        let Some(composition) = self.composition.clone() else {
            return Err(not_started());
        };
        let mut transaction = Transaction::with_base(self.doc.clone());
        if composition.changed {
            transaction.replace(
                Range::new(composition.range.start, composition.current_end),
                composition.original,
            )?;
        }
        transaction.set_selection(composition.selection_before);
        transaction.set_origin(Origin::Composition(composition.id));
        transaction.set_add_to_history(false);
        transaction.composition = Some(composition.id);
        let mut applied = self.apply(transaction)?;
        applied.state.composition = None;
        Ok(applied)
    }

    /// Returns the active composition identifier.
    #[must_use]
    pub fn composition_id(&self) -> Option<CompositionId> {
        self.composition.as_ref().map(|composition| composition.id)
    }

    /// Returns whether a composition is active.
    #[must_use]
    pub fn is_composing(&self) -> bool {
        self.composition.is_some()
    }
}
