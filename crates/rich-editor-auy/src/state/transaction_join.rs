//! Block joining.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit boundary checks before one structural step"
)]

use alloc::sync::Arc;

use crate::error::StepError;
use crate::model::{Block, Doc, Fragment, MAX_RUN_BYTES, TextBlock};
use crate::pos::Pos;
use crate::step::{Slice, Step};

use super::selection::Selection;
use super::transaction::Transaction;

impl Transaction {
    /// Joins the block containing a position with its previous sibling.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is not at a block boundary, the transaction already
    /// contains steps, or either side is not a text block.
    pub fn join_backward(&mut self, at: Pos) -> Result<(), StepError> {
        if at.0 == 0 {
            return Err(StepError::Unsupported {
                operation: alloc::boxed::Box::from("join_empty"),
            });
        }
        if !self.steps.is_empty() {
            return Err(StepError::Unsupported {
                operation: alloc::boxed::Box::from("join_after_steps"),
            });
        }
        let Some(doc) = self.base.clone() else {
            return Err(StepError::Unsupported {
                operation: alloc::boxed::Box::from("join_without_doc"),
            });
        };
        let (from, to, block) = join_parts(&doc, at)?;
        self.steps.push(Step::Replace {
            from,
            to,
            slice: Slice::from_fragment(Fragment::new(alloc::vec![Arc::new(block)])),
        });
        self.selection = Some(Selection::caret(Pos(at.0.saturating_sub(1))));
        Ok(())
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "explicit boundary and payload checks"
)]
fn join_parts(doc: &Doc, at: Pos) -> Result<(Pos, Pos, Block), StepError> {
    let mut current: Option<usize> = None;
    for index in 0..doc.blocks().len() {
        let Some(range) = doc.block_range(u32::try_from(index).unwrap_or(u32::MAX)) else {
            continue;
        };
        if range.start == at {
            current = Some(index);
            break;
        }
    }
    let Some(current) = current.filter(|index| *index > 0) else {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("join_boundary"),
        });
    };
    let Some(previous) = doc.block(u32::try_from(current.saturating_sub(1)).unwrap_or(u32::MAX))
    else {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("join_previous"),
        });
    };
    let Some(block) = doc.block(u32::try_from(current).unwrap_or(u32::MAX)) else {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("join_current"),
        });
    };
    if previous.kind() != block.kind() {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("join_different_kinds"),
        });
    }
    let (Some(left), Some(right)) = (previous.as_textblock(), block.as_textblock()) else {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("join_non_text"),
        });
    };
    let mut merged = TextBlock::empty();
    for item in left.items() {
        merged.push(item.clone());
    }
    for item in right.items() {
        merged.push(item.clone());
    }
    let Some(previous_range) =
        doc.block_range(u32::try_from(current.saturating_sub(1)).unwrap_or(u32::MAX))
    else {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("join_range"),
        });
    };
    let Some(current_range) = doc.block_range(u32::try_from(current).unwrap_or(u32::MAX)) else {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("join_range"),
        });
    };
    merged
        .normalize(u32::try_from(MAX_RUN_BYTES).unwrap_or(u32::MAX))
        .map_err(StepError::Limit)?;
    Ok((
        previous_range.start,
        current_range.end,
        Block::text(previous.kind(), previous.attrs().clone(), merged),
    ))
}
