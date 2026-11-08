#![allow(
    clippy::excessive_nesting,
    reason = "bounded explicit traversal and escape decoding"
)]

//! JSON string escape decoding.

use alloc::string::String;

use crate::error::ParseError;

use super::json_parser::Parser;

pub(super) fn escape(parser: &mut Parser<'_>, out: &mut String) -> Result<(), ParseError> {
    let Some(byte) = parser.peek() else {
        return Err(parser.error("unterminated_escape"));
    };
    parser.advance();
    match byte {
        b'"' => out.push('"'),
        b'\\' => out.push('\\'),
        b'/' => out.push('/'),
        b'b' => out.push('\u{0008}'),
        b'f' => out.push('\u{000c}'),
        b'n' => out.push('\n'),
        b'r' => out.push('\r'),
        b't' => out.push('\t'),
        b'u' => {
            let value = super::json_hex::hex4(parser)?;
            let scalar = if (0xd800..=0xdbff).contains(&value) {
                if parser.peek() != Some(b'\\') {
                    return Err(parser.error("invalid_surrogate_pair"));
                }
                parser.advance();
                if parser.peek() != Some(b'u') {
                    return Err(parser.error("invalid_surrogate_pair"));
                }
                parser.advance();
                let low = super::json_hex::hex4(parser)?;
                if !(0xdc00..=0xdfff).contains(&low) {
                    return Err(parser.error("invalid_surrogate_pair"));
                }
                0x10000_u32
                    .wrapping_add(value.wrapping_sub(0xd800).wrapping_shl(10))
                    .wrapping_add(low.wrapping_sub(0xdc00))
            } else if (0xdc00..=0xdfff).contains(&value) {
                return Err(parser.error("invalid_surrogate_pair"));
            } else {
                value
            };
            let Some(character) = char::from_u32(scalar) else {
                return Err(parser.error("invalid_escape"));
            };
            out.push(character);
        }
        _ => return Err(parser.error("invalid_escape")),
    }
    Ok(())
}
