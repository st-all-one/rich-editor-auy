//! Attribute and mark decoders.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic member decoding"
)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "wire decoders are crate-internal"
)]

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::error::ParseError;
use crate::model::{Attr, AttrValue, Attrs, DocId, Mark, MarkSet, NodeId, RefTarget, VersionTag};
use crate::pos::{Pos, Range};
use crate::schema::{AttrSpec, AttrType, Schema};

use super::json::{JsonKind, JsonValue};

/// Error path for attribute decoding.
#[derive(Clone, Copy)]
pub(crate) enum DecodePath<'a> {
    DocAttrs,
    Node(&'a [u32]),
}

impl DecodePath<'_> {
    fn boxed(&self) -> Box<str> {
        match self {
            Self::DocAttrs => Box::from("doc.attrs"),
            Self::Node(path) => {
                let mut out = alloc::string::String::new();
                for (index, component) in path.iter().enumerate() {
                    if index > 0 {
                        out.push('.');
                    }
                    out.push_str(&component.to_string());
                }
                out.into_boxed_str()
            }
        }
    }
}

pub(crate) fn decode_attrs(
    value: Option<&JsonValue>,
    specs: &[AttrSpec],
    path: DecodePath<'_>,
) -> Result<Attrs, ParseError> {
    let Some(value) = value else {
        return Ok(Attrs::empty());
    };
    let JsonKind::Object(members) = value.kind() else {
        return Err(ParseError {
            code: Box::from("attrs_not_object"),
            offset: value.start(),
            path: path.boxed(),
        });
    };
    let mut attrs = Vec::new();
    for (name, value) in members {
        let Some(spec) = specs.iter().find(|spec| *spec.name == **name) else {
            return Err(ParseError {
                code: Box::from("unknown_attr"),
                offset: value.start(),
                path: path.boxed(),
            });
        };
        let decoded = decode_value(value, &spec.kind).map_err(|error| ParseError {
            offset: value.start(),
            ..error
        })?;
        if !Schema::value_matches(&spec.kind, &decoded) {
            return Err(ParseError {
                code: Box::from("attr_type"),
                offset: value.start(),
                path: path.boxed(),
            });
        }
        attrs.push(Attr {
            id: spec.id,
            value: decoded,
        });
    }
    for spec in specs {
        if spec.required && spec.default.is_none() && !attrs.iter().any(|attr| attr.id == spec.id) {
            return Err(ParseError {
                code: Box::from("required_attr"),
                offset: value.start(),
                path: path.boxed(),
            });
        }
    }
    Attrs::new(attrs).ok_or_else(|| ParseError {
        code: Box::from("duplicate_attr"),
        offset: value.start(),
        path: path.boxed(),
    })
}

fn decode_value(value: &JsonValue, kind: &AttrType) -> Result<AttrValue, ParseError> {
    match kind {
        AttrType::Str => value
            .as_str()
            .map(|text| AttrValue::Str(Box::from(text)))
            .ok_or_else(|| ParseError::new("expected_string", 0)),
        AttrType::Int { .. } => value
            .as_i64()
            .map(AttrValue::Int)
            .ok_or_else(|| ParseError::new("expected_int", 0)),
        AttrType::Bool => match value.kind() {
            JsonKind::Bool(flag) => Ok(AttrValue::Bool(*flag)),
            _ => Err(ParseError::new("expected_bool", 0)),
        },
        AttrType::Ref => decode_ref(value).map(AttrValue::Ref),
        AttrType::Any => decode_any(value),
    }
}

fn decode_ref(value: &JsonValue) -> Result<RefTarget, ParseError> {
    if !matches!(value.kind(), JsonKind::Object(_)) {
        return Err(ParseError::new("expected_ref", 0));
    }
    let doc = value
        .get("doc")
        .and_then(JsonValue::as_str)
        .and_then(DocId::new)
        .ok_or_else(|| ParseError::new("invalid_ref_doc", 0))?;
    let node = match value.get("node") {
        None => None,
        Some(raw) => Some(
            raw.as_str()
                .and_then(NodeId::new)
                .ok_or_else(|| ParseError::new("invalid_ref_node", 0))?,
        ),
    };
    let version = match value.get("version") {
        None => None,
        Some(raw) => Some(
            raw.as_str()
                .and_then(VersionTag::new)
                .ok_or_else(|| ParseError::new("invalid_ref_version", 0))?,
        ),
    };
    let fragment = match value.get("fragment") {
        None => None,
        Some(fragment) => {
            let start = fragment
                .get("start")
                .and_then(JsonValue::as_i64)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(|| ParseError::new("invalid_ref_fragment", 0))?;
            let end = fragment
                .get("end")
                .and_then(JsonValue::as_i64)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(|| ParseError::new("invalid_ref_fragment", 0))?;
            if start > end {
                return Err(ParseError::new("invalid_ref_fragment", 0));
            }
            Some(Range::new(Pos(start), Pos(end)))
        }
    };
    Ok(RefTarget {
        doc,
        node,
        version,
        fragment,
    })
}

fn decode_any(value: &JsonValue) -> Result<AttrValue, ParseError> {
    match value.kind() {
        JsonKind::Null => Ok(AttrValue::Null),
        JsonKind::Bool(flag) => Ok(AttrValue::Bool(*flag)),
        JsonKind::Int(number) => Ok(AttrValue::Int(*number)),
        JsonKind::Str(text) => Ok(AttrValue::Str(text.clone())),
        JsonKind::Array(values) => {
            let mut out = Vec::new();
            for value in values {
                out.push(decode_any(value)?);
            }
            Ok(AttrValue::Array(out.into_boxed_slice()))
        }
        JsonKind::Object(_) => decode_ref(value).map(AttrValue::Ref),
    }
}

pub(crate) fn decode_marks(
    value: Option<&JsonValue>,
    schema: &Schema,
) -> Result<MarkSet, ParseError> {
    let Some(value) = value else {
        return Ok(MarkSet::empty());
    };
    let values = value
        .as_array()
        .ok_or_else(|| ParseError::new("marks_not_array", value.start()))?;
    let mut marks = MarkSet::empty();
    for value in values {
        let name = value
            .get("type")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| ParseError::new("missing_mark_type", value.start()))?;
        let id = schema
            .mark_id(name)
            .ok_or_else(|| ParseError::new("unknown_mark", value.start()))?;
        let specs = schema.mark(id).map_or(&[][..], |spec| &spec.attrs);
        let nested = value.get("attrs");
        let mut decoded = Vec::new();
        for spec in specs {
            let raw = nested
                .and_then(|attrs| attrs.get(&spec.name))
                .or_else(|| value.get(&spec.name));
            if let Some(raw) = raw {
                let target = decode_value(raw, &spec.kind).map_err(|error| ParseError {
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
        let attrs = Attrs::new(decoded)
            .ok_or_else(|| ParseError::new("duplicate_mark_attr", value.start()))?;
        marks = marks.with_mark(Mark { id, attrs });
    }
    Ok(marks)
}
