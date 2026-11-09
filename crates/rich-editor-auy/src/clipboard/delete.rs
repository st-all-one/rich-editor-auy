//! Deleting a document range.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::Limits;
use crate::error::StepError;
use crate::model::{Attrs, Block, Doc, InlineItem, TextBlock};
use crate::pos::{Pos, Range};
use crate::schema::Schema;
use crate::step::Slice;
use crate::step::slice_text;

use super::selection::Edge;

impl Doc {
    /// Returns whether both ends of a range sit inside the same text block.
    #[must_use]
    pub fn same_text_block(&self, range: Range) -> bool {
        matches!(
            (self.edge(range.start), self.edge(range.end)),
            (Ok(Edge::Inside(first, _)), Ok(Edge::Inside(second, _))) if first == second
        )
    }

    /// Returns the caret position after deleting a range.
    ///
    /// The caret stays at the range start, which is the position the deleted content occupied.
    #[must_use]
    pub fn caret_after_delete(&self, range: Range) -> Option<Pos> {
        let size = self.size().0;
        if range.start > range.end || range.end.0 > size {
            return None;
        }
        Some(range.start)
    }

    /// Deletes a range, keeping an editable text block at the caret.
    ///
    /// A range inside one text block removes inline runs; a range across blocks removes the
    /// complete blocks and keeps the surviving edge fragments. When the whole document is
    /// deleted, or the block at the caret cannot hold text, the schema filler is inserted.
    ///
    /// # Errors
    ///
    /// Returns an out-of-bounds error for an inverted or out-of-document range, and a limit error
    /// when the result would exceed a document limit.
    pub fn delete_range(&self, schema: &Schema, range: Range) -> Result<Self, StepError> {
        let doc_size = self.size().0;
        if range.start > range.end || range.end.0 > doc_size {
            return Err(range_error(range, doc_size));
        }
        if range.start == range.end {
            return Ok(self.clone());
        }
        if self.same_text_block(range) {
            return crate::step::apply_step(
                self,
                schema,
                &crate::step::Step::Replace {
                    from: range.start,
                    to: range.end,
                    slice: Slice::plain_text(""),
                },
            );
        }
        self.delete_blocks(schema, range)
    }

    /// Removes the complete blocks and keeps the surviving edge fragments.
    fn delete_blocks(&self, schema: &Schema, range: Range) -> Result<Self, StepError> {
        let prefix = match self.edge(range.start)? {
            Edge::Inside(index, offset) => Some(self.edge_block(index, 0, offset)?),
            Edge::BlockStart(_) | Edge::DocEnd => None,
        };
        let suffix = match self.edge(range.end)? {
            Edge::Inside(index, offset) => {
                let length = self.block_length(index)?;
                Some(self.edge_block(index, offset, length)?)
            }
            Edge::BlockStart(_) | Edge::DocEnd => None,
        };
        let from = match &prefix {
            Some((index, _)) => *index,
            None => self.boundary_index(range.start)?,
        };
        let to = match &suffix {
            Some((index, _)) => index.saturating_add(1),
            None => self.boundary_index(range.end)?,
        };
        let mut blocks: Vec<Arc<Block>> = Vec::new();
        blocks.extend(self.blocks().iter().take(from).cloned());
        match (prefix, suffix) {
            (Some((_, left)), Some((_, right))) => match left.kind() {
                kind if kind == right.kind() => {
                    blocks.push(merged_block(&left, &right));
                }
                _ => {
                    blocks.push(left);
                    blocks.push(right);
                }
            },
            (Some((_, left)), None) => blocks.push(left),
            (None, Some((_, right))) => blocks.push(right),
            (None, None) => {}
        }
        blocks.extend(self.blocks().iter().skip(to).cloned());
        let caret = from.min(blocks.len());
        ensure_editable_caret(&mut blocks, caret, schema);
        Self::new(self.attrs().clone(), blocks, &Limits::default()).map_err(StepError::Limit)
    }

    /// Returns the top-level boundary index for a position.
    fn boundary_index(&self, pos: Pos) -> Result<usize, StepError> {
        if pos == Pos(self.size().0) {
            return Ok(self.blocks().len());
        }
        crate::step::block_index_at(self, pos, true).ok_or_else(|| StepError::Unsupported {
            operation: Box::from("delete_not_on_block_boundary"),
        })
    }

    /// Returns the content length of a text block.
    fn block_length(&self, index: usize) -> Result<usize, StepError> {
        self.blocks()
            .get(index)
            .and_then(|block| block.as_textblock())
            .map(|text| usize::try_from(text.len().0).unwrap_or(usize::MAX))
            .ok_or_else(|| StepError::Unsupported {
                operation: Box::from("delete_structural_edge"),
            })
    }

    /// Returns the block at an index with one content fragment, without its anchor.
    fn edge_block(
        &self,
        index: usize,
        from: usize,
        to: usize,
    ) -> Result<(usize, Arc<Block>), StepError> {
        let Some(block) = self.blocks().get(index) else {
            return Err(StepError::Unsupported {
                operation: Box::from("delete_block_index"),
            });
        };
        let text = block.as_textblock().ok_or_else(|| StepError::Unsupported {
            operation: Box::from("delete_structural_edge"),
        })?;
        let fragment = slice_text(text, from, to)?;
        Ok((
            index,
            Arc::new(Block::text(
                block.kind(),
                block.attrs().without(crate::model::AttrId::NODE_ID),
                fragment,
            )),
        ))
    }
}

/// Returns the out-of-bounds error for a range.
pub(super) fn range_error(range: Range, size: u32) -> StepError {
    let position = if range.start > range.end {
        range.start.0
    } else {
        range.end.0
    };
    StepError::OutOfBounds { position, size }
}

/// Makes sure an editable text block sits at the caret.
fn ensure_editable_caret(blocks: &mut Vec<Arc<Block>>, caret: usize, schema: &Schema) {
    let editable = blocks
        .get(caret)
        .is_some_and(|block| block.as_textblock().is_some());
    if editable {
        return;
    }
    let Some(kind) = schema.doc_filler() else {
        return;
    };
    let filler = Arc::new(Block::text(
        kind,
        Attrs::default(),
        TextBlock::new(Vec::new()),
    ));
    if blocks.is_empty() {
        blocks.push(filler);
    } else {
        blocks.insert(caret.saturating_add(1), filler);
    }
}

/// Concatenates the surviving prefix and suffix of two text blocks.
fn merged_block(left: &Arc<Block>, right: &Arc<Block>) -> Arc<Block> {
    let mut items: Vec<InlineItem> = Vec::new();
    if let Some(text) = left.as_textblock() {
        items.extend(text.items().iter().cloned());
    }
    if let Some(text) = right.as_textblock() {
        items.extend(text.items().iter().cloned());
    }
    Arc::new(Block::text(
        left.kind(),
        left.attrs().clone(),
        TextBlock::new(items),
    ))
}
