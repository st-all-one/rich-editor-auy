//! Inline replacement that preserves runs and marks.

use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::TextBlock;

use super::text_edit::split_text;

/// Replaces a content range with inline runs, preserving their marks.
///
/// # Errors
///
/// Returns an error when the range is not a valid UTF-8 boundary.
pub(super) fn replace_textblock(
    block: &TextBlock,
    start: usize,
    end: usize,
    replacement: &TextBlock,
) -> Result<TextBlock, StepError> {
    let total = usize::try_from(block.len().0).unwrap_or(usize::MAX);
    if start > end || end > total {
        return Err(StepError::Unsupported {
            operation: alloc::boxed::Box::from("text_range"),
        });
    }
    let (before, rest) = split_text(block, start)?;
    let (_, after) = split_text(&rest, end.saturating_sub(start))?;
    let mut items = Vec::new();
    items.extend_from_slice(before.items());
    items.extend_from_slice(replacement.items());
    items.extend_from_slice(after.items());
    let mut result = TextBlock::new(items);
    result
        .normalize(u32::try_from(crate::model::MAX_RUN_BYTES).unwrap_or(4096))
        .map_err(StepError::Limit)?;
    Ok(result)
}
