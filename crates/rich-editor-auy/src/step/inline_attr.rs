//! Inline-atom attribute edits.

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Attr, AttrId, AttrValue, Attrs, Block, Doc, InlineAtom, InlineItem, TextBlock};
use crate::pos::Pos;
use crate::schema::Schema;

use super::apply::locate;

/// Applies an inline atom attribute change.
#[allow(
    clippy::too_many_arguments,
    reason = "one step expands to its explicit fields"
)]
pub(super) fn set_inline_attr(
    doc: &Doc,
    schema: &Schema,
    from: Pos,
    to: Pos,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    set_inline_attr_owned(doc.clone(), schema, from, to, attr, value)
}

/// Applies an inline atom attribute change, reusing the document when uniquely owned.
#[allow(
    clippy::too_many_arguments,
    reason = "one step expands to its explicit fields"
)]
pub(super) fn set_inline_attr_owned(
    doc: Doc,
    schema: &Schema,
    from: Pos,
    to: Pos,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    let (index, block_start, block) = locate(&doc, from)?;
    let text = block.as_textblock().ok_or_else(|| StepError::Unsupported {
        operation: Box::from("inline_attr_outside_textblock"),
    })?;
    let local = atom_offset(block_start, from, to)?;
    let atom = atom_at(text, local)?;
    let kind = atom.kind;
    let attrs = atom_attrs(schema, atom, attr, value)?;
    let mut items = Vec::with_capacity(text.items().len());
    let mut cursor = 0_u32;
    for item in text.items() {
        if cursor == local {
            items.push(InlineItem::Atom(InlineAtom {
                kind,
                attrs: attrs.clone(),
            }));
        } else {
            items.push(item.clone());
        }
        cursor = cursor.saturating_add(item.size());
    }
    let new_block = Block::text(block.kind(), block.attrs().clone(), TextBlock::new(items));
    doc.replacing_block_owned(index, new_block, &crate::Limits::default())
        .map_err(StepError::Limit)
}

/// Returns the previous value of an inline atom attribute.
pub(super) fn inline_attr_value(
    doc: &Doc,
    from: Pos,
    to: Pos,
    attr: AttrId,
) -> Result<AttrValue, StepError> {
    let (_, block_start, block) = locate(doc, from)?;
    let text = block.as_textblock().ok_or_else(|| StepError::Unsupported {
        operation: Box::from("inline_attr_outside_textblock"),
    })?;
    let local = atom_offset(block_start, from, to)?;
    let atom = atom_at(text, local)?;
    Ok(atom.attrs.get(attr).cloned().unwrap_or(AttrValue::Null))
}

/// Returns validated atom attributes with one attribute replaced or removed.
fn atom_attrs(
    schema: &Schema,
    atom: &InlineAtom,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Attrs, StepError> {
    let attrs = if matches!(value, AttrValue::Null) {
        atom.attrs.without(attr)
    } else {
        atom.attrs
            .with(Attr {
                id: attr,
                value: value.clone(),
            })
            .ok_or_else(|| StepError::Unknown {
                path: Box::from("inline_attr"),
            })?
    };
    schema
        .validate_attrs(atom.kind, &attrs)
        .map_err(|error| StepError::Unknown {
            path: Box::from(error.to_string()),
        })?;
    Ok(attrs)
}

/// Returns the content offset of a one-byte inline range.
fn atom_offset(block_start: u32, from: Pos, to: Pos) -> Result<u32, StepError> {
    if from >= to || to.0 != from.0.saturating_add(1) {
        return Err(StepError::Unsupported {
            operation: Box::from("inline_attr_range"),
        });
    }
    from.0
        .checked_sub(block_start.saturating_add(1))
        .ok_or_else(|| StepError::Unsupported {
            operation: Box::from("inline_attr_structural_token"),
        })
}

/// Returns the inline atom starting at a content offset.
fn atom_at(text: &TextBlock, local: u32) -> Result<&InlineAtom, StepError> {
    let mut cursor = 0_u32;
    for item in text.items() {
        if cursor == local {
            return match item {
                InlineItem::Atom(atom) => Ok(atom),
                InlineItem::Text(_) => Err(StepError::Unsupported {
                    operation: Box::from("inline_attr_not_atom"),
                }),
            };
        }
        cursor = cursor.saturating_add(item.size());
    }
    Err(StepError::Unsupported {
        operation: Box::from("inline_attr_missing_atom"),
    })
}
