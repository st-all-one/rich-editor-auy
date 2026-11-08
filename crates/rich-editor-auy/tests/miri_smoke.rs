//! Small deterministic target for Miri.

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::schema::Schema;

#[test]
fn model_survives_miri() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let paragraph = schema.node_kind("paragraph");
    assert!(paragraph.is_some(), "paragraph must exist");
    let Some(paragraph) = paragraph else {
        return;
    };
    let text = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from("miri"),
        MarkSet::empty(),
    ))]);
    let doc = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(paragraph, Attrs::empty(), text))],
    );
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    assert_eq!(doc.plain_text(), "miri");
    assert!(doc.to_json_string(&schema).contains("miri"));
}
