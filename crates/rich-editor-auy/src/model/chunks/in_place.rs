//! In-place chunk replacement.

use alloc::sync::Arc;

use crate::error::LimitError;
use crate::model::Block;
use crate::model::limits::check_block_limits;
use crate::pos::Size;

use super::{CHUNK_BLOCKS, Chunks};

impl Chunks {
    pub(crate) fn replacing_in_place(
        &mut self,
        index: usize,
        block: Block,
        limits: &crate::Limits,
    ) -> Result<(), LimitError> {
        check_block_limits(&block, limits)?;
        if let Some(flat) = &self.flat {
            let old_size = flat.get(index).map_or(0, |block| block.size().0);
            let new_size = block.size().0;
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
            if let Some(flat) = self.flat.as_mut()
                && !flat.replace_in_place(index, block)
            {
                return Err(LimitError::Nodes {
                    actual: u32::try_from(index).unwrap_or(u32::MAX),
                    max: u32::try_from(self.count).unwrap_or(u32::MAX),
                });
            }
            self.size = Size(total);
            return Ok(());
        }
        self.replacing_chunk_in_place(index, block, limits)
    }

    fn replacing_chunk_in_place(
        &mut self,
        index: usize,
        block: Block,
        limits: &crate::Limits,
    ) -> Result<(), LimitError> {
        let chunk_index = index / CHUNK_BLOCKS;
        let local = index % CHUNK_BLOCKS;
        let old_chunk = self
            .directory
            .get(chunk_index)
            .ok_or_else(|| LimitError::Nodes {
                actual: u32::try_from(index).unwrap_or(u32::MAX),
                max: u32::try_from(self.count).unwrap_or(u32::MAX),
            })?;
        let old_block_size = old_chunk.get(local).map_or(0, |block| block.size().0);
        let new_size = block.size().0;
        let total = self
            .size
            .0
            .saturating_sub(old_block_size)
            .saturating_add(new_size);
        if total > limits.max_doc_bytes {
            return Err(LimitError::DocBytes {
                actual: u64::from(total),
                max: limits.max_doc_bytes,
            });
        }
        let chunk = self
            .directory
            .get_mut(chunk_index)
            .ok_or_else(|| LimitError::Nodes {
                actual: u32::try_from(index).unwrap_or(u32::MAX),
                max: u32::try_from(self.count).unwrap_or(u32::MAX),
            })?;
        if let Some(chunk) = Arc::get_mut(chunk) {
            if !chunk.replace_in_place(local, block) {
                return Err(LimitError::Nodes {
                    actual: u32::try_from(index).unwrap_or(u32::MAX),
                    max: u32::try_from(self.count).unwrap_or(u32::MAX),
                });
            }
        } else {
            let new_chunk = chunk
                .replacing(local, block)
                .ok_or_else(|| LimitError::Nodes {
                    actual: u32::try_from(index).unwrap_or(u32::MAX),
                    max: u32::try_from(self.count).unwrap_or(u32::MAX),
                })?;
            *chunk = Arc::new(new_chunk);
        }
        for start in self.starts.iter_mut().skip(chunk_index.saturating_add(1)) {
            *start = start
                .saturating_sub(old_block_size)
                .saturating_add(new_size);
        }
        self.size = Size(total);
        Ok(())
    }
}
