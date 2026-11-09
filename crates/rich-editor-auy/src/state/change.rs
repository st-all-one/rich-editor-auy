//! Change sets derived from applied steps.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::model::Doc;
use crate::pos::{Pos, Range};
use crate::step::Step;

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
    /// Top-level block range potentially affected.
    pub blocks: core::ops::Range<u32>,
    /// Ordered old/new ranges.
    pub ranges: Box<[RangeChange]>,
    /// Whether the selection changed.
    pub selection: bool,
    /// Structural invalidation flag.
    pub full: bool,
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

    /// Derives a change set from applied steps.
    #[must_use]
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "selection flag mirrors the transaction result"
    )]
    pub fn from_steps(doc: &Doc, steps: &[Step], selection_changed: bool) -> Self {
        let mut ranges = Vec::new();
        let mut full = false;
        for step in steps {
            match step {
                Step::Replace { from, to, slice } => {
                    let new_len = if !slice.content.is_empty() {
                        slice.content.size().0
                    } else if let Some(runs) = &slice.runs {
                        runs.len().0
                    } else {
                        u32::try_from(slice.text.len()).unwrap_or(u32::MAX)
                    };
                    ranges.push(RangeChange {
                        old: Range::new(*from, *to),
                        new: Range::new(*from, Pos(from.0.saturating_add(new_len))),
                    });
                }
                Step::ReplaceAround {
                    from,
                    to,
                    insert,
                    slice,
                    ..
                } => {
                    full = true;
                    let new_len = u32::try_from(slice.text.len())
                        .unwrap_or(u32::MAX)
                        .saturating_add(*insert);
                    ranges.push(RangeChange {
                        old: Range::new(*from, *to),
                        new: Range::new(*from, Pos(from.0.saturating_add(new_len))),
                    });
                }
                Step::AddMark { from, to, .. }
                | Step::RemoveMark { from, to, .. }
                | Step::SetInlineAttr { from, to, .. }
                | Step::SetMarkAttr { from, to, .. } => {
                    if from != to {
                        ranges.push(RangeChange {
                            old: Range::new(*from, *to),
                            new: Range::new(*from, *to),
                        });
                    }
                }
                Step::SetNodeAttr { .. } | Step::SetDocAttr { .. } => {}
                Step::ReplaceDoc { .. } => {
                    full = true;
                }
            }
        }
        ranges.sort_by_key(|change| (change.old.start.0, change.new.start.0));
        full = full || ranges.len() > 1;
        let block_count = u32::try_from(doc.blocks().len()).unwrap_or(u32::MAX);
        Self {
            blocks: 0..block_count,
            ranges: ranges.into_boxed_slice(),
            selection: selection_changed,
            full,
        }
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
