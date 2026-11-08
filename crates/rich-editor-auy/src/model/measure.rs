//! Internal document measurements.

use alloc::string::String;
use alloc::vec::Vec;

use crate::pos::{Pos, Range};

use super::{Anchor, AttrValue, Block, Reference};

pub(super) fn count_descendants(block: &Block) -> u32 {
    match block.children() {
        Some(children) => {
            let mut total = 0_u32;
            for child in children {
                total = total.saturating_add(1);
                total = total.saturating_add(count_descendants(child));
            }
            total
        }
        None => 0,
    }
}

pub(super) fn text_of_block(block: &Block, out: &mut String) {
    if let Some(text) = block.as_textblock() {
        text.text_into(out);
    }
    if let Some(children) = block.children() {
        for child in children {
            text_of_block(child, out);
        }
    }
}

pub(super) fn collect_anchors(block: &Block, start: u32, out: &mut Vec<Anchor>) {
    let end = start.saturating_add(block.size().0);
    if let Some(id) = block.id() {
        out.push(Anchor {
            id,
            range: Range::new(Pos(start), Pos(end)),
        });
    }
    if let Some(children) = block.children() {
        let mut cursor = start.saturating_add(1);
        for child in children {
            collect_anchors(child, cursor, out);
            cursor = cursor.saturating_add(child.size().0);
        }
    }
}

pub(super) fn collect_references(block: &Block, start: u32, out: &mut Vec<Reference>) {
    let end = start.saturating_add(block.size().0);
    for attr in block.attrs() {
        if let AttrValue::Ref(target) = &attr.value {
            out.push(Reference {
                source: Range::new(Pos(start), Pos(end)),
                target: target.clone(),
            });
        }
    }
    if let Some(children) = block.children() {
        let mut cursor = start.saturating_add(1);
        for child in children {
            collect_references(child, cursor, out);
            cursor = cursor.saturating_add(child.size().0);
        }
    }
}

pub(super) fn block_depth(block: &Block, current: u32) -> u32 {
    let mut max = current;
    if let Some(children) = block.children() {
        for child in children {
            max = max.max(block_depth(child, current.saturating_add(1)));
        }
    }
    max
}
