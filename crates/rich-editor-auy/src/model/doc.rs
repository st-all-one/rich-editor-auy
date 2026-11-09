//! Immutable document snapshots.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::LimitError;
use crate::pos::{Pos, Range, Size};

use super::attrs::Attrs;
use super::block::Block;
use super::block_seq::BlockSeq;
use super::chunks::Chunks;
use super::limits::check_attr_limits;
use super::limits::check_block_limits;
use super::records::{Anchor, Reference};

/// A closed fragment of blocks in document order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fragment(Box<[Arc<Block>]>);

impl Fragment {
    /// Creates a fragment.
    #[must_use]
    pub fn new(blocks: Vec<Arc<Block>>) -> Self {
        Self(blocks.into_boxed_slice())
    }

    /// Returns blocks.
    #[must_use]
    pub fn blocks(&self) -> &[Arc<Block>] {
        &self.0
    }

    /// Returns whether the fragment is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the fragment size.
    #[must_use]
    pub fn size(&self) -> Size {
        let mut total = 0_u32;
        for block in &self.0 {
            total = total.saturating_add(block.size().0);
        }
        Size(total)
    }
}

#[derive(Debug)]
struct DocInner {
    attrs: Attrs,
    chunks: Chunks,
}

/// An immutable document snapshot.
#[derive(Clone, Debug)]
pub struct Doc(Arc<DocInner>);

impl PartialEq for Doc {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
            || (self.0.attrs == other.0.attrs && self.0.chunks == other.0.chunks)
    }
}

impl Eq for Doc {}

impl Doc {
    /// Creates a document and validates basic limits.
    ///
    /// # Errors
    ///
    /// Returns a limit error when the document exceeds configured counts.
    pub fn new(
        attrs: Attrs,
        blocks: Vec<Arc<Block>>,
        limits: &crate::Limits,
    ) -> Result<Self, LimitError> {
        let mut max_depth = 0_u32;
        for block in &blocks {
            max_depth = max_depth.max(super::measure::block_depth(block, 1));
        }
        if max_depth > u32::from(limits.max_depth) {
            return Err(LimitError::Depth {
                actual: max_depth,
                max: u32::from(limits.max_depth),
            });
        }
        let mut size = 0_u32;
        let mut count = 0_u32;
        for block in &blocks {
            check_block_limits(block, limits)?;
            size = size.saturating_add(block.size().0);
            count = count.saturating_add(1);
            count = count.saturating_add(super::measure::count_descendants(block));
        }
        check_attr_limits(&attrs, limits)?;
        if size > limits.max_doc_bytes {
            return Err(LimitError::DocBytes {
                actual: u64::from(size),
                max: limits.max_doc_bytes,
            });
        }
        if count > limits.max_nodes {
            return Err(LimitError::Nodes {
                actual: count,
                max: limits.max_nodes,
            });
        }
        Ok(Self(Arc::new(DocInner {
            attrs,
            chunks: Chunks::new(blocks),
        })))
    }

    /// Creates a document with default limits.
    ///
    /// # Errors
    ///
    /// Returns a limit error when the document exceeds defaults.
    pub fn from_blocks(attrs: Attrs, blocks: Vec<Arc<Block>>) -> Result<Self, LimitError> {
        Self::new(attrs, blocks, &crate::Limits::default())
    }

    /// Creates a snapshot from blocks that the caller has already measured.
    ///
    /// The decoder enforces node, depth, run, and byte budgets while it parses, so it does not
    /// need [`Doc::new`]'s second full traversal.
    pub(crate) fn from_decoded(attrs: Attrs, blocks: Vec<Arc<Block>>) -> Self {
        Self(Arc::new(DocInner {
            attrs,
            chunks: Chunks::new(blocks),
        }))
    }

    /// Creates a snapshot with one top-level block replaced.
    ///
    /// The caller guarantees that the replacement preserves node count; text edits and attribute
    /// updates do not add or remove structural nodes. Only the replacement block is rechecked
    /// against limits, so the hot path avoids rescanning an unchanged document.
    pub(crate) fn replacing_block(
        &self,
        index: usize,
        block: Block,
        limits: &crate::Limits,
    ) -> Result<Self, LimitError> {
        let chunks = self.0.chunks.replacing(index, block, limits)?;
        Ok(Self(Arc::new(DocInner {
            attrs: self.0.attrs.clone(),
            chunks,
        })))
    }

    /// Creates a snapshot with one block replaced, mutating in place when uniquely owned.
    pub(crate) fn replacing_block_owned(
        mut self,
        index: usize,
        block: Block,
        limits: &crate::Limits,
    ) -> Result<Self, LimitError> {
        if let Some(inner) = Arc::get_mut(&mut self.0) {
            inner.chunks.replacing_in_place(index, block, limits)?;
            return Ok(self);
        }
        self.replacing_block(index, block, limits)
    }

    /// Returns document attributes.
    #[must_use]
    pub fn attrs(&self) -> &Attrs {
        &self.0.attrs
    }

    /// Returns top-level blocks.
    #[must_use]
    pub fn blocks(&self) -> BlockSeq<'_> {
        self.0.chunks.view()
    }

    /// Returns a block by index.
    #[must_use]
    pub fn block(&self, index: u32) -> Option<&Arc<Block>> {
        self.0.chunks.get(usize::try_from(index).ok()?)
    }

    /// Returns the absolute range of a top-level block.
    #[must_use]
    pub fn block_range(&self, index: u32) -> Option<Range> {
        self.0.chunks.range(usize::try_from(index).ok()?)
    }

    /// Returns the block containing a position using the ordered start index.
    #[must_use]
    pub fn block_at(&self, pos: Pos) -> Option<(usize, &Arc<Block>)> {
        self.0.chunks.at(pos)
    }

    /// Returns the document size.
    #[must_use]
    pub fn size(&self) -> Size {
        self.0.chunks.size()
    }

    /// Returns whether two handles point at the same snapshot.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// Appends all text to `out` in reading order.
    pub fn text_into(&self, out: &mut String) {
        for block in self.0.chunks.iter() {
            super::measure::text_of_block(block, out);
        }
    }

    /// Returns the plain text of the document.
    #[must_use]
    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        self.text_into(&mut text);
        text
    }

    /// Collects node anchors in document order.
    #[must_use]
    pub fn anchors(&self) -> Vec<Anchor> {
        let mut anchors = Vec::new();
        let mut offset = 0_u32;
        for block in self.0.chunks.iter() {
            super::measure::collect_anchors(block, offset, &mut anchors);
            offset = offset.saturating_add(block.size().0);
        }
        anchors
    }

    /// Collects cross-publication references in document order.
    #[must_use]
    pub fn references(&self) -> Vec<Reference> {
        let mut references = Vec::new();
        let mut offset = 0_u32;
        for block in self.0.chunks.iter() {
            super::measure::collect_references(block, offset, &mut references);
            offset = offset.saturating_add(block.size().0);
        }
        references
    }
}
