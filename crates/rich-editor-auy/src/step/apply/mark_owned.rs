//! Owned mark application.

use alloc::boxed::Box;

use crate::error::StepError;
use crate::model::{Block, Doc};
use crate::pos::Pos;
use crate::schema::Schema;

use super::super::text_edit;

#[allow(
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools,
    reason = "mark operation shares one implementation"
)]
pub(super) fn mark_range_owned(
    mut result: Doc,
    schema: &Schema,
    from: Pos,
    to: Pos,
    mark: &crate::model::Mark,
    add: bool,
) -> Result<Doc, StepError> {
    if from >= to {
        return Ok(result);
    }
    let mut offset = 0_u32;
    let mut index = 0_usize;
    while index < result.blocks().len() {
        let Some(block) = result
            .block(u32::try_from(index).unwrap_or(u32::MAX))
            .cloned()
        else {
            break;
        };
        let block_start = offset;
        let block_end = offset.saturating_add(block.size().0);
        offset = block_end;
        if block_end <= from.0 || block_start >= to.0 {
            index = index.saturating_add(1);
            continue;
        }
        let Some(text) = block.as_textblock() else {
            return Err(StepError::Unsupported {
                operation: Box::from("mark_outside_textblock"),
            });
        };
        if !schema.mark_allowed(block.kind(), mark.id) {
            return Err(StepError::Unknown {
                path: Box::from("mark"),
            });
        }
        let start = from.0.max(block_start.saturating_add(1));
        let end = to.0.min(block_end.saturating_sub(1));
        if start > end {
            index = index.saturating_add(1);
            continue;
        }
        let local_start = usize::try_from(start.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let local_end = usize::try_from(end.saturating_sub(block_start.saturating_add(1)))
            .unwrap_or(usize::MAX);
        let new_text = if add {
            text_edit::add_mark(text, local_start, local_end, mark)?
        } else {
            text_edit::remove_mark(text, local_start, local_end, mark)?
        };
        let new_block = Block::text(block.kind(), block.attrs().clone(), new_text);
        offset = block_start.saturating_add(new_block.size().0);
        result = result.replacing_block_owned(index, new_block, &crate::Limits::default())?;
        index = index.saturating_add(1);
    }
    Ok(result)
}
