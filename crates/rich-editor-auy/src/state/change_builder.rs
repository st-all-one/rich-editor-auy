//! Change set accumulation across applied steps.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Range as BlockRange;

use crate::model::Doc;
use crate::pos::{Assoc, Pos, Range};
use crate::step::Step;

use super::change::{ChangeSet, RangeChange};

/// Accumulates a change set while steps are applied in order.
pub(super) struct ChangeBuilder {
    window: Option<Range>,
    ranges: Vec<RangeChange>,
    structural: Option<(Range, BlockRange<u32>)>,
    structural_final: Option<Range>,
    full: bool,
}

impl ChangeBuilder {
    /// Creates a builder for a transaction.
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            ranges: Vec::new(),
            structural: None,
            structural_final: None,
            full: false,
        }
    }

    /// Records one step against the document it will be applied to.
    pub(crate) fn push(&mut self, step: &Step, old: &Doc) {
        if step.is_noop() {
            return;
        }
        let step_range = step_range(step);
        if let Some(range) = step_range {
            self.window = Some(union(self.window, range));
        }
        match step {
            Step::ReplaceDoc { .. } => self.full = true,
            Step::ReplaceAround { .. } => {
                self.full |= self.register_structural(old, step);
            }
            Step::Replace { slice, .. } if !slice.content.is_empty() => {
                self.full |= self.register_structural(old, step);
            }
            _ => {}
        }
        if let Some(range) = step_range {
            self.ranges.push(step_change(step, range));
        }
        if let Some(range) = self.window
            && let Some(mapped) = map_bounds(step, range)
        {
            self.window = Some(mapped);
        }
        if let Some(range) = self.structural_final {
            if let Some(mapped) = map_bounds(step, range) {
                self.structural_final = Some(mapped);
            }
        } else if let Some((range, _)) = self.structural
            && let Some(mapped) = map_bounds(step, range)
        {
            self.structural_final = Some(mapped);
        }
    }

    /// Finishes the change set against the resulting document.
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "the selection flag mirrors the transaction result"
    )]
    pub(crate) fn finish(mut self, new: &Doc, selection_changed: bool) -> ChangeSet {
        self.ranges
            .sort_by_key(|change| (change.old.start.0, change.new.start.0));
        let block_count = u32::try_from(new.blocks().len()).unwrap_or(u32::MAX);
        let blocks = if self.full {
            0..block_count
        } else {
            self.window.map_or(0..0, |range| block_span(new, range))
        };
        let (inserted_blocks, removed_blocks) = if self.full {
            (Box::default(), Box::default())
        } else {
            derive_blocks(new, blocks.start, self.structural, self.structural_final)
        };
        ChangeSet {
            blocks,
            ranges: self.ranges.into_boxed_slice(),
            selection: selection_changed,
            full: self.full,
            inserted_blocks,
            removed_blocks,
        }
    }

    /// Records the single structural step; a second one forces a full redraw.
    fn register_structural(&mut self, old: &Doc, step: &Step) -> bool {
        if self.structural.is_some() {
            return true;
        }
        let Some(range) = step_range(step) else {
            return true;
        };
        self.structural = Some((range, block_span(old, range)));
        false
    }
}

/// Inserted and removed top-level block indices.
type BlockIndices = (Box<[u32]>, Box<[u32]>);

/// Derives inserted/removed block indices from the structural span.
fn derive_blocks(
    new: &Doc,
    window_start: u32,
    structural: Option<(Range, BlockRange<u32>)>,
    structural_final: Option<Range>,
) -> BlockIndices {
    let (Some((_, old_span)), Some(final_range)) = (structural, structural_final) else {
        return (Box::default(), Box::default());
    };
    let new_span = block_span(new, final_range);
    let old_len = old_span.end.saturating_sub(old_span.start);
    let new_len = new_span.end.saturating_sub(new_span.start);
    let start = window_start;
    match new_len.cmp(&old_len) {
        core::cmp::Ordering::Greater => {
            let from = start.saturating_add(old_len).min(new_span.end);
            let to = start.saturating_add(new_len).min(new_span.end);
            (indices(from, to), Box::default())
        }
        core::cmp::Ordering::Less => {
            let from = start.saturating_add(new_len);
            let to = start.saturating_add(old_len);
            (Box::default(), indices(from, to))
        }
        core::cmp::Ordering::Equal => (Box::default(), Box::default()),
    }
}

