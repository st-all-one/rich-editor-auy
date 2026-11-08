#![allow(
    clippy::excessive_nesting,
    clippy::redundant_pub_crate,
    reason = "explicit chunk directory traversal in a crate-internal module"
)]

//! Chunked top-level block storage.
//!
//! A flat `Box<[Arc<Block>]>` makes every single-block edit copy all block pointers and the whole
//! start index. Chunks keep that copy proportional to one chunk plus the chunk directory, which is
//! the difference between O(blocks) and O(chunk + blocks/chunk) for the hot path.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::slice;

use crate::error::LimitError;
use crate::pos::{Pos, Range, Size};

use super::block::Block;
use super::limits::check_block_limits;

/// Number of top-level blocks in one chunk.
pub(crate) const CHUNK_BLOCKS: usize = 128;

/// One chunk of blocks and its relative start index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Chunk {
    blocks: Box<[Arc<Block>]>,
    starts: Box<[u32]>,
    size: u32,
}

impl Chunk {
    fn new(blocks: Vec<Arc<Block>>) -> Self {
        let mut starts = Vec::with_capacity(blocks.len());
        let mut offset = 0_u32;
        for block in &blocks {
            starts.push(offset);
            offset = offset.saturating_add(block.size().0);
        }
        Self {
            blocks: blocks.into_boxed_slice(),
            starts: starts.into_boxed_slice(),
            size: offset,
        }
    }

    fn len(&self) -> usize {
        self.blocks.len()
    }

    fn get(&self, index: usize) -> Option<&Arc<Block>> {
        self.blocks.get(index)
    }

    fn range(&self, index: usize) -> Option<Range> {
        let block = self.blocks.get(index)?;
        let start = *self.starts.get(index)?;
        let end = start.checked_add(block.size().0)?;
        Some(Range::new(Pos(start), Pos(end)))
    }

    fn at(&self, local: u32) -> Option<(usize, &Arc<Block>)> {
        let index = self
            .starts
            .partition_point(|start| *start <= local)
            .saturating_sub(1);
        let block = self.blocks.get(index)?;
        Some((index, block))
    }
}

/// Chunk directory for a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Chunks {
    directory: Box<[Arc<Chunk>]>,
    starts: Box<[u32]>,
    count: usize,
    size: Size,
}

