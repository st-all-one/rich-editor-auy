//! Shared test strategies and helpers.
//!
//! This crate is internal and is not published.

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, InlineItem, MarkSet, NodeKind, Run, TextBlock};
use rich_editor_auy::schema::Schema;

/// Returns the testkit version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Builds one plain paragraph for a schema, when the schema declares `paragraph`.
#[must_use]
pub fn paragraph(schema: &Schema, text: &str) -> Option<Arc<Block>> {
    let kind = schema.node_kind("paragraph")?;
    let mut content = TextBlock::empty();
    content.push(InlineItem::Text(Run::new(
        String::from(text),
        MarkSet::empty(),
    )));
    Some(Arc::new(Block::text(kind, Attrs::empty(), content)))
}

/// Returns the paragraph kind, for callers that need to assert schema presence.
#[must_use]
pub fn paragraph_kind(schema: &Schema) -> Option<NodeKind> {
    schema.node_kind("paragraph")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rich_editor_auy::model::Doc;

    #[test]
    fn builds_a_paragraph_fixture() {
        let schema = Schema::basic();
        assert!(schema.is_ok(), "basic schema must compile");
        let Ok(schema) = schema else {
            return;
        };
        let paragraph = paragraph(&schema, "hello");
        assert!(paragraph.is_some(), "paragraph fixture must exist");
        let Some(paragraph) = paragraph else {
            return;
        };
        let doc = Doc::from_blocks(Attrs::empty(), vec![paragraph]);
        assert!(doc.is_ok(), "document must build");
        let Ok(doc) = doc else {
            return;
        };
        assert_eq!(doc.plain_text(), "hello");
        assert!(paragraph_kind(&schema).is_some());
    }
}
