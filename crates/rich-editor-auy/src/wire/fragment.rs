//! Canonical fragment JSON: a detached block sequence.
//!
//! A fragment is written as `{"blocks":[<block>,...]}`. Decoding reuses the document block
//! decoder and validates the blocks as the content of an attribute-less document, so the
//! accepted block set is exactly the document content set of the schema. Error paths are
//! array indices relative to `blocks` (for example `0.2.1`).

use alloc::boxed::Box;
use alloc::string::String;

use crate::error::ParseError;
use crate::model::{Attrs, Doc, Fragment};
use crate::schema::Schema;

use super::json::JsonKind;
use super::json_parser::parse_json;
use super::options::ParseOptions;
use super::parser::{check_doc, decode_blocks};
use super::writer::write_fragment_into;

/// Decodes a fragment from canonical JSON.
///
/// An empty `blocks` array decodes to an empty fragment.
///
/// # Errors
///
/// Returns a located parse error when the input is not a fragment object, when `blocks` is
/// missing or not an array, or when a block is invalid against the schema.
pub fn from_fragment_json(
    input: &str,
    schema: &Schema,
    options: &ParseOptions,
) -> Result<Fragment, ParseError> {
    let value = parse_json(input, options.limits.max_depth)?;
    if !matches!(value.kind(), JsonKind::Object(_)) {
        return Err(ParseError {
            code: Box::from("fragment_not_object"),
            offset: value.start(),
            path: Box::from("blocks"),
        });
    }
    let blocks = value.get("blocks").ok_or_else(|| ParseError {
        code: Box::from("missing_blocks"),
        offset: value.start(),
        path: Box::from("blocks"),
    })?;
    let values = blocks.as_array().ok_or_else(|| ParseError {
        code: Box::from("blocks_not_array"),
        offset: blocks.start(),
        path: Box::from("blocks"),
    })?;
    if values.is_empty() {
        return Ok(Fragment::default());
    }
    let decoded = decode_blocks(values, schema, options)?;
    let doc = Doc::new(Attrs::empty(), decoded, &options.limits).map_err(|_error| ParseError {
        code: Box::from("limit"),
        offset: value.start(),
        path: Box::from("blocks"),
    })?;
    let doc = check_doc(doc, schema, value.start())?;
    Ok(Fragment::new(doc.blocks().to_vec()))
}

impl Fragment {
    /// Writes the fragment as canonical JSON into a sink.
    ///
    /// # Errors
    ///
    /// Propagates formatting failures from the sink and rejects blocks whose kind or
    /// attributes are absent from the schema.
    pub fn to_json_into(
        &self,
        schema: &Schema,
        out: &mut impl core::fmt::Write,
    ) -> core::fmt::Result {
        write_fragment_into(self, schema, out)
    }

    /// Returns the canonical fragment JSON.
    ///
    /// The writer for a `String` is infallible; the fallback keeps the signature
    /// panic-free and returns an empty string when the sink reports an error.
    #[must_use]
    pub fn to_json_string(&self, schema: &Schema) -> String {
        let capacity = usize::try_from(self.size().0).unwrap_or(0);
        let mut out = String::with_capacity(capacity.saturating_add(64));
        if self.to_json_into(schema, &mut out).is_err() {
            return String::new();
        }
        out
    }
}
