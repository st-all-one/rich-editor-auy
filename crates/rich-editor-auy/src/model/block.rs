//! Blocks, text blocks, runs, and inline atoms.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::pos::{Pos, Range, Size};

use super::attrs::{AttrValue, Attrs};
use super::ids::{AttrId, NodeId, NodeKind};
use super::marks::MarkSet;
use super::text::{InlineAtom, InlineItem, TextBlock};

/// Payload of an atom block.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AtomData {
    /// Atom attributes.
    pub attrs: Attrs,
}

/// Block payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockContent {
    /// Inline text block.
    Text(TextBlock),
    /// Nested blocks.
    Blocks(Box<[Arc<Block>]>),
    /// Atomic block.
    Atom(AtomData),
}

/// A canonical block node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    kind: NodeKind,
    attrs: Attrs,
    content: BlockContent,
    size: Size,
}

impl Block {
    /// Creates a text block.
    #[must_use]
    pub fn text(kind: NodeKind, attrs: Attrs, content: TextBlock) -> Self {
        let size = content.size();
        Self {
            kind,
            attrs,
            content: BlockContent::Text(content),
            size,
        }
    }

    /// Creates a container block.
    #[must_use]
    pub fn container(kind: NodeKind, attrs: Attrs, children: Vec<Arc<Self>>) -> Self {
        let mut size = 2_u32;
        for child in &children {
            size = size.saturating_add(child.size().0);
        }
        Self {
            kind,
            attrs,
            content: BlockContent::Blocks(children.into_boxed_slice()),
            size: Size(size),
        }
    }

    /// Creates an atom block.
    #[must_use]
    pub fn atom(kind: NodeKind, attrs: Attrs, atom: AtomData) -> Self {
        Self {
            kind,
            attrs,
            content: BlockContent::Atom(atom),
            size: Size(1),
        }
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns attributes.
    #[must_use]
    pub const fn attrs(&self) -> &Attrs {
        &self.attrs
    }

    /// Returns the block payload.
    #[must_use]
    pub const fn content(&self) -> &BlockContent {
        &self.content
    }

    /// Returns the structural size.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the anchor identifier, when present.
    #[must_use]
    pub fn id(&self) -> Option<NodeId> {
        match self.attrs.get(AttrId::NODE_ID) {
            Some(AttrValue::Str(value)) => NodeId::new(value),
            _ => None,
        }
    }

    /// Returns whether this is a text block.
    #[must_use]
    pub const fn is_textblock(&self) -> bool {
        matches!(self.content, BlockContent::Text(_))
    }

    /// Returns whether this is an atom block.
    #[must_use]
    pub const fn is_atom(&self) -> bool {
        matches!(self.content, BlockContent::Atom(_))
    }

    /// Returns the text payload.
    #[must_use]
    pub const fn as_textblock(&self) -> Option<&TextBlock> {
        match &self.content {
            BlockContent::Text(block) => Some(block),
            BlockContent::Blocks(_) | BlockContent::Atom(_) => None,
        }
    }

    /// Returns child blocks.
    #[must_use]
    pub const fn children(&self) -> Option<&[Arc<Self>]> {
        match &self.content {
            BlockContent::Blocks(children) => Some(children),
            BlockContent::Text(_) | BlockContent::Atom(_) => None,
        }
    }

    /// Iterates child blocks.
    #[must_use]
    pub fn iter_children(&self) -> BlockIter<'_> {
        match self.children() {
            Some(children) => BlockIter {
                inner: children.iter(),
            },
            None => BlockIter { inner: [].iter() },
        }
    }

    /// Iterates inline items with absolute ranges.
    #[must_use]
    pub fn iter_inline(&self, base: Pos) -> InlineIter<'_> {
        match &self.content {
            BlockContent::Text(block) => InlineIter {
                items: block.items(),
                base,
                index: 0,
                offset: 0,
            },
            BlockContent::Blocks(_) | BlockContent::Atom(_) => InlineIter {
                items: &[],
                base,
                index: 0,
                offset: 0,
            },
        }
    }
}

/// Iterator over child blocks.
#[derive(Clone, Debug)]
pub struct BlockIter<'a> {
    inner: core::slice::Iter<'a, Arc<Block>>,
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a Arc<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Borrowed inline item view with an absolute range.
#[derive(Clone, Copy, Debug)]
pub enum InlineView<'a> {
    /// Text run.
    Text {
        /// Absolute range.
        range: Range<Pos>,
        /// Text.
        text: &'a str,
        /// Marks.
        marks: &'a MarkSet,
    },
    /// Atom.
    Atom {
        /// Absolute range.
        range: Range<Pos>,
        /// Atom.
        atom: &'a InlineAtom,
    },
}

/// Iterator over inline views.
#[derive(Clone, Debug)]
pub struct InlineIter<'a> {
    items: &'a [InlineItem],
    base: Pos,
    index: usize,
    offset: u32,
}

impl<'a> Iterator for InlineIter<'a> {
    type Item = InlineView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.get(self.index)?;
        let start = self.base.0.checked_add(self.offset)?;
        let size = item.size();
        let end = start.checked_add(size)?;
        self.index = self.index.checked_add(1)?;
        self.offset = self.offset.saturating_add(size);
        let range = Range::new(Pos(start), Pos(end));
        match item {
            InlineItem::Text(run) => Some(InlineView::Text {
                range,
                text: &run.text,
                marks: &run.marks,
            }),
            InlineItem::Atom(atom) => Some(InlineView::Atom { range, atom }),
        }
    }
}
