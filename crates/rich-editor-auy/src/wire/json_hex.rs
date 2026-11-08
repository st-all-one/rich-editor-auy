//! Strict four-digit unicode escape.

use crate::error::ParseError;

use super::json_parser::Parser;

pub(super) fn hex4(parser: &mut Parser<'_>) -> Result<u32, ParseError> {
    let mut value = 0_u32;
    for _ in 0..4 {
        let Some(byte) = parser.peek() else {
            return Err(parser.error("invalid_escape"));
        };
        let digit = match byte {
            b'0'..=b'9' => u32::from(byte.saturating_sub(b'0')),
            b'a'..=b'f' => u32::from(byte.saturating_sub(b'a')).saturating_add(10),
            b'A'..=b'F' => u32::from(byte.saturating_sub(b'A')).saturating_add(10),
            _ => return Err(parser.error("invalid_escape")),
        };
        parser.advance();
        value = value.saturating_mul(16).saturating_add(digit);
    }
    Ok(value)
}
