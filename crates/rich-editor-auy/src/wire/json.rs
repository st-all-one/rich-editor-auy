//! Small, strict JSON parser used by the wire decoder.
#![allow(
    clippy::redundant_pub_crate,
    reason = "JSON machinery is crate-internal"
)]

use alloc::boxed::Box;

/// Ordered object members with unique keys.
pub(crate) type JsonObject = Box<[(Box<str>, JsonValue)]>;

/// Parsed JSON value. The wire decoder converts this into canonical model values.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum JsonValue {
    /// Null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Signed integer; floats and exponents are rejected.
    Int(i64),
    /// String.
    Str(Box<str>),
    /// Array.
    Array(Box<[Self]>),
    /// Object with unique keys, preserving input order.
    Object(JsonObject),
}

impl JsonValue {
    /// Returns an object member by key.
    #[must_use]
    pub(crate) fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(members) => members
                .iter()
                .find(|(name, _)| &**name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// Returns a string member.
    #[must_use]
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value),
            _ => None,
        }
    }

    /// Returns an integer member.
    #[must_use]
    pub(crate) fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns an array member.
    #[must_use]
    pub(crate) fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }
}
