//! Complete public schema preset.

mod attrs;
mod marks;
mod nodes;

use crate::error::SchemaError;

use super::Schema;

impl Schema {
    /// Builds the complete editor schema.
    ///
    /// This preset is the complete contract for tokens accepted by the wire decoder and the
    /// ingestion boundary. Every node, mark, atom, and attribute must be declared here;
    /// there is no generic escape hatch.
    ///
    /// # Errors
    ///
    /// Returns a schema error only if the preset itself is inconsistent.
    #[allow(clippy::too_many_lines, reason = "one readable canonical preset")]
    pub fn full() -> Result<Self, SchemaError> {
        Self::builder()
            .doc(attrs::doc())
            .node(nodes::text())
            .node(nodes::paragraph())
            .node(nodes::heading())
            .node(nodes::bullet_list())
            .node(nodes::ordered_list())
            .node(nodes::list_item())
            .node(nodes::blockquote())
            .node(nodes::code_block())
            .node(nodes::raw_code())
            .node(nodes::table())
            .node(nodes::table_caption())
            .node(nodes::table_row())
            .node(nodes::table_header())
            .node(nodes::table_cell())
            .node(nodes::figure())
            .node(nodes::caption())
            .node(nodes::separator())
            .node(nodes::image())
            .node(nodes::image_block())
            .node(nodes::video())
            .node(nodes::audio())
            .node(nodes::attachment())
            .node(nodes::hard_break())
            .mark(marks::strong())
            .mark(marks::em())
            .mark(marks::code())
            .mark(marks::underline())
            .mark(marks::strike())
            .mark(marks::highlight())
            .mark(marks::sup())
            .mark(marks::sub())
            .mark(marks::link())
            .mark(marks::reference())
            .build()
    }
}
