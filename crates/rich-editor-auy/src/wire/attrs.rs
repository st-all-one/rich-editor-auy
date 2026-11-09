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

use crate::Limits;
use crate::error::ParseError;
use crate::model::{Attr, AttrValue, Attrs, DocId, NodeId, RefTarget, VersionTag};
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
    limits: &Limits,
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
    if members.len() > usize::from(limits.max_attrs_per_node) {
        return Err(ParseError {
            code: Box::from("limit"),
            offset: value.start(),
            path: path.boxed(),
        });
    }
    let mut attrs = Vec::with_capacity(members.len());
    for (name, member) in members {
        attrs.push(decode_member(member, name, specs, path, limits)?);
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

/// Decodes one declared attribute, checking its type, URL safety, and limits.
fn decode_member(
    value: &JsonValue,
    name: &str,
    specs: &[AttrSpec],
    path: DecodePath<'_>,
    limits: &Limits,
) -> Result<Attr, ParseError> {
    let Some(spec) = specs.iter().find(|spec| *spec.name == *name) else {
        return Err(ParseError {
            code: Box::from("unknown_attr"),
            offset: value.start(),
            path: path.boxed(),
        });
    };
    let decoded = decode_value(value, &spec.kind, limits, 0).map_err(|error| ParseError {
        offset: value.start(),
        ..error
    })?;
    if crate::schema::is_url_attr(&spec.name)
        && let AttrValue::Str(url) = &decoded
        && crate::schema::is_unsafe_url(url)
    {
        return Err(ParseError {
            code: Box::from("unsafe_url"),
            offset: value.start(),
            path: path.boxed(),
        });
    }
    if !Schema::value_matches(&spec.kind, &decoded) {
        return Err(ParseError {
            code: Box::from("attr_type"),
            offset: value.start(),
            path: path.boxed(),
        });
    }
    Ok(Attr {
        id: spec.id,
        value: decoded,
    })
}

/// Decodes one attribute value, rejecting oversized strings and over-deep nesting before
/// any `Box`/`Vec` is built.
pub(super) fn decode_value(
    value: &JsonValue,
    kind: &AttrType,
    limits: &Limits,
    depth: u8,
) -> Result<AttrValue, ParseError> {
    if u32::from(depth) > u32::from(limits.max_depth) {
        return Err(ParseError::new("limit", value.start()));
    }
    let next = depth.saturating_add(1);
    match kind {
        AttrType::Str | AttrType::Enum(_) => {
            let text = value
                .as_str()
                .ok_or_else(|| ParseError::new("expected_string", 0))?;
            check_string(text.len(), limits, value.start())?;
            Ok(AttrValue::Str(Box::from(text)))
        }
        AttrType::Int { .. } => value
            .as_i64()
            .map(AttrValue::Int)
            .ok_or_else(|| ParseError::new("expected_int", 0)),
        AttrType::Bool => match value.kind() {
            JsonKind::Bool(flag) => Ok(AttrValue::Bool(*flag)),
            _ => Err(ParseError::new("expected_bool", 0)),
        },
        AttrType::Ref => decode_ref(value).map(AttrValue::Ref),
        AttrType::List(inner) => {
            let JsonKind::Array(values) = value.kind() else {
                return Err(ParseError::new("expected_array", 0));
            };
            let mut items = Vec::with_capacity(values.len());
            for item in values {
                items.push(decode_value(item, inner, limits, next)?);
            }
            Ok(AttrValue::Array(items.into_boxed_slice()))
        }
        AttrType::Map(inner) => {
            let JsonKind::Object(members) = value.kind() else {
                return Err(ParseError::new("expected_map", 0));
            };
            let mut entries = Vec::with_capacity(members.len());
            for (name, member) in members {
                check_string(name.len(), limits, member.start())?;
                entries.push((name.clone(), decode_value(member, inner, limits, next)?));
            }
            AttrValue::map(entries).ok_or_else(|| ParseError::new("duplicate_map_key", 0))
        }
        AttrType::Any => decode_any(value, limits, next),
    }
}

/// Rejects an attribute string that exceeds the configured byte budget.
fn check_string(len: usize, limits: &Limits, offset: u32) -> Result<(), ParseError> {
    if u32::try_from(len).unwrap_or(u32::MAX) > limits.max_attr_string_bytes {
        return Err(ParseError::new("limit", offset));
    }
    Ok(())
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

fn decode_any(value: &JsonValue, limits: &Limits, depth: u8) -> Result<AttrValue, ParseError> {
    if u32::from(depth) > u32::from(limits.max_depth) {
        return Err(ParseError::new("limit", value.start()));
    }
    let next = depth.saturating_add(1);
    match value.kind() {
        JsonKind::Null => Ok(AttrValue::Null),
        JsonKind::Bool(flag) => Ok(AttrValue::Bool(*flag)),
        JsonKind::Int(number) => Ok(AttrValue::Int(*number)),
        JsonKind::Str(text) => {
            check_string(text.len(), limits, value.start())?;
            Ok(AttrValue::Str(text.clone()))
        }
        JsonKind::Array(values) => {
            let mut out = Vec::with_capacity(values.len());
            for value in values {
                out.push(decode_any(value, limits, next)?);
            }
            Ok(AttrValue::Array(out.into_boxed_slice()))
        }
        JsonKind::Object(members) => {
            if value.get("doc").is_some() {
                decode_ref(value).map(AttrValue::Ref)
            } else {
                let mut out = Vec::with_capacity(members.len());
                for (name, member) in members {
                    check_string(name.len(), limits, member.start())?;
                    out.push((name.clone(), decode_any(member, limits, next)?));
                }
                AttrValue::map(out).ok_or_else(|| ParseError::new("duplicate_map_key", 0))
            }
        }
    }
}
