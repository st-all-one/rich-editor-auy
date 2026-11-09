//! Change sets derived from applied steps.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::boxed::Box;
use core::ops::Range as BlockRange;

use crate::model::Doc;
use crate::pos::{Pos, Range};
use crate::step::Step;

use super::change_builder::ChangeBuilder;

/// One old/new range pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeChange {
    /// Range in the old document.
    pub old: Range,
    /// Range in the new document.
    pub new: Range,
}

/// Sufficient information for a UI to repaint and announce a change.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "selection and structural flags are independent"
)]
pub struct ChangeSet {
    /// Top-level block range potentially affected in the new document.
    pub blocks: BlockRange<u32>,
    /// Ordered old/new ranges.
    pub ranges: Box<[RangeChange]>,
    /// Whether the selection changed.
    pub selection: bool,
    /// Whether the whole document was replaced.
    pub full: bool,
    /// New top-level block indices created by the change.
    pub inserted_blocks: Box<[u32]>,
    /// Old top-level block indices removed by the change.
    pub removed_blocks: Box<[u32]>,
}

impl ChangeSet {
    /// Creates an empty change set.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns whether no document content changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty() && !self.full
    }

    /// Derives a change set from steps applied between two documents.
    ///
    /// `old` and `new` are the documents before and after the steps.
    #[must_use]
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "the selection flag mirrors the transaction result"
    )]
    pub fn from_steps(old: &Doc, new: &Doc, steps: &[Step], selection_changed: bool) -> Self {
        let mut builder = ChangeBuilder::new();
        for step in steps {
            builder.push(step, old);
        }
        builder.finish(new, selection_changed)
    }

    /// Returns ranges as positions.
    pub fn old_ranges(&self) -> impl Iterator<Item = Range> + '_ {
        self.ranges.iter().map(|change| change.old)
    }

    /// Returns the first changed position, if any.
    #[must_use]
    pub fn first_old_position(&self) -> Option<Pos> {
        self.ranges.first().map(|change| change.old.start)
    }
}
