//! Structural block replacement helpers.

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Block, Doc};
use crate::pos::Pos;

pub(super) fn replace_blocks(
    doc: &Doc,
    from: Pos,
    to: Pos,
    replacement: &[Arc<Block>],
) -> Result<Doc, StepError> {
    if from >= to {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("structural_insert"),
        });
    }
    let start = block_index_at(doc, from, true).ok_or_else(|| StepError::OutOfBounds {
        position: from.0,
        size: doc.size().0,
    })?;
    let end = block_index_at(doc, to, false).ok_or_else(|| StepError::OutOfBounds {
        position: to.0,
        size: doc.size().0,
    })?;
    if end < start {
        return Err(StepError::OutOfBounds {
            position: to.0,
            size: doc.size().0,
        });
    }
    let mut blocks = Vec::new();
    blocks.extend(doc.blocks().iter().take(start).cloned());
    blocks.extend_from_slice(replacement);
    blocks.extend(doc.blocks().iter().skip(end).cloned());
    Doc::new(doc.attrs().clone(), blocks, &crate::Limits::default()).map_err(StepError::Limit)
}

#[allow(
    clippy::fn_params_excessive_bools,
    reason = "one boundary selector covers both block edges"
)]
pub(super) fn block_index_at(doc: &Doc, pos: Pos, start: bool) -> Option<usize> {
    if !start && pos == Pos(doc.size().0) {
        return Some(doc.blocks().len());
    }
    let mut offset = 0_u32;
    for (index, block) in doc.blocks().iter().enumerate() {
        let end = offset.checked_add(block.size().0)?;
        if (start && pos == Pos(offset)) || (!start && pos == Pos(end)) {
            return Some(index);
        }
        offset = end;
    }
    None
}
