//! Complexity gate: local lookups must not scan the document.
#![cfg(feature = "visits")]

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;
use rich_editor_auy::step::take_locate_visits;

#[test]
fn inserting_at_the_end_uses_the_block_index() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let schema = Arc::new(schema);
    let paragraph = schema.node_kind("paragraph");
    assert!(paragraph.is_some(), "paragraph must exist");
    let Some(paragraph) = paragraph else {
        return;
    };
    let mut blocks = Vec::new();
    for _ in 0..10_000 {
        let text = TextBlock::new(vec![InlineItem::Text(Run::new(
            String::from("x"),
            MarkSet::empty(),
        ))]);
        blocks.push(Arc::new(Block::text(paragraph, Attrs::empty(), text)));
    }
    let doc = Doc::from_blocks(Attrs::empty(), blocks);
    assert!(doc.is_ok(), "large document must build");
    let Ok(doc) = doc else {
        return;
    };
    let end = Pos(doc.size().0.saturating_sub(1));
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    assert!(transaction.insert_text(end, "y").is_ok());
    take_locate_visits();
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "insert must apply");
    assert!(
        take_locate_visits() <= 4,
        "a local lookup must not scan every block"
    );
}