impl Chunks {
    pub(crate) fn new(blocks: Vec<Arc<Block>>) -> Self {
        let mut chunks = Vec::new();
        let mut starts = Vec::new();
        let mut count = 0_usize;
        let mut size = 0_u32;
        let mut iter = blocks.into_iter();
        loop {
            let part: Vec<Arc<Block>> = iter.by_ref().take(CHUNK_BLOCKS).collect();
            if part.is_empty() {
                break;
            }
            let chunk = Chunk::new(part);
            starts.push(size);
            size = size.saturating_add(chunk.size);
            count = count.saturating_add(chunk.len());
            chunks.push(Arc::new(chunk));
        }
        Self {
            directory: chunks.into_boxed_slice(),
            starts: starts.into_boxed_slice(),
            count,
            size: Size(size),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.count
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Arc<Block>> {
        let chunk = self.directory.get(index / CHUNK_BLOCKS)?;
        chunk.get(index % CHUNK_BLOCKS)
    }

    pub(crate) fn range(&self, index: usize) -> Option<Range> {
        let chunk_index = index / CHUNK_BLOCKS;
        let local = index % CHUNK_BLOCKS;
        let chunk = self.directory.get(chunk_index)?;
        let start = *self.starts.get(chunk_index)?;
        chunk.range(local).map(|range| {
            Range::new(
                Pos(start.saturating_add(range.start.0)),
                Pos(start.saturating_add(range.end.0)),
            )
        })
    }

    pub(crate) fn at(&self, pos: Pos) -> Option<(usize, &Arc<Block>)> {
        if pos > self.size.as_pos() || self.directory.is_empty() {
            return None;
        }
        let chunk_index = self
            .starts
            .partition_point(|start| *start <= pos.0)
            .saturating_sub(1);
        let chunk = self.directory.get(chunk_index)?;
        let chunk_start = *self.starts.get(chunk_index)?;
        let local = pos.0.saturating_sub(chunk_start);
        let (local_index, block) = chunk.at(local)?;
        Some((
            chunk_index
                .saturating_mul(CHUNK_BLOCKS)
                .saturating_add(local_index),
            block,
        ))
    }

    pub(crate) fn iter(&self) -> TopBlockIter<'_> {
        TopBlockIter {
            chunks: self.directory.iter(),
            current: [].iter(),
        }
    }

    pub(crate) fn to_vec(&self) -> Vec<Arc<Block>> {
        self.iter().cloned().collect()
    }

    pub(crate) fn replacing(
        &self,
        index: usize,
        block: Block,
        limits: &crate::Limits,
    ) -> Result<Self, LimitError> {
        check_block_limits(&block, limits)?;
        let chunk_index = index / CHUNK_BLOCKS;
        let local = index % CHUNK_BLOCKS;
        let old_chunk = self
            .directory
            .get(chunk_index)
            .ok_or_else(|| LimitError::Nodes {
                actual: u32::try_from(index).unwrap_or(u32::MAX),
                max: u32::try_from(self.count).unwrap_or(u32::MAX),
            })?;
        let old_size = old_chunk.size;
        let mut blocks = old_chunk.blocks.to_vec();
        let Some(slot) = blocks.get_mut(local) else {
            return Err(LimitError::Nodes {
                actual: u32::try_from(index).unwrap_or(u32::MAX),
                max: u32::try_from(self.count).unwrap_or(u32::MAX),
            });
        };
        *slot = Arc::new(block);
        let new_chunk = Arc::new(Chunk::new(blocks));
        let new_size = new_chunk.size;
        let mut chunks = self.directory.to_vec();
        let Some(slot) = chunks.get_mut(chunk_index) else {
            return Err(LimitError::Nodes {
                actual: u32::try_from(index).unwrap_or(u32::MAX),
                max: u32::try_from(self.count).unwrap_or(u32::MAX),
            });
        };
        *slot = new_chunk;
        let mut starts = self.starts.to_vec();
        for start in starts.iter_mut().skip(chunk_index.saturating_add(1)) {
            *start = start.saturating_add(new_size).saturating_sub(old_size);
        }
        let total = self
            .size
            .0
            .saturating_sub(old_size)
            .saturating_add(new_size);
        if total > limits.max_doc_bytes {
            return Err(LimitError::DocBytes {
                actual: u64::from(total),
                max: limits.max_doc_bytes,
            });
        }
        Ok(Self {
            directory: chunks.into_boxed_slice(),
            starts: starts.into_boxed_slice(),
            count: self.count,
            size: Size(total),
        })
    }
}

/// Borrowed view over top-level blocks.
#[derive(Clone, Copy, Debug)]
pub struct BlockSeq<'a> {
    chunks: &'a Chunks,
}

impl<'a> BlockSeq<'a> {
    /// Returns the number of top-level blocks.
    #[must_use]
    pub fn len(self) -> usize {
        self.chunks.len()
    }

    /// Returns whether the document has no top-level blocks.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.chunks.is_empty()
    }

    /// Returns a block by index.
    #[must_use]
    pub fn get(self, index: usize) -> Option<&'a Arc<Block>> {
        self.chunks.get(index)
    }

    /// Iterates blocks in document order.
    pub fn iter(self) -> TopBlockIter<'a> {
        self.chunks.iter()
    }

    /// Copies block handles into a vector.
    #[must_use]
    pub fn to_vec(self) -> Vec<Arc<Block>> {
        self.chunks.to_vec()
    }
}

/// Iterator over the flattened chunk directory.
#[derive(Clone, Debug)]
pub struct TopBlockIter<'a> {
    chunks: slice::Iter<'a, Arc<Chunk>>,
    current: slice::Iter<'a, Arc<Block>>,
}

impl<'a> Iterator for TopBlockIter<'a> {
    type Item = &'a Arc<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(block) = self.current.next() {
                return Some(block);
            }
            let chunk = self.chunks.next()?;
            self.current = chunk.blocks.iter();
        }
    }
}

impl Chunks {
    pub(crate) fn view(&self) -> BlockSeq<'_> {
        BlockSeq { chunks: self }
    }
}
