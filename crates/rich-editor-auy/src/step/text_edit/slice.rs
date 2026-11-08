//! Text range extraction.

use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{InlineItem, TextBlock};

use super::{push_run, range_error};

/// Extracts one content range from a text block without splitting twice.
///
/// # Errors
///
/// Returns an error when the range is not a UTF-8 boundary.
pub(crate) fn slice_text(
    block: &TextBlock,
    start: usize,
    end: usize,
) -> Result<TextBlock, StepError> {
    let total = usize::try_from(block.len().0).unwrap_or(usize::MAX);
    if start > end || end > total {
        return Err(range_error());
    }
    let mut items = Vec::new();
    let mut cursor = 0_usize;
    for item in block.items() {
        match item {
            InlineItem::Text(run) => {
                let run_start = cursor;
                let run_end = cursor.saturating_add(run.text.len());
                let from = start.max(run_start);
                let to = end.min(run_end);
                if from < to {
                    let offset_start = from.saturating_sub(run_start);
                    let offset_end = to.saturating_sub(run_start);
                    push_run(
                        &mut items,
                        run.text
                            .get(offset_start..offset_end)
                            .ok_or_else(range_error)?,
                        run.marks.clone(),
                    );
                }
                cursor = run_end;
            }
            InlineItem::Atom(atom) => {
                if cursor >= start && cursor < end {
                    items.push(InlineItem::Atom(atom.clone()));
                }
                cursor = cursor.saturating_add(1);
            }
        }
    }
    let mut result = TextBlock::new(items);
    result
        .normalize(u32::try_from(crate::model::MAX_RUN_BYTES).unwrap_or(4096))
        .map_err(StepError::Limit)?;
    Ok(result)
}
