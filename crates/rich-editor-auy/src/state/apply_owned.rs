//! Owned transaction application.

use alloc::vec::Vec;

use crate::step::{Mapping, apply_step_owned};

use super::history::Group;
use super::{Applied, ApplyError, ChangeSet, EditorState, Transaction, step_bytes};

impl EditorState {
    /// Applies a transaction by value, mutating the document in place when it is uniquely owned.
    ///
    /// # Errors
    ///
    /// Returns a typed error. The original state is not reused after an error.
    #[allow(
        clippy::too_many_lines,
        reason = "one pass applies steps and records history"
    )]
    pub fn apply_owned(self, mut transaction: Transaction) -> Result<Applied, ApplyError> {
        transaction.base = None;
        let Transaction {
            steps,
            selection: transaction_selection,
            stored_marks: transaction_stored_marks,
            origin,
            group,
            add_to_history,
            composition: transaction_composition,
            ..
        } = transaction;
        let Self {
            doc,
            selection: state_selection,
            stored_marks,
            schema,
            history,
            composition,
            next_composition,
        } = self;
        let mut doc = doc;
        let mut inverse = Vec::new();
        let mut mapping = Mapping::identity();
        for step in &steps {
            if step.is_noop() {
                continue;
            }
            inverse.push(step.invert(&doc)?);
            doc = apply_step_owned(doc, &schema, step)?;
            mapping = mapping.with_map(step.step_map());
        }
        let selection = transaction_selection.unwrap_or_else(|| state_selection.map(&mapping));
        let stored_marks = transaction_stored_marks.unwrap_or_else(|| stored_marks.clone());
        let composition = if transaction_composition.is_some() {
            composition
        } else {
            None
        };
        let mut state = Self {
            doc,
            selection,
            stored_marks,
            schema,
            history,
            composition,
            next_composition,
        };
        let changes = ChangeSet::from_steps(&state.doc, &steps, selection != state_selection);
        if add_to_history && !inverse.is_empty() {
            let bytes = step_bytes(&steps).saturating_add(step_bytes(&inverse));
            state.history.record(Group {
                id: group,
                steps,
                inverse,
                selection_before: state_selection,
                selection_after: selection,
                origin,
                bytes,
            });
        }
        Ok(Applied { state, changes })
    }
}
