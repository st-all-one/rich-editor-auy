//! Parser for the content-expression grammar.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::error::SchemaError;

/// Parsed content expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Expr {
    /// Empty sequence.
    Empty,
    /// Named node or group.
    Name(Box<str>),
    /// Ordered sequence.
    Seq(Vec<Self>),
    /// Ordered alternatives.
    Alt(Vec<Self>),
    /// Repetition with inclusive bounds; `None` means unbounded.
    Repeat {
        /// Repeated expression.
        expr: Box<Self>,
        /// Minimum repetitions.
        min: u32,
        /// Maximum repetitions.
        max: Option<u32>,
    },
}

/// Parses a content expression.
pub(super) fn parse(input: &str) -> Result<Expr, SchemaError> {
    let mut parser = Parser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    let expression = parser.parse_alt()?;
    parser.skip_space();
    if parser.pos < parser.bytes.len() {
        return Err(invalid(input, parser.pos));
    }
    Ok(expression)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn parse_alt(&mut self) -> Result<Expr, SchemaError> {
        let mut alternatives = Vec::from([self.parse_seq()?]);
        loop {
            self.skip_space();
            if self.peek() == Some(b'|') {
                self.pos = self.pos.saturating_add(1);
                alternatives.push(self.parse_seq()?);
            } else {
                break;
            }
        }
        if alternatives.len() == 1 {
            return alternatives.pop().ok_or_else(|| invalid("", self.pos));
        }
        Ok(Expr::Alt(alternatives))
    }

    fn parse_seq(&mut self) -> Result<Expr, SchemaError> {
        let mut terms = Vec::new();
        loop {
            self.skip_space();
            match self.peek() {
                None | Some(b'|' | b')') => break,
                _ => terms.push(self.parse_term()?),
            }
        }
        match terms.len() {
            0 => Ok(Expr::Empty),
            1 => terms.pop().ok_or_else(|| invalid("", self.pos)),
            _ => Ok(Expr::Seq(terms)),
        }
    }

    fn parse_term(&mut self) -> Result<Expr, SchemaError> {
        let atom = self.parse_atom()?;
        self.skip_space();
        let Some(byte) = self.peek() else {
            return Ok(atom);
        };
        match byte {
            b'*' => {
                self.pos = self.pos.saturating_add(1);
                Ok(Expr::Repeat {
                    expr: Box::new(atom),
                    min: 0,
                    max: None,
                })
            }
            b'+' => {
                self.pos = self.pos.saturating_add(1);
                Ok(Expr::Repeat {
                    expr: Box::new(atom),
                    min: 1,
                    max: None,
                })
            }
            b'?' => {
                self.pos = self.pos.saturating_add(1);
                Ok(Expr::Repeat {
                    expr: Box::new(atom),
                    min: 0,
                    max: Some(1),
                })
            }
            b'{' => self.parse_bounds(atom),
            _ => Ok(atom),
        }
    }

    fn parse_bounds(&mut self, atom: Expr) -> Result<Expr, SchemaError> {
        self.pos = self.pos.saturating_add(1);
        self.skip_space();
        let min = self.parse_number()?;
        self.skip_space();
        let max = match self.peek() {
            Some(b'}') => Some(min),
            Some(b',') => {
                self.pos = self.pos.saturating_add(1);
                self.skip_space();
                if self.peek() == Some(b'}') {
                    None
                } else {
                    Some(self.parse_number()?)
                }
            }
            _ => return Err(invalid("", self.pos)),
        };
        self.skip_space();
        if self.peek() != Some(b'}') {
            return Err(invalid("", self.pos));
        }
        self.pos = self.pos.saturating_add(1);
        if let Some(upper) = max
            && upper < min
        {
            return Err(invalid("", self.pos));
        }
        Ok(Expr::Repeat {
            expr: Box::new(atom),
            min,
            max,
        })
    }

    fn parse_atom(&mut self) -> Result<Expr, SchemaError> {
        self.skip_space();
        match self.peek() {
            Some(b'(') => {
                self.pos = self.pos.saturating_add(1);
                let expression = self.parse_alt()?;
                self.skip_space();
                if self.peek() != Some(b')') {
                    return Err(invalid("", self.pos));
                }
                self.pos = self.pos.saturating_add(1);
                Ok(expression)
            }
            Some(_) => self.parse_name(),
            None => Err(invalid("", self.pos)),
        }
    }

    fn parse_name(&mut self) -> Result<Expr, SchemaError> {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            if byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' {
                self.pos = self.pos.saturating_add(1);
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(invalid("", self.pos));
        }
        let Some(slice) = self.bytes.get(start..self.pos) else {
            return Err(invalid("", self.pos));
        };
        let Ok(name) = core::str::from_utf8(slice) else {
            return Err(invalid("", self.pos));
        };
        Ok(Expr::Name(Box::from(name)))
    }

    fn parse_number(&mut self) -> Result<u32, SchemaError> {
        let start = self.pos;
        let mut value = 0_u32;
        while let Some(byte) = self.peek() {
            if !byte.is_ascii_digit() {
                break;
            }
            let digit = u32::from(byte.saturating_sub(b'0'));
            value = value
                .checked_mul(10)
                .and_then(|current| current.checked_add(digit))
                .ok_or_else(|| invalid("", self.pos))?;
            self.pos = self.pos.saturating_add(1);
        }
        if self.pos == start {
            return Err(invalid("", self.pos));
        }
        Ok(value)
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.pos = self.pos.saturating_add(1);
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
}

fn invalid(expression: &str, offset: usize) -> SchemaError {
    SchemaError::InvalidContent {
        expression: Box::from(expression),
        offset: u32::try_from(offset).unwrap_or(u32::MAX),
    }
}
