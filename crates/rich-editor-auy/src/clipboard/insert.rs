//! Inserting and replacing with a slice.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::Limits;
use crate::error::StepError;
use crate::model::{Block, Doc};
use crate::pos::{Pos, Range};
use crate::schema::Schema;
use crate::step::{Slice, Step, apply_step};

/// Returns the top-level boundary index for a position.
pub(super) fn boundary(doc: &Doc, pos: Pos) -> Option<usize> {
    if pos == Pos(doc.size().0) {
        return Some(doc.blocks().len());
    }
    crate::step::block_index_at(doc, pos, true)
}

/// Collects every top-level block of a document.
fn collect_blocks(doc: &Doc) -> Vec<Arc<Block>> {
    doc.blocks().iter().cloned().collect()
}

impl Doc {
    /// Inserts a slice at a position.
    ///
    /// An inline slice is spliced into the text block at the position. A block slice is inserted
    /// at a block boundary; a position inside a text block splits it first, so the caret keeps its
    /// logical place.
    ///
    /// # Errors
    ///
    /// Returns an out-of-bounds error for a position outside the document, an unsupported error
    /// when an inline slice lands on a structural token, and a limit error when the result would
    /// exceed a document limit.
    pub fn insert_slice(&self, schema: &Schema, at: Pos, slice: &Slice) -> Result<Self, StepError> {
        if at.0 > self.size().0 {
            return Err(StepError::OutOfBounds {
                position: at.0,
                size: self.size().0,
            });
        }
        if slice.is_empty() {
            return Ok(self.clone());
        }
        if slice.content.is_empty() {
            return apply_step(
                self,
                schema,
                &Step::Replace {
                    from: at,
                    to: at,
                    slice: slice.clone(),
                },
            );
        }
        if slice.open_start > 0 || slice.open_end > 0 {
            return super::open::insert_open(self, slice, at);
        }
        if let Some(index) = self.boundary(at) {
            return self.spliced(index, slice);
        }
        let split = crate::step::split_at(self, at)?;
        // After the split the position sits at the end of the left half, so the new blocks go
        // right after it.
        let index = split.block_at(at).map_or_else(
            || split.blocks().len(),
            |(index, _)| index.saturating_add(1),
        );
        split.spliced(index, slice)
    }

    /// Replaces a range with a slice.
    ///
    /// An inline slice replaces runs inside one text block; a block slice replaces whole blocks and
    /// requires both ends to be block boundaries.
    ///
    /// # Errors
    ///
    /// Returns an out-of-bounds error for an inverted or out-of-document range, an unsupported
    /// error when a block slice is not block-aligned, and a limit error when the result would
    /// exceed a document limit.
    pub fn replace_slice(
        &self,
        schema: &Schema,
        range: Range,
        slice: &Slice,
    ) -> Result<Self, StepError> {
        let doc_size = self.size().0;
        if range.start > range.end || range.end.0 > doc_size {
            return Err(super::delete::range_error(range, doc_size));
        }
        if slice.content.is_empty() {
            return apply_step(
                self,
                schema,
                &Step::Replace {
                    from: range.start,
                    to: range.end,
                    slice: slice.clone(),
                },
            );
        }
        if slice.open_start > 0 || slice.open_end > 0 {
            return super::open::replace_open(self, slice, range);
        }
        let from = self
            .boundary(range.start)
            .ok_or_else(|| StepError::Unsupported {
                operation: Box::from("slice_replace_start_not_aligned"),
            })?;
        let to = self
            .boundary(range.end)
            .ok_or_else(|| StepError::Unsupported {
                operation: Box::from("slice_replace_end_not_aligned"),
            })?;
        let mut blocks: Vec<Arc<Block>> = Vec::new();
        blocks.extend(self.blocks().iter().take(from).cloned());
        blocks.extend_from_slice(slice.content.blocks());
        blocks.extend(self.blocks().iter().skip(to).cloned());
        Self::new(self.attrs().clone(), blocks, &Limits::default()).map_err(StepError::Limit)
    }

    /// Inserts the blocks of a slice at a top-level index.
    fn spliced(&self, index: usize, slice: &Slice) -> Result<Self, StepError> {
        let mut merged = collect_blocks(self);
        merged.splice(index..index, slice.content.blocks().iter().cloned());
        Self::new(self.attrs().clone(), merged, &Limits::default()).map_err(StepError::Limit)
    }

    /// Returns the top-level boundary index for a position.
    fn boundary(&self, pos: Pos) -> Option<usize> {
        boundary(self, pos)
    }
}
