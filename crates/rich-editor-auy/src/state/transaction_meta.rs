//! Transaction metadata setters and composition.

use alloc::boxed::Box;

use crate::error::StepError;
use crate::model::GroupId;
use crate::pos::Range;
use crate::step::{Slice, Step};

use super::composition::{CompositionEvent, CompositionId};
use super::history::Origin;
use super::selection::Selection;
use super::transaction::Transaction;

impl Transaction {
    /// Applies an IME composition update.
    ///
    /// `Update` steps stay out of history; `Commit` re-enables recording so the accumulated
    /// steps form one undo group, and `Cancel` drops the steps added since `Start`.
    ///
    /// # Errors
    ///
    /// Returns an error when composition has not started or the composed text is too large.
    #[allow(
        clippy::too_many_lines,
        reason = "one exhaustive event handler keeps composition transitions visible"
    )]
    pub fn apply_composition(&mut self, event: CompositionEvent) -> Result<(), StepError> {
        match event {
            CompositionEvent::Start { at } => {
                if self.composition.is_some() {
                    return Err(StepError::Unsupported {
                        operation: Box::from("composition_already_started"),
                    });
                }
                if !at.is_ordered() {
                    return Err(StepError::OutOfBounds {
                        position: at.end.0,
                        size: at.start.0,
                    });
                }
                self.composition = Some(CompositionId(0));
                self.composition_range = Some(at);
                self.composition_step_start = self.steps.len();
                self.composition_selection = self.selection;
                self.origin = Origin::Composition(CompositionId(0));
                self.add_to_history = false;
            }
            CompositionEvent::Update { text } => {
                let Some(range) = self.composition_range else {
                    return Err(StepError::Unsupported {
                        operation: Box::from("composition_not_started"),
                    });
                };
                let Ok(length) = u32::try_from(text.len()) else {
                    return Err(StepError::Unsupported {
                        operation: Box::from("composition_text_limit"),
                    });
                };
                let Some(end) = range.start.checked_add(length) else {
                    return Err(StepError::Unsupported {
                        operation: Box::from("composition_text_limit"),
                    });
                };
                self.steps.push(Step::Replace {
                    from: range.start,
                    to: range.end,
                    slice: Slice::plain_text(&text),
                });
                self.composition_range = Some(Range::new(range.start, end));
                self.selection = Some(Selection::caret(end));
                self.add_to_history = false;
            }
            CompositionEvent::Commit => {
                if self.composition.is_some() {
                    self.add_to_history = true;
                }
                self.composition = None;
                self.composition_range = None;
                self.composition_step_start = self.steps.len();
                self.composition_selection = None;
            }
            CompositionEvent::Cancel => {
                if self.composition.is_some() {
                    self.steps.truncate(self.composition_step_start);
                    self.selection = self.composition_selection;
                    self.add_to_history = false;
                }
                self.composition = None;
                self.composition_range = None;
                self.composition_step_start = self.steps.len();
                self.composition_selection = None;
            }
        }
        Ok(())
    }

    /// Sets an explicit selection.
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = Some(selection);
    }

    /// Sets the transaction origin.
    pub fn set_origin(&mut self, origin: Origin) {
        self.origin = origin;
    }

    /// Sets the undo group.
    pub fn set_group(&mut self, group: GroupId) {
        self.group = Some(group);
    }

    /// Controls whether the transaction enters history.
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "single history switch mirrors the transaction flag"
    )]
    pub fn set_add_to_history(&mut self, add: bool) {
        self.add_to_history = add;
    }
}
