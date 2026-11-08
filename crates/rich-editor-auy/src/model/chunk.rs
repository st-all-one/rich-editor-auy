#![allow(
    clippy::redundant_pub_crate,
    reason = "crate-internal chunk representation"
)]

//! One chunk of top-level blocks.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::slice;

use crate::model::Block;
use crate::pos::{Pos, Range};

/// One chunk of blocks and its relative start index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Chunk {
    blocks: Box<[Arc<Block>]>,
    starts: Box<[u32]>,
    size: u32,
}

impl Chunk {
    pub(crate) fn new(blocks: Vec<Arc<Block>>) -> Self {
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

    pub(crate) fn len(&self) -> usize {
        self.blocks.len()
    }

    pub(crate) fn iter(&self) -> slice::Iter<'_, Arc<Block>> {
        self.blocks.iter()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Arc<Block>> {
        self.blocks.get(index)
    }

    pub(crate) fn range(&self, index: usize) -> Option<Range> {
        let block = self.blocks.get(index)?;
        let start = *self.starts.get(index)?;
        let end = start.checked_add(block.size().0)?;
        Some(Range::new(Pos(start), Pos(end)))
    }

    pub(crate) fn at(&self, local: u32) -> Option<(usize, &Arc<Block>)> {
        let index = self
            .starts
            .partition_point(|start| *start <= local)
            .saturating_sub(1);
        let block = self.blocks.get(index)?;
        Some((index, block))
    }

    pub(crate) fn size(&self) -> u32 {
        self.size
    }

    pub(crate) fn replacing(&self, local: usize, block: Block) -> Option<Self> {
        let old = self.blocks.get(local)?;
        let old_size = old.size().0;
        let new_size = block.size().0;
        let mut blocks = self.blocks.to_vec();
        let slot = blocks.get_mut(local)?;
        *slot = Arc::new(block);
        let mut starts = self.starts.to_vec();
        for start in starts.iter_mut().skip(local.saturating_add(1)) {
            *start = start.saturating_add(new_size).saturating_sub(old_size);
        }
        Some(Self {
            blocks: blocks.into_boxed_slice(),
            starts: starts.into_boxed_slice(),
            size: self.size.saturating_sub(old_size).saturating_add(new_size),
        })
    }
}
