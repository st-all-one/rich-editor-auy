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

use crate::error::LimitError;
use crate::pos::{Pos, Range, Size};

use super::block::Block;
use super::block_seq::{BlockSeq, TopBlockIter};
use super::chunk::Chunk;
use super::limits::check_block_limits;

mod in_place;

/// Number of top-level blocks in one chunk.
pub(crate) const CHUNK_BLOCKS: usize = 128;

/// Chunk directory for a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Chunks {
    directory: Box<[Arc<Chunk>]>,
    starts: Box<[u32]>,
    count: usize,
    size: Size,
    flat: Option<Chunk>,
}

impl Chunks {
    pub(crate) fn new(blocks: Vec<Arc<Block>>) -> Self {
        if blocks.len() <= CHUNK_BLOCKS {
            let flat = Chunk::new(blocks);
            let count = flat.len();
            let size = flat.size();
            return Self {
                directory: Box::new([]),
                starts: Box::new([]),
                count,
                size: Size(size),
                flat: Some(flat),
            };
        }
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
            size = size.saturating_add(chunk.size());
            count = count.saturating_add(chunk.len());
            chunks.push(Arc::new(chunk));
        }
        Self {
            directory: chunks.into_boxed_slice(),
            starts: starts.into_boxed_slice(),
            count,
            size: Size(size),
            flat: None,
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
        if let Some(flat) = &self.flat {
            return flat.get(index);
        }
        let chunk = self.directory.get(index / CHUNK_BLOCKS)?;
        chunk.get(index % CHUNK_BLOCKS)
    }

    pub(crate) fn range(&self, index: usize) -> Option<Range> {
        if let Some(flat) = &self.flat {
            return flat.range(index);
        }
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
        if pos > self.size.as_pos() || self.count == 0 {
            return None;
        }
        if let Some(flat) = &self.flat {
            return flat.at(pos.0);
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
        if let Some(flat) = &self.flat {
            return TopBlockIter::flat(flat.iter());
        }
        TopBlockIter::new(self.directory.iter())
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
        if let Some(flat) = &self.flat {
            return self.replacing_flat(flat, index, block, limits);
        }
        self.replacing_chunked(index, block, limits)
    }

    fn replacing_flat(
        &self,
        flat: &Chunk,
        index: usize,
        block: Block,
        limits: &crate::Limits,
    ) -> Result<Self, LimitError> {
        let new_flat = flat
            .replacing(index, block)
            .ok_or_else(|| LimitError::Nodes {
                actual: u32::try_from(index).unwrap_or(u32::MAX),
                max: u32::try_from(self.count).unwrap_or(u32::MAX),
            })?;
        let total = new_flat.size();
        if total > limits.max_doc_bytes {
            return Err(LimitError::DocBytes {
                actual: u64::from(total),
                max: limits.max_doc_bytes,
            });
        }
        Ok(Self {
            directory: Box::new([]),
            starts: Box::new([]),
            count: self.count,
            size: Size(total),
            flat: Some(new_flat),
        })
    }

    fn replacing_chunked(
        &self,
        index: usize,
        block: Block,
        limits: &crate::Limits,
    ) -> Result<Self, LimitError> {
        let chunk_index = index / CHUNK_BLOCKS;
        let local = index % CHUNK_BLOCKS;
        let old_chunk = self
            .directory
            .get(chunk_index)
            .ok_or_else(|| LimitError::Nodes {
                actual: u32::try_from(index).unwrap_or(u32::MAX),
                max: u32::try_from(self.count).unwrap_or(u32::MAX),
            })?;
        let old_size = old_chunk.size();
        let new_chunk =
            Arc::new(
                old_chunk
                    .replacing(local, block)
                    .ok_or_else(|| LimitError::Nodes {
                        actual: u32::try_from(index).unwrap_or(u32::MAX),
                        max: u32::try_from(self.count).unwrap_or(u32::MAX),
                    })?,
            );
        let new_size = new_chunk.size();
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
            flat: None,
        })
    }
}

impl Chunks {
    pub(crate) fn view(&self) -> BlockSeq<'_> {
        BlockSeq::new(self)
    }
}
