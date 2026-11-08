#![allow(
    clippy::excessive_nesting,
    reason = "bounded explicit traversal and escape decoding"
)]

//! Document limit traversal.

use crate::error::LimitError;

use super::attrs::Attrs;
use super::block::{Block, BlockContent};

pub(super) fn check_block_limits(block: &Block, limits: &crate::Limits) -> Result<(), LimitError> {
    check_attr_limits(block.attrs(), limits)?;
    match block.content() {
        BlockContent::Text(text) => {
            let runs = u32::try_from(text.items().len()).unwrap_or(u32::MAX);
            if runs > limits.max_runs_per_block {
                return Err(LimitError::Runs {
                    actual: runs,
                    max: limits.max_runs_per_block,
                });
            }
            let mut text_bytes = 0_u64;
            for item in text.items() {
                if let super::text::InlineItem::Text(run) = item {
                    text_bytes = text_bytes
                        .saturating_add(u64::try_from(run.text.len()).unwrap_or(u64::MAX));
                    for mark in run.marks.iter_attrs() {
                        let actual = u32::try_from(mark.attrs.len()).unwrap_or(u32::MAX);
                        if actual > u32::from(limits.max_mark_attrs) {
                            return Err(LimitError::MarkAttrs {
                                actual,
                                max: u32::from(limits.max_mark_attrs),
                            });
                        }
                        check_attr_limits(&mark.attrs, limits)?;
                    }
                }
            }
            if text_bytes > u64::from(limits.max_block_text_bytes) {
                return Err(LimitError::BlockTextBytes {
                    actual: text_bytes,
                    max: limits.max_block_text_bytes,
                });
            }
        }
        BlockContent::Blocks(children) => {
            for child in children {
                check_block_limits(child, limits)?;
            }
        }
        BlockContent::Atom(atom) => check_attr_limits(&atom.attrs, limits)?,
    }
    Ok(())
}

pub(super) fn check_attr_limits(attrs: &Attrs, limits: &crate::Limits) -> Result<(), LimitError> {
    let actual = u32::try_from(attrs.len()).unwrap_or(u32::MAX);
    if actual > u32::from(limits.max_attrs_per_node) {
        return Err(LimitError::Attrs {
            actual,
            max: u32::from(limits.max_attrs_per_node),
        });
    }
    for attr in attrs {
        check_value_limits(&attr.value, limits)?;
    }
    Ok(())
}

fn check_value_limits(
    value: &super::attrs::AttrValue,
    limits: &crate::Limits,
) -> Result<(), LimitError> {
    match value {
        super::attrs::AttrValue::Str(text) => check_string_limit(text, limits),
        super::attrs::AttrValue::Array(values) => {
            for value in values {
                check_value_limits(value, limits)?;
            }
            Ok(())
        }
        super::attrs::AttrValue::Ref(target) => {
            if let Some(node) = &target.node {
                check_string_limit(node.as_str(), limits)?;
            }
            if let Some(version) = &target.version {
                check_string_limit(version.as_str(), limits)?;
            }
            Ok(())
        }
        super::attrs::AttrValue::Null
        | super::attrs::AttrValue::Bool(_)
        | super::attrs::AttrValue::Int(_) => Ok(()),
    }
}

fn check_string_limit(text: &str, limits: &crate::Limits) -> Result<(), LimitError> {
    let actual = u64::try_from(text.len()).unwrap_or(u64::MAX);
    if actual > u64::from(limits.max_attr_string_bytes) {
        return Err(LimitError::AttrStringBytes {
            actual,
            max: limits.max_attr_string_bytes,
        });
    }
    Ok(())
}
