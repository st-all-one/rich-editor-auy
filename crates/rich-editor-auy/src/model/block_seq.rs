#![allow(clippy::excessive_nesting, reason = "flattened chunk iterator")]

//! Borrowed top-level block views.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::slice;

use super::block::Block;
use super::chunks::{Chunk, Chunks};

/// Borrowed view over top-level blocks.
#[derive(Clone, Copy, Debug)]
pub struct BlockSeq<'a> {
    chunks: &'a Chunks,
}

impl<'a> BlockSeq<'a> {
    pub(crate) fn new(chunks: &'a Chunks) -> Self {
        Self { chunks }
    }

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

impl<'a> TopBlockIter<'a> {
    pub(crate) fn new(chunks: slice::Iter<'a, Arc<Chunk>>) -> Self {
        Self {
            chunks,
            current: [].iter(),
        }
    }
}

impl<'a> Iterator for TopBlockIter<'a> {
    type Item = &'a Arc<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(block) = self.current.next() {
                return Some(block);
            }
            let chunk = self.chunks.next()?;
            self.current = chunk.iter();
        }
    }
}
