//! Transaction metadata setters and composition.

use crate::error::StepError;
use crate::model::GroupId;
use crate::step::{Slice, Step};

use super::composition::{CompositionEvent, CompositionId};
use super::history::Origin;
use super::selection::Selection;
use super::transaction::Transaction;

impl Transaction {
    /// Applies an IME composition update.
    ///
    /// # Errors
    ///
    /// Returns an error when composition has not started.
    pub fn apply_composition(&mut self, event: CompositionEvent) -> Result<(), StepError> {
        match event {
            CompositionEvent::Start { at } => {
                self.composition = Some(CompositionId(0));
                self.composition_range = Some(at);
                self.origin = Origin::Composition(CompositionId(0));
            }
            CompositionEvent::Update { text } => {
                let Some(range) = self.composition_range else {
                    return Err(StepError::Unsupported {
                        operation: alloc::boxed::Box::from("composition_not_started"),
                    });
                };
                self.steps.push(Step::Replace {
                    from: range.start,
                    to: range.end,
                    slice: Slice::plain_text(&text),
                });
                let length = u32::try_from(text.len()).unwrap_or(u32::MAX);
                let end = range.start.checked_add(length).unwrap_or(range.start);
                self.composition_range = Some(crate::pos::Range::new(range.start, end));
            }
            CompositionEvent::Commit | CompositionEvent::Cancel => {
                self.composition = None;
                self.composition_range = None;
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
