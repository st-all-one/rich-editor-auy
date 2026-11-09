//! Text extraction helpers for tree construction.

use alloc::string::String;

use crate::model::Block;

pub(super) fn block_text(block: &Block) -> String {
    let mut text = String::new();
    if let Some(inner) = block.as_textblock() {
        inner.text_into(&mut text);
    }
    text
}

pub(super) fn descendant_text(block: &Block, out: &mut String) {
    if let Some(inner) = block.as_textblock() {
        inner.text_into(out);
    }
    if let Some(children) = block.children() {
        for child in children {
            descendant_text(child, out);
        }
    }
}
