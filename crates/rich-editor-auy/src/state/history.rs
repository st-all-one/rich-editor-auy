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
use crate::model::{Doc, GroupId};
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
    /// Structured ingestion from an external source.
    Import,
}

/// Observable history state for hosts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent undo and redo availability flags"
)]
pub struct HistoryInfo {
    /// Whether an undo group is available.
    pub can_undo: bool,
    /// Whether a redo group is available.
    pub can_redo: bool,
    /// Number of undoable groups.
    pub undo_depth: usize,
    /// Number of redoable groups.
    pub redo_depth: usize,
    /// Bytes retained by undo and redo groups.
    pub bytes: usize,
    /// Origin of the latest undoable group.
    pub last_origin: Option<Origin>,
}

/// One undo group.
#[derive(Clone, Debug)]
pub(crate) struct Group {
    pub(crate) id: Option<GroupId>,
    pub(crate) steps: Vec<Step>,
    pub(crate) inverse: Vec<Step>,
    pub(crate) selection_before: Selection,
    pub(crate) selection_after: Selection,
    pub(crate) origin: Origin,
    pub(crate) bytes: usize,
}

impl Group {
    /// Appends `next` so the pair undoes and redoes as one entry.
    fn absorb(&mut self, next: Self) {
        self.steps.extend(next.steps);
        self.inverse.extend(next.inverse);
        self.selection_after = next.selection_after;
        self.origin = next.origin;
        self.bytes = self.bytes.saturating_add(next.bytes);
    }
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
    ///
    /// `max_groups` of zero retains no undo groups. Under a byte budget at least one group is
    /// kept so the latest edit remains undoable.
    #[must_use]
    pub fn new(max_groups: usize, max_bytes: usize) -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            max_groups,
            max_bytes,
        }
    }

    /// Returns the configured group budget.
    #[must_use]
    pub const fn max_groups(&self) -> usize {
        self.max_groups
    }

    /// Returns the configured byte budget.
    #[must_use]
    pub const fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    /// Replaces the budgets and trims the retained undo groups.
    pub fn set_limits(&mut self, max_groups: usize, max_bytes: usize) {
        self.max_groups = max_groups;
        self.max_bytes = max_bytes;
        self.trim();
    }

    /// Discards every undo and redo group.
    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
    }

    /// Returns an observable snapshot.
    #[must_use]
    pub fn info(&self) -> HistoryInfo {
        HistoryInfo {
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
            undo_depth: self.done.len(),
            redo_depth: self.undone.len(),
            bytes: self.done_bytes().saturating_add(self.undone_bytes()),
            last_origin: self.last_origin(),
        }
    }

    /// Records one group, clearing redo and merging consecutive host groups.
    pub(crate) fn record(&mut self, group: Group) {
        self.undone.clear();
        if let Some(id) = group.id
            && let Some(last) = self.done.last_mut()
            && last.id == Some(id)
        {
            last.absorb(group);
            self.trim();
            return;
        }
        self.done.push(group);
        self.trim();
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

    fn done_bytes(&self) -> usize {
        self.done.iter().map(|group| group.bytes).sum()
    }

    fn undone_bytes(&self) -> usize {
        self.undone.iter().map(|group| group.bytes).sum()
    }

    fn trim(&mut self) {
        while self.done.len() > self.max_groups {
            self.done.remove(0);
        }
        if self.done.is_empty() {
            return;
        }
        let mut total = self.done_bytes();
        while total > self.max_bytes && self.done.len() > 1 {
            let removed = self.done.remove(0);
            total = total.saturating_sub(removed.bytes);
        }
    }
}

impl super::EditorState {
    /// Returns the observable history state.
    #[must_use]
    pub fn history_info(&self) -> HistoryInfo {
        self.history.info()
    }

    /// Returns the configured history budgets.
    #[must_use]
    pub fn history_limits(&self) -> (usize, usize) {
        (self.history.max_groups(), self.history.max_bytes())
    }

    /// Returns a state with every undo and redo group discarded.
    #[must_use]
    pub fn clear_history(&self) -> Self {
        let mut state = self.clone();
        state.history.clear();
        state
    }

    /// Returns a state with replacement history budgets.
    #[must_use]
    pub fn with_history_limits(&self, max_groups: usize, max_bytes: usize) -> Self {
        let mut state = self.clone();
        state.history.set_limits(max_groups, max_bytes);
        state
    }
}
