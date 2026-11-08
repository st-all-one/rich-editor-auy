//! Inline text runs and text blocks.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::LimitError;
use crate::math::usize_to_u32;
use crate::pos::Size;

use super::attrs::Attrs;
use super::ids::NodeKind;
use super::marks::MarkSet;

/// Maximum bytes in a canonical text run.
pub const MAX_RUN_BYTES: usize = 4096;

/// Payload of an inline atom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineAtom {
    /// Atom node kind.
    pub kind: NodeKind,
    /// Atom attributes.
    pub attrs: Attrs,
}

/// A text run with one mark set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// UTF-8 text.
    pub text: String,
    /// Marks applied to the whole run.
    pub marks: MarkSet,
}

impl Run {
    /// Creates a run.
    #[must_use]
    pub fn new(text: String, marks: MarkSet) -> Self {
        Self { text, marks }
    }

    /// Returns the byte length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns whether the run is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// One inline item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlineItem {
    /// Text run.
    Text(Run),
    /// Atomic inline node.
    Atom(InlineAtom),
}

impl InlineItem {
    /// Returns the item size.
    #[must_use]
    pub fn size(&self) -> u32 {
        match self {
            Self::Text(run) => usize_to_u32(run.text.len()).unwrap_or(u32::MAX),
            Self::Atom(_) => 1,
        }
    }
}

/// A block with inline content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBlock {
    items: Vec<InlineItem>,
    len: Size,
}

impl TextBlock {
    /// Creates a text block, normalizing empty and adjacent runs.
    #[must_use]
    pub fn new(items: Vec<InlineItem>) -> Self {
        let mut block = Self {
            items: Vec::with_capacity(items.len()),
            len: Size::ZERO,
        };
        for item in items {
            block.push(item);
        }
        block
    }

    /// Creates an empty text block.
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Appends an item, merging adjacent runs with equal marks.
    pub fn push(&mut self, item: InlineItem) {
        match item {
            InlineItem::Text(run) => {
                if run.is_empty() {
                    return;
                }
                if let Some(InlineItem::Text(previous)) = self.items.last_mut()
                    && previous.marks == run.marks
                {
                    previous.text.push_str(&run.text);
                    self.recompute_len();
                    return;
                }
                self.items.push(InlineItem::Text(run));
            }
            InlineItem::Atom(atom) => self.items.push(InlineItem::Atom(atom)),
        }
        self.recompute_len();
    }

    /// Returns inline items.
    #[must_use]
    pub fn items(&self) -> &[InlineItem] {
        &self.items
    }

    /// Returns text byte length, excluding structural tokens.
    #[must_use]
    pub const fn len(&self) -> Size {
        self.len
    }

    /// Returns whether the block has no inline content.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the size including open and close tokens.
    #[must_use]
    pub fn size(&self) -> Size {
        match self.len.0.checked_add(2) {
            Some(value) => Size(value),
            None => Size(u32::MAX),
        }
    }

    /// Appends plain text to `out`.
    pub fn text_into(&self, out: &mut String) {
        for item in &self.items {
            if let InlineItem::Text(run) = item {
                out.push_str(&run.text);
            }
        }
    }

    /// Returns the run containing an offset relative to the content start.
    #[must_use]
    pub fn run_at(&self, offset: Size) -> Option<(usize, Size)> {
        let mut cursor = 0_u32;
        for (index, item) in self.items.iter().enumerate() {
            let size = item.size();
            let next = cursor.checked_add(size)?;
            if offset.0 >= cursor && offset.0 < next {
                return Some((index, Size(cursor)));
            }
            cursor = next;
        }
        None
    }

    /// Normalizes runs, enforcing the run byte ceiling.
    ///
    /// # Errors
    ///
    /// Returns a limit error when a single run cannot be represented.
    pub fn normalize(&mut self, max_run_bytes: u32) -> Result<(), LimitError> {
        let ceiling = usize::try_from(max_run_bytes).unwrap_or(MAX_RUN_BYTES);
        let mut normalized = Vec::new();
        for item in core::mem::take(&mut self.items) {
            match item {
                InlineItem::Text(run) => {
                    let text = run.text;
                    let marks = run.marks;
                    let mut start = 0_usize;
                    while start < text.len() {
                        let mut end = text.len().min(start.saturating_add(ceiling.max(1)));
                        while end > start && !text.is_char_boundary(end) {
                            end = end.saturating_sub(1);
                        }
                        if end == start {
                            return Err(LimitError::RunBytes {
                                actual: u64::try_from(text.len()).unwrap_or(u64::MAX),
                                max: max_run_bytes,
                            });
                        }
                        let Some(slice) = text.get(start..end) else {
                            return Err(LimitError::RunBytes {
                                actual: u64::try_from(text.len()).unwrap_or(u64::MAX),
                                max: max_run_bytes,
                            });
                        };
                        normalized.push(InlineItem::Text(Run::new(
                            String::from(slice),
                            marks.clone(),
                        )));
                        start = end;
                    }
                }
                InlineItem::Atom(atom) => normalized.push(InlineItem::Atom(atom)),
            }
        }
        self.items = normalized;
        self.recompute_len();
        Ok(())
    }

    fn recompute_len(&mut self) {
        let mut total = 0_u32;
        for item in &self.items {
            total = total.saturating_add(item.size());
        }
        self.len = Size(total);
    }
}
