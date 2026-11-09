//! Canonical JSON wire format and envelope.

mod attrs;
mod budget;
mod envelope;
mod fragment;
pub(crate) mod json;
mod json_escape;
mod json_hex;
pub(crate) mod json_parser;
mod marks;
mod options;
mod parser;
mod slice;
mod view;
mod writer;

pub use envelope::{ENVELOPE_VERSION, Envelope};
pub use fragment::from_fragment_json;
pub use options::ParseOptions;
pub use parser::check_json;
pub use parser::from_json;
pub use slice::from_slice_json;
pub use view::{view_json, write_view_into};
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
        let capacity = usize::try_from(self.size().0)
            .unwrap_or(0)
            .saturating_add(self.blocks().len().saturating_mul(32))
            .saturating_add(64);
        let mut out = alloc::string::String::with_capacity(capacity);
        if self.to_json_into(schema, &mut out).is_err() {
            return alloc::string::String::new();
        }
        out
    }
}
