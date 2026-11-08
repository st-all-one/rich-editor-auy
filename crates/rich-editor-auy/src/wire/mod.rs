//! Canonical JSON wire format and envelope.

mod attrs;
mod envelope;
pub(crate) mod json;
mod json_hex;
pub(crate) mod json_parser;
mod options;
mod parser;
mod writer;

pub use envelope::Envelope;
pub use options::ParseOptions;
pub use parser::from_json;
pub use writer::write_doc_into;
pub(crate) use writer::write_string;

use crate::model::Doc;
use crate::schema::Schema;

impl Doc {
    /// Writes the canonical document JSON into a sink.
    ///
    /// # Errors
    ///
    /// Propagates formatting failures from the sink.
    pub fn to_json_into(
        &self,
        schema: &Schema,
        out: &mut impl core::fmt::Write,
    ) -> core::fmt::Result {
        write_doc_into(self, schema, out)
    }

    /// Returns the canonical document JSON.
    ///
    /// The writer for a `String` is infallible; the fallback is unreachable and exists only to
    /// keep the public signature allocation-free and panic-free.
    #[must_use]
    pub fn to_json_string(&self, schema: &Schema) -> alloc::string::String {
        let mut out = alloc::string::String::new();
        if self.to_json_into(schema, &mut out).is_err() {
            return alloc::string::String::new();
        }
        out
    }
}
