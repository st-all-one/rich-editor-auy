//! Chunk directory boundaries.

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

fn document(count: usize) -> Option<Doc> {
    let schema = Schema::basic().ok()?;
    let paragraph = schema.node_kind("paragraph")?;
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        let text = TextBlock::new(vec![InlineItem::Text(Run::new(
            String::from("x"),
            MarkSet::empty(),
        ))]);
        blocks.push(Arc::new(Block::text(paragraph, Attrs::empty(), text)));
    }
    Doc::from_blocks(Attrs::empty(), blocks).ok()
}

#[test]
fn block_lookup_and_edits_cross_chunk_boundaries() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let Some(doc) = document(300) else {
        return;
    };
    assert_eq!(doc.blocks().len(), 300);
    assert_eq!(doc.blocks().iter().count(), 300);
    for index in [0_u32, 127, 128, 129, 255, 299] {
        let Some(range) = doc.block_range(index) else {
            continue;
        };
        assert_eq!(
            doc.block_at(range.start).map(|(found, _)| found),
            usize::try_from(index).ok()
        );
        assert_eq!(
            doc.block_at(Pos(range.end.0 - 1)).map(|(found, _)| found),
            usize::try_from(index).ok()
        );
    }
    assert_eq!(doc.blocks().get(128).map(|block| block.size().0), Some(3));
    let state = EditorState::from_doc(Arc::new(schema), doc);
    let mut transaction = state.transaction();
    assert!(transaction.insert_text(Pos(385), "y").is_ok());
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "insert must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(applied.state.doc().blocks().len(), 300);
    assert_eq!(
        applied
            .state
            .doc()
            .block_range(129)
            .map(|range| range.start),
        Some(Pos(388))
    );
}
