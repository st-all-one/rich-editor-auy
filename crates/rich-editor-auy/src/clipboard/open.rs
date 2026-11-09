//! Open-edge merging for structural slices.
//!
//! A slice copied from a partial range keeps its edge blocks open. Inserting or replacing with such
//! a slice merges the host prefix/suffix into the open edges instead of splitting the host block.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::Limits;
use crate::error::StepError;
use crate::model::{Block, Doc, TextBlock};
use crate::pos::{Pos, Range};
use crate::step::{Slice, slice_text};

/// Inserts a slice whose edge blocks are open, merging them into the host text block.
///
/// # Errors
///
/// Returns an unsupported error when the position is not inside a text block or when an open
/// edge is not text, and a limit error when the result exceeds a document limit.
pub(super) fn insert_open(doc: &Doc, slice: &Slice, at: Pos) -> Result<Doc, StepError> {
    check_open_depth(slice)?;
    let Some((index, host, local)) = strict_text_at(doc, at)? else {
        return Err(open_boundary());
    };
    let text = host.as_textblock().ok_or_else(open_structural)?;
    let length = usize::try_from(text.len().0).unwrap_or(usize::MAX);
    let prefix = slice_text(text, 0, local)?;
    let suffix = slice_text(text, local, length)?;
    let mut blocks = slice.content.blocks().to_vec();
    if slice.open_start > 0 {
        merge_into(&mut blocks, host, &prefix, EdgeSide::Start)?;
    }
    if slice.open_end > 0 {
        merge_into(&mut blocks, host, &suffix, EdgeSide::End)?;
    }
    let mut merged = doc.blocks().to_vec();
    merged.splice(index..index.saturating_add(1), blocks);
    Doc::new(doc.attrs().clone(), merged, &Limits::default()).map_err(StepError::Limit)
}

/// Replaces a range with a slice whose edge blocks are open.
///
/// # Errors
///
/// Returns an unsupported error when an open edge does not sit inside a text block, when the
/// merged edges disagree on the block kind, or when a closed edge is not block-aligned.
pub(super) fn replace_open(doc: &Doc, slice: &Slice, range: Range) -> Result<Doc, StepError> {
    check_open_depth(slice)?;
    if range.start >= range.end {
        return Err(open_boundary());
    }
    let start_edge = open_edge(doc, slice, range.start, EdgeSide::Start)?;
    let end_edge = open_edge(doc, slice, range.end, EdgeSide::End)?;
    let mut blocks = slice.content.blocks().to_vec();
    if let Some((_, host, local)) = start_edge {
        let text = host.as_textblock().ok_or_else(open_structural)?;
        let prefix = slice_text(text, 0, local)?;
        merge_into(&mut blocks, host, &prefix, EdgeSide::Start)?;
    }
    if let Some((_, host, local)) = end_edge {
        let text = host.as_textblock().ok_or_else(open_structural)?;
        let length = usize::try_from(text.len().0).unwrap_or(usize::MAX);
        let suffix = slice_text(text, local, length)?;
        merge_into(&mut blocks, host, &suffix, EdgeSide::End)?;
    }
    let start_index = start_edge
        .map(|(index, _, _)| index)
        .or_else(|| crate::clipboard::insert::boundary(doc, range.start))
        .unwrap_or(0);
    let end_index = end_edge
        .map(|(index, _, _)| index)
        .or_else(|| crate::clipboard::insert::boundary(doc, range.end))
        .unwrap_or_else(|| doc.blocks().len());
    let mut merged = Vec::with_capacity(doc.blocks().len().saturating_add(blocks.len()));
    merged.extend(doc.blocks().iter().take(start_index).cloned());
    merged.extend(blocks);
    let skip = end_index.saturating_add(usize::from(end_edge.is_some()));
    merged.extend(doc.blocks().iter().skip(skip).cloned());
    Doc::new(doc.attrs().clone(), merged, &Limits::default()).map_err(StepError::Limit)
}

