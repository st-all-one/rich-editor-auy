//! Node and document attribute steps.

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;

use crate::error::StepError;
use crate::model::{Attr, AttrId, AttrValue, Block, Doc};
use crate::pos::Pos;
use crate::schema::Schema;

use super::apply::locate;

/// Sets a node attribute through the immutable path.
pub(super) fn set_node_attr(
    doc: &Doc,
    schema: &Schema,
    pos: Pos,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    set_node_attr_owned(doc.clone(), schema, pos, attr, value)
}

/// Sets a node attribute, reusing the document when uniquely owned.
pub(super) fn set_node_attr_owned(
    doc: Doc,
    schema: &Schema,
    pos: Pos,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    let (index, _start, block) = locate(&doc, pos)?;
    let attrs = if matches!(value, AttrValue::Null) {
        block.attrs().without(attr)
    } else {
        block
            .attrs()
            .with(Attr {
                id: attr,
                value: value.clone(),
            })
            .ok_or_else(|| StepError::Unknown {
                path: Box::from("attr"),
            })?
    };
    schema
        .validate_attrs(block.kind(), &attrs)
        .map_err(|error| StepError::Unknown {
            path: Box::from(error.to_string()),
        })?;
    let new_block = match block.as_textblock() {
        Some(text) => Block::text(block.kind(), attrs, text.clone()),
        None => Block::container(
            block.kind(),
            attrs,
            block
                .children()
                .map(<[Arc<Block>]>::to_vec)
                .unwrap_or_default(),
        ),
    };
    replace_block_owned(doc, index, new_block)
}

/// Sets a document-root attribute.
pub(super) fn set_doc_attr(
    doc: &Doc,
    schema: &Schema,
    attr: AttrId,
    value: &AttrValue,
) -> Result<Doc, StepError> {
    let Some(spec) = schema.doc().attrs.iter().find(|spec| spec.id == attr) else {
        return Err(StepError::Unknown {
            path: Box::from("doc_attr"),
        });
    };
    if !matches!(value, AttrValue::Null) && !Schema::value_matches(&spec.kind, value) {
        return Err(StepError::Unknown {
            path: Box::from("doc_attr_type"),
        });
    }
    let attrs = if matches!(value, AttrValue::Null) {
        doc.attrs().without(attr)
    } else {
        doc.attrs()
            .with(Attr {
                id: attr,
                value: value.clone(),
            })
            .ok_or_else(|| StepError::Unknown {
                path: Box::from("doc_attr"),
            })?
    };
    Doc::new(attrs, doc.blocks().to_vec(), &crate::Limits::default()).map_err(StepError::Limit)
}

/// Replaces one top-level block by index.
pub(super) fn replace_block_owned(doc: Doc, index: usize, block: Block) -> Result<Doc, StepError> {
    if index >= doc.blocks().len() {
        return Err(StepError::OutOfBounds {
            position: u32::try_from(index).unwrap_or(u32::MAX),
            size: doc.size().0,
        });
    }
    doc.replacing_block_owned(index, block, &crate::Limits::default())
        .map_err(StepError::Limit)
}
