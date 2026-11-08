//! Logical position conversion and scalar snapping.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::string::String;

use crate::model::Doc;
use crate::pos::Pos;

/// Converts a document position to UTF-16 code units inside its text block.
#[must_use]
pub fn pos_to_utf16(doc: &Doc, pos: Pos) -> Option<u32> {
    let (_, text, offset) = text_position(doc, pos)?;
    let prefix = text.get(..offset)?;
    u32::try_from(prefix.encode_utf16().count()).ok()
}

/// Converts a UTF-16 offset inside a block to a document position.
#[must_use]
pub fn utf16_to_pos(doc: &Doc, block: u32, utf16: u32) -> Option<Pos> {
    let range = doc.block_range(block)?;
    let textblock = doc.block(block)?.as_textblock()?;
    let mut text = String::new();
    textblock.text_into(&mut text);
    let mut units = 0_u32;
    for (offset, character) in text.char_indices() {
        if units == utf16 {
            let position = range
                .start
                .checked_add(1)?
                .checked_add(u32::try_from(offset).ok()?)?;
            return Some(position);
        }
        units = units.saturating_add(u32::try_from(character.len_utf16()).unwrap_or(1));
    }
    if units == utf16 {
        return range
            .start
            .checked_add(1)?
            .checked_add(u32::try_from(text.len()).ok()?);
    }
    None
}

/// Snaps a position to the nearest previous scalar boundary inside its block.
#[must_use]
pub fn snap_to_scalar(doc: &Doc, pos: Pos) -> Pos {
    let Some((_, text, offset)) = text_position(doc, pos) else {
        return pos;
    };
    let mut boundary = offset.min(text.len());
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary = boundary.saturating_sub(1);
    }
    pos.checked_sub(u32::try_from(offset.saturating_sub(boundary)).unwrap_or(0))
        .unwrap_or(pos)
}

fn text_position(doc: &Doc, pos: Pos) -> Option<(u32, String, usize)> {
    let mut offset = 0_u32;
    for (index, block) in doc.blocks().iter().enumerate() {
        let start = offset;
        let end = start.checked_add(block.size().0)?;
        if pos.0 >= start && pos.0 <= end {
            let textblock = block.as_textblock()?;
            let mut text = String::new();
            textblock.text_into(&mut text);
            let local = pos.0.checked_sub(start)?.checked_sub(1)?;
            let local = usize::try_from(local).ok()?;
            if local > text.len() {
                return None;
            }
            return Some((u32::try_from(index).ok()?, text, local));
        }
        offset = end;
    }
    None
}