/// Resolves one edge of an open structural replacement.
fn open_edge<'a>(
    doc: &'a Doc,
    slice: &Slice,
    pos: Pos,
    side: EdgeSide,
) -> Result<Option<TextEdge<'a>>, StepError> {
    let open = match side {
        EdgeSide::Start => slice.open_start,
        EdgeSide::End => slice.open_end,
    };
    let boundary = crate::clipboard::insert::boundary(doc, pos);
    if open > 0 {
        return match strict_text_at(doc, pos)? {
            Some(edge) if boundary.is_none() => Ok(Some(edge)),
            _ => Err(open_boundary()),
        };
    }
    if boundary.is_none() {
        let operation = match side {
            EdgeSide::Start => "slice_replace_start_not_aligned",
            EdgeSide::End => "slice_replace_end_not_aligned",
        };
        return Err(StepError::Unsupported {
            operation: Box::from(operation),
        });
    }
    Ok(None)
}

/// Returns the text block that strictly contains a position, with its content offset.
fn strict_text_at(doc: &Doc, pos: Pos) -> Result<Option<TextEdge<'_>>, StepError> {
    if pos == Pos(doc.size().0) || crate::clipboard::insert::boundary(doc, pos).is_some() {
        return Ok(None);
    }
    let Some((index, block)) = doc.block_at(pos) else {
        return Ok(None);
    };
    let Some(text) = block.as_textblock() else {
        return Err(StepError::Unsupported {
            operation: Box::from("slice_structural"),
        });
    };
    let start = doc
        .block_range(u32::try_from(index).unwrap_or(u32::MAX))
        .map_or(0, |range| range.start.0);
    let local =
        usize::try_from(pos.0.saturating_sub(start).saturating_sub(1)).unwrap_or(usize::MAX);
    let length = usize::try_from(text.len().0).unwrap_or(usize::MAX);
    if local > length {
        return Ok(None);
    }
    Ok(Some((index, block, local)))
}

/// Rejects open depths the model cannot represent.
fn check_open_depth(slice: &Slice) -> Result<(), StepError> {
    if slice.open_start > 1 || slice.open_end > 1 {
        return Err(StepError::Unsupported {
            operation: Box::from("open_slice_depth"),
        });
    }
    Ok(())
}

/// Reports an open edge that cannot be merged.
fn open_boundary() -> StepError {
    StepError::Unsupported {
        operation: Box::from("open_slice_boundary"),
    }
}

/// Reports an open edge whose block is not text.
fn open_structural() -> StepError {
    StepError::Unsupported {
        operation: Box::from("open_slice_structural"),
    }
}

/// One host block beside an open edge.
type TextEdge<'a> = (usize, &'a Arc<Block>, usize);

/// Which slice edge a merge targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EdgeSide {
    /// First block of the slice.
    Start,
    /// Last block of the slice.
    End,
}

/// Merges a host fragment into the first or last block of a slice.
fn merge_into(
    blocks: &mut [Arc<Block>],
    host: &Arc<Block>,
    fragment: &TextBlock,
    side: EdgeSide,
) -> Result<(), StepError> {
    let slot = match side {
        EdgeSide::Start => blocks.first_mut(),
        EdgeSide::End => blocks.last_mut(),
    };
    let Some(block) = slot else {
        return Err(open_boundary());
    };
    if block.kind() != host.kind() {
        return Err(StepError::Unsupported {
            operation: Box::from("open_slice_kind"),
        });
    }
    let text = block.as_textblock().ok_or_else(open_structural)?;
    let mut items = Vec::with_capacity(text.items().len().saturating_add(fragment.items().len()));
    match side {
        EdgeSide::Start => {
            items.extend(fragment.items().iter().cloned());
            items.extend(text.items().iter().cloned());
        }
        EdgeSide::End => {
            items.extend(text.items().iter().cloned());
            items.extend(fragment.items().iter().cloned());
        }
    }
    *block = Arc::new(Block::text(
        host.kind(),
        host.attrs().clone(),
        TextBlock::new(items),
    ));
    Ok(())
}
