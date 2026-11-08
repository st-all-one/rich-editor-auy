//! Strict JSON parser.
#![allow(
    clippy::redundant_pub_crate,
    reason = "JSON machinery is crate-internal"
)]
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::error::ParseError;

use super::json::{JsonKind, JsonValue};
use super::json_escape::escape;

/// Parses one strict JSON value.
///
/// # Errors
///
/// Returns a located error for malformed JSON, duplicate keys, floats, or exponents.
pub(crate) fn parse_json(input: &str, max_depth: u16) -> Result<JsonValue, ParseError> {
    let mut parser = Parser {
        bytes: input.as_bytes(),
        pos: 0,
        depth: 0,
        max_depth,
    };
    parser.skip_space();
    let value = parser.value()?;
    parser.skip_space();
    if parser.pos != parser.bytes.len() {
        return Err(ParseError::new("trailing_data", parser.offset()));
    }
    Ok(value)
}

pub(crate) struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    depth: u16,
    max_depth: u16,
}

impl Parser<'_> {
    fn value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_space();
        if self.depth > self.max_depth {
            return Err(self.error("depth_limit"));
        }
        let start = self.offset();
        match self.peek() {
            Some(b'{') => self.object(start),
            Some(b'[') => self.array(start),
            Some(b'"') => self
                .string()
                .map(|value| JsonValue::new(JsonKind::Str(value), start)),
            Some(b't') => self.literal(b"true", JsonKind::Bool(true), start),
            Some(b'f') => self.literal(b"false", JsonKind::Bool(false), start),
            Some(b'n') => self.literal(b"null", JsonKind::Null, start),
            Some(b'-' | b'0'..=b'9') => self
                .number()
                .map(|value| JsonValue::new(JsonKind::Int(value), start)),
            Some(_) => Err(self.error("unexpected_token")),
            None => Err(self.error("unexpected_end")),
        }
    }
    fn object(&mut self, start: u32) -> Result<JsonValue, ParseError> {
        if self.depth >= self.max_depth {
            return Err(self.error("depth_limit"));
        }
        self.depth = self.depth.saturating_add(1);
        let result = self.object_body();
        self.depth = self.depth.saturating_sub(1);
        result.map(|kind| JsonValue::new(kind, start))
    }

    fn object_body(&mut self) -> Result<JsonKind, ParseError> {
        self.advance();
        self.skip_space();
        let mut members: Vec<(Box<str>, JsonValue)> = Vec::new();
        if self.peek() == Some(b'}') {
            self.advance();
            return Ok(JsonKind::Object(members.into_boxed_slice()));
        }
        loop {
            self.skip_space();
            let key = self.string()?;
            self.skip_space();
            if self.peek() != Some(b':') {
                return Err(self.error("expected_colon"));
            }
            self.advance();
            let value = self.value()?;
            members.push((key, value));
            self.skip_space();
            match self.peek() {
                Some(b',') => self.advance(),
                Some(b'}') => {
                    self.advance();
                    break;
                }
                _ => return Err(self.error("expected_object_end")),
            }
        }
        let mut seen = BTreeSet::new();
        for (name, _) in &members {
            if !seen.insert(name.as_ref()) {
                return Err(self.error("duplicate_key"));
            }
        }
        Ok(JsonKind::Object(members.into_boxed_slice()))
    }

    fn array(&mut self, start: u32) -> Result<JsonValue, ParseError> {
        if self.depth >= self.max_depth {
            return Err(self.error("depth_limit"));
        }
        self.depth = self.depth.saturating_add(1);
        let result = self.array_body();
        self.depth = self.depth.saturating_sub(1);
        result.map(|kind| JsonValue::new(kind, start))
    }

    fn array_body(&mut self) -> Result<JsonKind, ParseError> {
        self.advance();
        self.skip_space();
        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.advance();
            return Ok(JsonKind::Array(values.into_boxed_slice()));
        }
        loop {
            values.push(self.value()?);
            self.skip_space();
            match self.peek() {
                Some(b',') => self.advance(),
                Some(b']') => {
                    self.advance();
                    break;
                }
                _ => return Err(self.error("expected_array_end")),
            }
        }
        Ok(JsonKind::Array(values.into_boxed_slice()))
    }

    fn string(&mut self) -> Result<Box<str>, ParseError> {
        if self.peek() != Some(b'"') {
            return Err(self.error("expected_string"));
        }
        self.advance();
        let start = self.pos;
        let mut offset = 0_usize;
        while let Some(&byte) = self.bytes.get(start.saturating_add(offset)) {
            match byte {
                b'"' => {
                    let end = start.saturating_add(offset);
                    self.pos = end.saturating_add(1);
                    let Some(slice) = self.bytes.get(start..end) else {
                        return Err(self.error("invalid_utf8"));
                    };
                    let Ok(text) = core::str::from_utf8(slice) else {
                        return Err(self.error("invalid_utf8"));
                    };
                    return Ok(Box::from(text));
                }
                b'\\' => {
                    self.pos = start;
                    return self.string_escaped();
                }
                0x00..=0x1f => {
                    self.pos = start.saturating_add(offset);
                    return Err(self.error("control_character"));
                }
                _ => offset = offset.saturating_add(1),
            }
        }
        Err(self.error("unterminated_string"))
    }

    fn string_escaped(&mut self) -> Result<Box<str>, ParseError> {
        let mut out = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.error("unterminated_string"));
            };
            match byte {
                b'"' => {
                    self.advance();
                    return Ok(out.into_boxed_str());
                }
                b'\\' => {
                    self.advance();
                    escape(self, &mut out)?;
                }
                0x00..=0x1f => return Err(self.error("control_character")),
                _ => {
                    let start = self.pos;
                    self.advance_utf8()?;
                    let Some(slice) = self.bytes.get(start..self.pos) else {
                        return Err(self.error("invalid_utf8"));
                    };
                    let Ok(text) = core::str::from_utf8(slice) else {
                        return Err(self.error("invalid_utf8"));
                    };
                    out.push_str(text);
                }
            }
        }
    }

    fn number(&mut self) -> Result<i64, ParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.advance();
        }
        let digits_start = self.pos;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.advance();
        }
        if self.pos == digits_start {
            return Err(self.error("invalid_number"));
        }
        if self.peek() == Some(b'.') || self.peek() == Some(b'e') || self.peek() == Some(b'E') {
            return Err(self.error("float_not_allowed"));
        }
        let Some(slice) = self.bytes.get(start..self.pos) else {
            return Err(self.error("invalid_number"));
        };
        let Ok(text) = core::str::from_utf8(slice) else {
            return Err(self.error("invalid_number"));
        };
        text.parse::<i64>()
            .map_err(|_| self.error("integer_out_of_range"))
    }

    fn literal(
        &mut self,
        expected: &[u8],
        kind: JsonKind,
        start: u32,
    ) -> Result<JsonValue, ParseError> {
        for expected_byte in expected {
            if self.peek() != Some(*expected_byte) {
                return Err(self.error("invalid_literal"));
            }
            self.advance();
        }
        Ok(JsonValue::new(kind, start))
    }

    fn advance_utf8(&mut self) -> Result<(), ParseError> {
        let Some(byte) = self.peek() else {
            return Err(self.error("unexpected_end"));
        };
        let width = if byte < 0x80 {
            1
        } else if byte < 0xe0 {
            2
        } else if byte < 0xf0 {
            3
        } else if byte < 0xf8 {
            4
        } else {
            return Err(self.error("invalid_utf8"));
        };
        for _ in 1..width {
            self.advance();
        }
        self.advance();
        Ok(())
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.advance();
        }
    }

    pub(crate) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub(crate) fn advance(&mut self) {
        self.pos = self.pos.saturating_add(1);
    }

    fn offset(&self) -> u32 {
        u32::try_from(self.pos).unwrap_or(u32::MAX)
    }

    pub(crate) fn error(&self, code: &str) -> ParseError {
        ParseError::new(code, self.offset())
    }
}
