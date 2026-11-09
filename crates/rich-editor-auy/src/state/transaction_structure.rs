//! Structural transaction commands.

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Attr, AttrId, AttrValue, Attrs, Block, Doc, Fragment, InlineItem, NodeKind};
use crate::pos::Pos;
use crate::schema::Schema;
use crate::step::{Slice, Step};

use super::Transaction;

impl Transaction {
    /// Changes the type of the top-level text block containing a position.
    ///
    /// The step is a structural replacement, so the inverse restores the previous type and
    /// attributes. The node `id` is preserved unless `attrs` sets it explicitly.
    ///
    /// # Errors
    ///
    /// Returns [`StepError::Unsupported`] when the target is not a top-level text block, when the
    /// new type cannot hold inline text, when a run carries a mark the new type forbids, or when
    /// the attributes do not match the new type. Returns [`StepError::OutOfBounds`] when the
    /// position is outside the document.
    pub fn set_block_type(
        &mut self,
        schema: &Schema,
        at: Pos,
        kind: NodeKind,
        attrs: Attrs,
    ) -> Result<(), StepError> {
        let doc = self
            .base
            .as_ref()
            .ok_or_else(|| unsupported("set_block_type_without_base"))?;
        let (index, block) = doc.block_at(at).ok_or_else(|| out_of_bounds(doc, at))?;
        let range = u32::try_from(index)
            .ok()
            .and_then(|index| doc.block_range(index))
            .ok_or_else(|| out_of_bounds(doc, at))?;
        let text = block
            .as_textblock()
            .ok_or_else(|| unsupported("set_block_type_structural"))?;
        let Some(spec) = schema.node(kind) else {
            return Err(unsupported("set_block_type_unknown"));
        };
        if spec.inline || spec.atom {
            return Err(unsupported("set_block_type_not_block"));
        }
        let accepts_text = schema
            .node_kind("text")
            .is_some_and(|text| schema.content_allows(kind, &[text]));
        if !accepts_text {
            return Err(unsupported("set_block_type_content"));
        }
        let forbidden_mark = text.items().iter().any(|item| match item {
            InlineItem::Text(run) => schema
                .marks()
                .iter()
                .any(|mark| run.marks.contains(mark.id) && !schema.mark_allowed(kind, mark.id)),
            InlineItem::Atom(_) => false,
        });
        if forbidden_mark {
            return Err(unsupported("set_block_type_marks"));
        }
        let attrs = preserve_id(block, attrs);
        schema
            .validate_attrs(kind, &attrs)
            .map_err(|_| unsupported("set_block_type_attrs"))?;
        let replacement = Block::text(kind, attrs, text.clone());
        self.steps.push(Step::Replace {
            from: range.start,
            to: range.end,
            slice: Slice::from_fragment(Fragment::new(Vec::from([Arc::new(replacement)]))),
        });
        Ok(())
    }
}

/// Keeps the stable node id when the caller does not provide one.
fn preserve_id(block: &Block, attrs: Attrs) -> Attrs {
    if attrs.get(AttrId::NODE_ID).is_some() {
        return attrs;
    }
    match block.attrs().get(AttrId::NODE_ID) {
        Some(AttrValue::Str(value)) => attrs
            .with(Attr {
                id: AttrId::NODE_ID,
                value: AttrValue::Str(value.clone()),
            })
            .unwrap_or(attrs),
        _ => attrs,
    }
}

/// Builds an unsupported-operation error.
fn unsupported(operation: &str) -> StepError {
    StepError::Unsupported {
        operation: operation.into(),
    }
}

/// Builds an out-of-bounds error for a position.
fn out_of_bounds(doc: &Doc, at: Pos) -> StepError {
    StepError::OutOfBounds {
        position: at.0,
        size: doc.size().0,
    }
}
