//! Closed edit steps and their effect on canonical documents.

mod apply;
mod invert;
mod map;
mod structural;
mod text_edit;
mod text_splice;

pub use apply::apply_step;
pub(crate) use apply::apply_step_owned;
#[cfg(feature = "visits")]
pub use apply::take_locate_visits;
pub use invert::invert_step;
pub use map::{Mapping, StepMap, StepMapRange};

use alloc::boxed::Box;

use crate::error::StepError;
use crate::model::{AttrId, AttrValue, Doc, Fragment, Mark, TextBlock};
use crate::pos::{Pos, Range};
use crate::schema::Schema;

use apply::replace_len;

/// Inline content carried by a replace step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slice {
    /// Closed fragment for structural replacements.
    pub content: Fragment,
    /// Plain inline text for text-block replacements.
    pub text: Box<str>,
    /// Inline runs with marks for exact inverses.
    pub runs: Option<TextBlock>,
    /// Number of opening tokens to keep from the start.
    pub open_start: u8,
    /// Number of closing tokens to keep from the end.
    pub open_end: u8,
}

impl Slice {
    /// Creates a plain-text slice.
    #[must_use]
    pub fn plain_text(text: &str) -> Self {
        Self {
            content: Fragment::default(),
            text: Box::from(text),
            runs: None,
            open_start: 0,
            open_end: 0,
        }
    }

    /// Creates a fragment slice.
    #[must_use]
    pub fn from_fragment(content: Fragment) -> Self {
        Self {
            content,
            text: Box::from(""),
            runs: None,
            open_start: 0,
            open_end: 0,
        }
    }

    /// Creates a slice that preserves inline runs and marks.
    #[must_use]
    pub fn from_textblock(block: TextBlock) -> Self {
        let mut text = alloc::string::String::new();
        block.text_into(&mut text);
        Self {
            content: Fragment::default(),
            text: text.into_boxed_str(),
            runs: Some(block),
            open_start: 0,
            open_end: 0,
        }
    }

    /// Returns whether the slice carries no content.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
            && self.content.is_empty()
            && self.runs.as_ref().is_none_or(TextBlock::is_empty)
    }
}

/// One atomic edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Replace a range with a slice.
    Replace {
        /// Start position.
        from: Pos,
        /// End position.
        to: Pos,
        /// Replacement.
        slice: Slice,
    },
    /// Replace around a gap; used for block splitting.
    ReplaceAround {
        /// Start position.
        from: Pos,
        /// End position.
        to: Pos,
        /// Gap start.
        gap_from: Pos,
        /// Gap end.
        gap_to: Pos,
        /// Number of inserted boundary tokens.
        insert: u32,
        /// Replacement.
        slice: Slice,
    },
    /// Add a mark to a range.
    AddMark {
        /// Start position.
        from: Pos,
        /// End position.
        to: Pos,
        /// Mark to add.
        mark: Mark,
    },
    /// Remove a mark from a range.
    RemoveMark {
        /// Start position.
        from: Pos,
        /// End position.
        to: Pos,
        /// Mark to remove.
        mark: Mark,
    },
    /// Set an attribute on the block containing a position.
    SetNodeAttr {
        /// Position inside the block.
        pos: Pos,
        /// Attribute index.
        attr: AttrId,
        /// New value.
        value: AttrValue,
    },
    /// Set a document-root attribute.
    SetDocAttr {
        /// Attribute index.
        attr: AttrId,
        /// New value.
        value: AttrValue,
    },
}

impl Step {
    /// Applies the step to a document.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the step is invalid for the document or schema.
    pub fn apply(&self, doc: &Doc, schema: &Schema) -> Result<Doc, StepError> {
        apply_step(doc, schema, self)
    }

    /// Produces the inverse step against the pre-step document.
    ///
    /// # Errors
    ///
    /// Returns an error when the step cannot be inverted by the MVP algebra.
    pub fn invert(&self, doc: &Doc) -> Result<Self, StepError> {
        invert_step(doc, self)
    }

    /// Returns the position map for the step.
    #[must_use]
    pub fn step_map(&self) -> StepMap {
        match self {
            Self::Replace { from, to, slice } => {
                StepMap::replacement(*from, to.0.saturating_sub(from.0), replace_len(slice))
            }
            Self::ReplaceAround {
                from, to, insert, ..
            } => StepMap::replacement(*from, to.0.saturating_sub(from.0), *insert),
            Self::AddMark { .. }
            | Self::RemoveMark { .. }
            | Self::SetNodeAttr { .. }
            | Self::SetDocAttr { .. } => StepMap::identity(),
        }
    }

    /// Returns whether the step changes no content.
    #[must_use]
    pub fn is_noop(&self) -> bool {
        match self {
            Self::Replace { from, to, slice } => from == to && slice.is_empty(),
            Self::AddMark { from, to, .. } | Self::RemoveMark { from, to, .. } => from == to,
            _ => false,
        }
    }
}

/// Inclusive document range covered by a step.
#[must_use]
pub fn range_of(step: &Step) -> Option<Range> {
    match step {
        Step::Replace { from, to, .. }
        | Step::ReplaceAround { from, to, .. }
        | Step::AddMark { from, to, .. }
        | Step::RemoveMark { from, to, .. } => Some(Range::new(*from, *to)),
        Step::SetNodeAttr { .. } | Step::SetDocAttr { .. } => None,
    }
}
