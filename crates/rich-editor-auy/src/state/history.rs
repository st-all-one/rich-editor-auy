//! Deterministic undo and redo.
#![allow(
    clippy::redundant_pub_crate,
    reason = "internal algebra is crate-scoped"
)]
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::vec::Vec;

use crate::error::EditError;
use crate::model::Doc;
use crate::schema::Schema;
use crate::step::{Step, apply_step};

use super::selection::Selection;

/// Result of an undo or redo operation.
type HistoryResult = Result<Option<(Doc, Selection, Vec<Step>)>, EditError>;

/// Origin of a transaction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Origin {
    /// User input.
    #[default]
    User,
    /// Paste or drop.
    Paste,
    /// IME composition.
    Composition(super::CompositionId),
    /// Remote change.
    Remote,
    /// Programmatic API.
    Api,
}

/// One undo group.
#[derive(Clone, Debug)]
pub(crate) struct Group {
    pub(crate) steps: Vec<Step>,
    pub(crate) inverse: Vec<Step>,
    pub(crate) selection_before: Selection,
    pub(crate) selection_after: Selection,
    pub(crate) origin: Origin,
    pub(crate) bytes: usize,
}

/// Bounded linear history.
#[derive(Clone, Debug)]
pub struct History {
    done: Vec<Group>,
    undone: Vec<Group>,
    max_groups: usize,
    max_bytes: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(200, 8 * 1024 * 1024)
    }
}

impl History {
    /// Creates a history with explicit budgets.
    #[must_use]
    pub fn new(max_groups: usize, max_bytes: usize) -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            max_groups,
            max_bytes,
        }
    }

    /// Records one group and clears redo.
    pub(crate) fn record(&mut self, group: Group) {
        self.undone.clear();
        self.done.push(group);
        while self.done.len() > self.max_groups {
            self.done.remove(0);
        }
        let mut total = self.bytes();
        while total > self.max_bytes && self.done.len() > 1 {
            if self.done.is_empty() {
                break;
            }
            self.done.remove(0);
            total = self.bytes();
        }
    }

    /// Returns whether undo is possible.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    /// Returns whether redo is possible.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// Returns the origin of the latest undo group.
    #[must_use]
    pub fn last_origin(&self) -> Option<Origin> {
        self.done.last().map(|group| group.origin)
    }

    /// Undoes the latest group.
    ///
    /// # Errors
    ///
    /// Returns an error when an inverse step no longer applies.
    pub(crate) fn undo(&mut self, doc: &Doc, schema: &Schema) -> HistoryResult {
        let Some(group) = self.done.pop() else {
            return Ok(None);
        };
        let mut current = doc.clone();
        for step in group.inverse.iter().rev() {
            current = apply_step(&current, schema, step).map_err(EditError::Step)?;
        }
        let changes = group.inverse.clone();
        self.undone.push(group.clone());
        Ok(Some((current, group.selection_before, changes)))
    }

    /// Redoes the latest undone group.
    ///
    /// # Errors
    ///
    /// Returns an error when a step no longer applies.
    pub(crate) fn redo(&mut self, doc: &Doc, schema: &Schema) -> HistoryResult {
        let Some(group) = self.undone.pop() else {
            return Ok(None);
        };
        let mut current = doc.clone();
        for step in &group.steps {
            current = apply_step(&current, schema, step).map_err(EditError::Step)?;
        }
        let changes = group.steps.clone();
        self.done.push(group.clone());
        Ok(Some((current, group.selection_after, changes)))
    }

    fn bytes(&self) -> usize {
        self.done.iter().map(|group| group.bytes).sum()
    }
}
