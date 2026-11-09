//! Canonical slice JSON.
//!
//! A structural slice is written exactly like a fragment (`{"blocks":[...]}`). An inline slice is
//! `{"text":"...","runs":[...]}`; `runs` is present only when the slice carries marks or atoms.
//! Decoding reuses the document decoders, so the accepted shapes are the document shapes.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::error::ParseError;
use crate::model::TextBlock;
use crate::schema::Schema;
use crate::step::Slice;

use super::fragment::from_fragment_json;
use super::json::{JsonKind, JsonValue};
use super::json_parser::parse_json;
use super::options::ParseOptions;
use super::parser::decode_inline;
use super::writer::{write_fragment_body_into, write_items, write_string};

impl Slice {
    /// Writes the slice as canonical JSON into a sink.
    ///
    /// # Errors
    ///
    /// Propagates formatting failures from the sink and rejects blocks whose kind or attributes
    /// are absent from the schema.
    pub fn to_json_into(
        &self,
        schema: &Schema,
        out: &mut impl core::fmt::Write,
    ) -> core::fmt::Result {
        write_slice_into(self, schema, out)
    }

    /// Returns the canonical slice JSON.
    ///
    /// The writer for a `String` is infallible; the fallback keeps the signature panic-free and
    /// returns an empty string when the sink reports an error.
    #[must_use]
    pub fn to_json_string(&self, schema: &Schema) -> String {
        let mut out = String::new();
        if self.to_json_into(schema, &mut out).is_err() {
            return String::new();
        }
        out
    }
}

/// Writes a slice as canonical JSON.
///
/// # Errors
///
/// Propagates formatting failures from the sink and rejects unknown node kinds.
pub(super) fn write_slice_into(
    slice: &Slice,
    schema: &Schema,
    out: &mut impl core::fmt::Write,
) -> core::fmt::Result {
    if !slice.content.is_empty() {
        out.write_str("{\"blocks\":")?;
        write_fragment_body_into(&slice.content, schema, out)?;
        write_open_bounds(slice, out)?;
        return out.write_char('}');
    }
    out.write_str("{\"text\":")?;
    write_string(&slice.text, out)?;
    if let Some(runs) = &slice.runs {
        out.write_str(",\"runs\":")?;
        write_items(runs.items(), schema, out)?;
    }
    write_open_bounds(slice, out)?;
    out.write_char('}')
}

/// Writes the open-boundary members when the slice keeps an edge block open.
///
/// # Errors
///
/// Propagates formatting failures from the sink.
fn write_open_bounds(slice: &Slice, out: &mut impl core::fmt::Write) -> core::fmt::Result {
    if slice.open_start > 0 {
        write!(out, ",\"openStart\":{}", slice.open_start)?;
    }
    if slice.open_end > 0 {
        write!(out, ",\"openEnd\":{}", slice.open_end)?;
    }
    Ok(())
}

/// Decodes a slice from canonical JSON.
///
/// # Errors
///
/// Returns a located parse error when the input is not an object, when neither `blocks` nor
/// `text` is usable, when a run is invalid against the schema, or when `runs` disagree with
/// `text`.
pub fn from_slice_json(
    input: &str,
    schema: &Schema,
    options: &ParseOptions,
) -> Result<Slice, ParseError> {
    let value = parse_json(input, options.limits.max_depth)?;
    if !matches!(value.kind(), JsonKind::Object(_)) {
        return Err(ParseError {
            code: Box::from("slice_not_object"),
            offset: value.start(),
            path: Box::from("slice"),
        });
    }
    let open_start = open_depth(&value, "openStart")?;
    let open_end = open_depth(&value, "openEnd")?;
    if value.get("blocks").is_some() {
        let mut slice = Slice::from_fragment(from_fragment_json(input, schema, options)?);
        slice.open_start = open_start;
        slice.open_end = open_end;
        return Ok(slice);
    }
    if open_start > 0 || open_end > 0 {
        return Err(ParseError {
            code: Box::from("invalid_open"),
            offset: value.start(),
            path: Box::from("slice"),
        });
    }
    decode_text_slice(&value, schema, options)
}

/// Decodes an inline slice from an object that carries `text` and optional `runs`.
///
/// # Errors
///
/// Returns a located parse error when `text` is absent or not a string, when the text exceeds
/// the configured limit, when a run is invalid, or when `runs` disagree with `text`.
fn decode_text_slice(
    value: &JsonValue,
    schema: &Schema,
    options: &ParseOptions,
) -> Result<Slice, ParseError> {
    let text = value.get("text").ok_or_else(|| ParseError {
        code: Box::from("missing_text"),
        offset: value.start(),
        path: Box::from("slice"),
    })?;
    let text = text.as_str().ok_or_else(|| ParseError {
        code: Box::from("text_not_string"),
        offset: text.start(),
        path: Box::from("slice.text"),
    })?;
    if u32::try_from(text.len()).unwrap_or(u32::MAX) > options.limits.max_block_text_bytes {
        return Err(ParseError {
            code: Box::from("limit_exceeded"),
            offset: value.start(),
            path: Box::from("slice.text"),
        });
    }
    let Some(runs) = value.get("runs") else {
        return Ok(Slice::plain_text(text));
    };
    let values = runs.as_array().ok_or_else(|| ParseError {
        code: Box::from("runs_not_array"),
        offset: runs.start(),
        path: Box::from("slice.runs"),
    })?;
    let mut items = Vec::with_capacity(values.len());
    let mut path: Vec<u32> = Vec::with_capacity(2);
    for (index, item) in values.iter().enumerate() {
        path.clear();
        path.push(1);
        path.push(u32::try_from(index).unwrap_or(u32::MAX));
        items.push(decode_inline(item, schema, &path, &options.limits)?);
    }
    let block = TextBlock::new(items);
    if !text.is_empty() && block.len().0 != u32::try_from(text.len()).unwrap_or(u32::MAX) {
        return Err(ParseError {
            code: Box::from("slice_text_mismatch"),
            offset: value.start(),
            path: Box::from("slice.runs"),
        });
    }
    Ok(Slice::from_textblock(block))
}

/// Reads an optional open-boundary depth, limited to the one level the model represents.
///
/// # Errors
///
/// Returns a located parse error when the value is not an integer in `0..=1`.
fn open_depth(value: &JsonValue, key: &'static str) -> Result<u8, ParseError> {
    let Some(node) = value.get(key) else {
        return Ok(0);
    };
    match node.as_i64() {
        Some(0) => Ok(0),
        Some(1) => Ok(1),
        _ => Err(ParseError {
            code: Box::from("invalid_open"),
            offset: node.start(),
            path: Box::from(key),
        }),
    }
}
