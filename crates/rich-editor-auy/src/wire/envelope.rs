//! Durable envelope format.

use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{self, Write};

use crate::error::{EditError, ParseError};
use crate::model::{Doc, DocId};
use crate::schema::Schema;

use super::json::JsonValue;
use super::json_parser::parse_json;
use super::options::ParseOptions;
use super::parser::decode_doc;
use super::writer::{write_doc_into, write_string};

/// A durable envelope around a canonical document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// Publication document.
    pub doc: Doc,
    /// Optional publication identifier.
    pub doc_id: Option<DocId>,
}

impl Envelope {
    /// Creates an envelope for a document.
    #[must_use]
    pub fn new(doc: Doc) -> Self {
        Self { doc, doc_id: None }
    }

    /// Sets the publication identifier.
    #[must_use]
    pub fn with_doc_id(mut self, doc_id: DocId) -> Self {
        self.doc_id = Some(doc_id);
        self
    }

    /// Serializes the envelope.
    ///
    /// # Errors
    ///
    /// Propagates formatting failures.
    pub fn to_json_into(&self, schema: &Schema, out: &mut impl Write) -> fmt::Result {
        out.write_str("{\"format\":\"rich-editor-auy\",\"version\":1,\"schema_fingerprint\":")?;
        write_string(schema.fingerprint(), out)?;
        if let Some(doc_id) = &self.doc_id {
            out.write_str(",\"doc_id\":")?;
            write_string(doc_id.as_str(), out)?;
        }
        out.write_str(",\"doc\":")?;
        write_doc_into(&self.doc, schema, out)?;
        out.write_char('}')
    }

    /// Decodes an envelope and verifies its schema fingerprint.
    ///
    /// # Errors
    ///
    /// Returns a parse or schema-mismatch error.
    pub fn from_json(
        input: &str,
        schema: &Schema,
        options: &ParseOptions,
    ) -> Result<Self, EditError> {
        let value = parse_json(input, options.limits.max_depth).map_err(EditError::Parse)?;
        let format = value
            .get("format")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| EditError::Parse(ParseError::new("missing_format", 0)))?;
        if format != "rich-editor-auy" {
            return Err(EditError::Parse(ParseError::new("unknown_format", 0)));
        }
        let version = value
            .get("version")
            .and_then(JsonValue::as_i64)
            .ok_or_else(|| EditError::Parse(ParseError::new("missing_version", 0)))?;
        if version != 1 {
            return Err(EditError::Parse(ParseError::new("unsupported_version", 0)));
        }
        let fingerprint = value
            .get("schema_fingerprint")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| EditError::Parse(ParseError::new("missing_fingerprint", 0)))?;
        if fingerprint != schema.fingerprint() {
            return Err(EditError::Step(crate::error::StepError::ContentMismatch {
                path: Box::from("schema_fingerprint"),
            }));
        }
        let doc_value = value
            .get("doc")
            .ok_or_else(|| EditError::Parse(ParseError::new("missing_doc", 0)))?;
        let doc = decode_doc(doc_value, schema, options)?;
        let doc_id = value
            .get("doc_id")
            .and_then(JsonValue::as_str)
            .and_then(DocId::new);
        Ok(Self { doc, doc_id })
    }

    /// Serializes the envelope to a string.
    ///
    /// # Errors
    ///
    /// Propagates formatting failures.
    pub fn to_json_string(&self, schema: &Schema) -> Result<String, fmt::Error> {
        let mut out = String::new();
        self.to_json_into(schema, &mut out)?;
        Ok(out)
    }

    /// Consumes the envelope and returns the document.
    #[must_use]
    pub fn into_doc(self) -> Doc {
        self.doc
    }
}