/// Builds an index range as a boxed slice.
fn indices(from: u32, to: u32) -> Box<[u32]> {
    if to <= from {
        return Box::default();
    }
    let mut out = Vec::with_capacity(usize::try_from(to.saturating_sub(from)).unwrap_or(0));
    let mut index = from;
    while index < to {
        out.push(index);
        index = index.saturating_add(1);
    }
    out.into_boxed_slice()
}

/// Returns the top-level block span covered by a range in a document.
fn block_span(doc: &Doc, range: Range) -> BlockRange<u32> {
    let count = u32::try_from(doc.blocks().len()).unwrap_or(u32::MAX);
    if count == 0 {
        return 0..0;
    }
    let start = doc
        .block_at(range.start)
        .and_then(|(index, _)| u32::try_from(index).ok())
        .unwrap_or(0)
        .min(count.saturating_sub(1));
    let end_pos = if range.end > range.start {
        Pos(range.end.0.saturating_sub(1))
    } else {
        range.start
    };
    let end_index = doc
        .block_at(end_pos)
        .and_then(|(index, _)| u32::try_from(index).ok())
        .unwrap_or_else(|| count.saturating_sub(1))
        .min(count.saturating_sub(1));
    start..end_index.saturating_add(1).min(count)
}

/// Returns the position range a step touches, when it has one.
fn step_range(step: &Step) -> Option<Range> {
    match step {
        Step::Replace { from, to, .. }
        | Step::ReplaceAround { from, to, .. }
        | Step::AddMark { from, to, .. }
        | Step::RemoveMark { from, to, .. }
        | Step::SetInlineAttr { from, to, .. }
        | Step::SetMarkAttr { from, to, .. } => Some(Range::new(*from, *to)),
        Step::SetNodeAttr { pos, .. } => Some(Range::new(*pos, *pos)),
        Step::SetDocAttr { .. } | Step::ReplaceDoc { .. } => None,
    }
}

/// Builds the repaint range for one step.
fn step_change(step: &Step, range: Range) -> RangeChange {
    match step {
        Step::Replace { from, slice, .. } => RangeChange {
            old: range,
            new: Range::new(*from, Pos(from.0.saturating_add(slice.size()))),
        },
        Step::ReplaceAround {
            from,
            insert,
            slice,
            ..
        } => RangeChange {
            old: range,
            new: Range::new(
                *from,
                Pos(from.0.saturating_add(slice.size()).saturating_add(*insert)),
            ),
        },
        _ => RangeChange {
            old: range,
            new: range,
        },
    }
}

/// Maps a range so an insertion at its start stays inside the range.
fn map_bounds(step: &Step, range: Range) -> Option<Range> {
    let start = map_position(step, range.start, Assoc::Before)?;
    let end = map_position(step, range.end, Assoc::After)?;
    Some(Range::new(start, Pos(start.0.max(end.0))))
}

/// Maps one position through a step without allocating a step map.
fn map_position(step: &Step, pos: Pos, assoc: Assoc) -> Option<Pos> {
    let Some((start, old_len, new_len)) = step_shape(step) else {
        return Some(pos);
    };
    let current = pos.0;
    let old_end = start.checked_add(old_len)?;
    if current < start {
        return Some(pos);
    }
    if current > old_end {
        let mapped = if new_len < old_len {
            current.checked_sub(old_len.checked_sub(new_len)?)?
        } else {
            current.checked_add(new_len.checked_sub(old_len)?)?
        };
        return Some(Pos(mapped));
    }
    if current == start {
        let mapped = match assoc {
            Assoc::Before => start,
            Assoc::After => start.checked_add(new_len)?,
        };
        return Some(Pos(mapped));
    }
    if current == old_end {
        return Some(Pos(start.checked_add(new_len)?));
    }
    let offset = current.checked_sub(start)?;
    Some(Pos(start.checked_add(offset.min(new_len))?))
}

/// Returns the replacement bounds of a step, when it shifts positions.
fn step_shape(step: &Step) -> Option<(u32, u32, u32)> {
    match step {
        Step::Replace { from, to, slice } => {
            Some((from.0, to.0.saturating_sub(from.0), slice.size()))
        }
        Step::ReplaceAround {
            from, to, insert, ..
        } => Some((from.0, to.0.saturating_sub(from.0), *insert)),
        Step::ReplaceDoc { doc, old_size } => Some((0, *old_size, doc.size().0)),
        _ => None,
    }
}

/// Returns the union of two ranges.
fn union(current: Option<Range>, next: Range) -> Range {
    match current {
        None => next,
        Some(range) => Range::new(
            Pos(range.start.0.min(next.start.0)),
            Pos(range.end.0.max(next.end.0)),
        ),
    }
}
