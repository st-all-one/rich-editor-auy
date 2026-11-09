//! Mark decoder.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic member decoding"
)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "wire decoders are crate-internal"
)]

use alloc::vec::Vec;

use crate::Limits;
use crate::error::ParseError;
use crate::model::{Attr, Attrs, Mark, MarkSet};
use crate::schema::Schema;

use super::attrs::decode_value;
use super::json::JsonValue;

pub(crate) fn decode_marks(
    value: Option<&JsonValue>,
    schema: &Schema,
    limits: &Limits,
) -> Result<MarkSet, ParseError> {
    let Some(value) = value else {
        return Ok(MarkSet::empty());
    };
    let values = value
        .as_array()
        .ok_or_else(|| ParseError::new("marks_not_array", value.start()))?;
    if values.len() > usize::try_from(u64::BITS).unwrap_or(64) {
        return Err(ParseError::new("mark_limit", value.start()));
    }
    let mut ids = MarkSet::empty();
    let mut marks = Vec::with_capacity(values.len());
    for value in values {
        let name = value
            .get("type")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| ParseError::new("missing_mark_type", value.start()))?;
        let id = schema
            .mark_id(name)
            .ok_or_else(|| ParseError::new("unknown_mark", value.start()))?;
        if ids.contains(id) {
            return Err(ParseError::new("duplicate_mark", value.start()));
        }
        ids = ids.with_mark(Mark::new(id));
        let specs = schema.mark(id).map_or(&[][..], |spec| &spec.attrs);
        let nested = value.get("attrs");
        let mut decoded = Vec::new();
        for spec in specs {
            let raw = nested
                .and_then(|attrs| attrs.get(&spec.name))
                .or_else(|| value.get(&spec.name));
            if let Some(raw) = raw {
                let target =
                    decode_value(raw, &spec.kind, limits, 0).map_err(|error| ParseError {
                        offset: raw.start(),
                        ..error
                    })?;
                if !Schema::value_matches(&spec.kind, &target) {
                    return Err(ParseError::new("mark_attr_type", raw.start()));
                }
                decoded.push(Attr {
                    id: spec.id,
                    value: target,
                });
            } else if spec.required && spec.default.is_none() {
                return Err(ParseError::new("required_mark_attr", value.start()));
            }
        }
        if decoded.len() > usize::from(limits.max_mark_attrs) {
            return Err(ParseError::new("limit", value.start()));
        }
        let attrs = Attrs::new(decoded)
            .ok_or_else(|| ParseError::new("duplicate_mark_attr", value.start()))?;
        marks.push(Mark { id, attrs });
    }
    Ok(MarkSet::from_marks(marks))
}
