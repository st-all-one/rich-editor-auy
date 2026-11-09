//! Copying a document range into a slice.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{
    AtomData, AttrId, Attrs, Block, BlockContent, Doc, Fragment, InlineAtom, InlineItem, NodeKind,
    TextBlock,
};
use crate::pos::{Pos, Range};
use crate::step::Slice;
use crate::step::slice_text;

/// One end of a range, classified by its relation to block boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Edge {
    /// End of the document.
    DocEnd,
    /// Start of the block at this index.
    BlockStart(usize),
    /// Content offset inside the text block at this index.
    Inside(usize, usize),
}

impl Doc {
    /// Copies a range into a slice.
    ///
    /// A range inside one text block copies inline runs and marks; a range that spans blocks
    /// copies the partial edge blocks plus every complete block between them, with anchors
    /// removed.
    ///
    /// # Errors
    ///
    /// Returns an out-of-bounds error for an inverted or out-of-document range, and an
    /// unsupported error when an edge falls on a structural token or inside a container.
    pub fn slice(&self, range: Range) -> Result<Slice, StepError> {
        let doc_size = self.size().0;
        if range.start > range.end || range.end.0 > doc_size {
            return Err(super::delete::range_error(range, doc_size));
        }
        let start = self.edge(range.start)?;
        let end = self.edge(range.end)?;
        if let (Edge::Inside(first, from), Edge::Inside(second, to)) = (start, end)
            && first == second
        {
            let block = self.text_block(first)?;
            return Ok(Slice::from_textblock(slice_text(block, from, to)?));
        }
        let open_start = u8::from(matches!(start, Edge::Inside(..)));
        let open_end = u8::from(matches!(end, Edge::Inside(..)));
        let mut blocks: Vec<Arc<Block>> = Vec::new();
        let whole_from = match start {
            Edge::DocEnd => self.blocks().len(),
            Edge::BlockStart(index) => index,
            Edge::Inside(index, offset) => {
                let block = self.text_block(index)?;
                let length = usize::try_from(block.len().0).unwrap_or(usize::MAX);
                blocks.push(self.partial_block(index, offset, length)?);
                index.saturating_add(1)
            }
        };
        let whole_to = match end {
            Edge::DocEnd => self.blocks().len(),
            Edge::BlockStart(index) | Edge::Inside(index, _) => index,
        };
        let whole = (whole_from..whole_to).filter_map(|index| self.blocks().get(index));
        blocks.extend(whole.map(strip_block_ids));
        if let Edge::Inside(index, offset) = end {
            blocks.push(self.partial_block(index, 0, offset)?);
        }
        let mut slice = Slice::from_fragment(Fragment::new(blocks));
        slice.open_start = open_start;
        slice.open_end = open_end;
        Ok(slice)
    }

    /// Copies one edge fragment of a text block.
    fn partial_block(&self, index: usize, from: usize, to: usize) -> Result<Arc<Block>, StepError> {
        let text = slice_text(self.text_block(index)?, from, to)?;
        Ok(Arc::new(Block::text(
            self.block_kind(index)?,
            self.block_attrs(index)?,
            text,
        )))
    }

    /// Borrows the text block at an index.
    fn text_block(&self, index: usize) -> Result<&TextBlock, StepError> {
        self.blocks()
            .get(index)
            .and_then(|block| block.as_textblock())
            .ok_or_else(|| StepError::Unsupported {
                operation: Box::from("slice_structural"),
            })
    }

    /// Returns the kind of the top-level block at an index.
    fn block_kind(&self, index: usize) -> Result<NodeKind, StepError> {
        self.blocks()
            .get(index)
            .map(|block| block.kind())
            .ok_or_else(|| StepError::Unsupported {
                operation: Box::from("slice_block_index"),
            })
    }

    /// Returns the attributes of the top-level block at an index, without its anchor.
    fn block_attrs(&self, index: usize) -> Result<Attrs, StepError> {
        self.blocks()
            .get(index)
            .map(|block| block.attrs().without(AttrId::NODE_ID))
            .ok_or_else(|| StepError::Unsupported {
                operation: Box::from("slice_block_index"),
            })
    }

    /// Classifies one position as a block boundary or a content offset.
    pub(super) fn edge(&self, pos: Pos) -> Result<Edge, StepError> {
        if pos == Pos(self.size().0) {
            return Ok(Edge::DocEnd);
        }
        if let Some(index) = crate::step::block_index_at(self, pos, true) {
            return Ok(Edge::BlockStart(index));
        }
        let Some((index, block)) = self.block_at(pos) else {
            return Err(StepError::OutOfBounds {
                position: pos.0,
                size: self.size().0,
            });
        };
        if block.as_textblock().is_none() {
            return Err(StepError::Unsupported {
                operation: Box::from("slice_token"),
            });
        }
        let start = self
            .block_range(u32::try_from(index).unwrap_or(u32::MAX))
            .map_or(0, |range| range.start.0);
        let local = pos.0.saturating_sub(start);
        if local == 0 {
            return Ok(Edge::BlockStart(index));
        }
        Ok(Edge::Inside(
            index,
            usize::try_from(local.saturating_sub(1)).unwrap_or(usize::MAX),
        ))
    }
}

/// Returns a copy of a block without block anchors.
fn strip_block_ids(block: &Arc<Block>) -> Arc<Block> {
    let attrs = block.attrs().without(AttrId::NODE_ID);
    match block.content() {
        BlockContent::Text(text) => {
            let items = text.items().iter().map(strip_atom_id).collect::<Vec<_>>();
            Arc::new(Block::text(block.kind(), attrs, TextBlock::new(items)))
        }
        BlockContent::Blocks(children) => Arc::new(Block::container(
            block.kind(),
            attrs,
            children.iter().map(strip_block_ids).collect::<Vec<_>>(),
        )),
        BlockContent::Atom(data) => Arc::new(Block::atom(
            block.kind(),
            attrs,
            AtomData {
                attrs: data.attrs.without(AttrId::NODE_ID),
            },
        )),
    }
}

/// Returns the item without an inline atom anchor.
fn strip_atom_id(item: &InlineItem) -> InlineItem {
    match item {
        InlineItem::Text(run) => InlineItem::Text(run.clone()),
        InlineItem::Atom(atom) => InlineItem::Atom(InlineAtom {
            kind: atom.kind,
            attrs: atom.attrs.without(AttrId::NODE_ID),
        }),
    }
}
