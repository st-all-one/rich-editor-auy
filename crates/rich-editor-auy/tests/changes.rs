//! Change set block reporting tests.

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

fn state_with(texts: &[&str]) -> Option<(Arc<Schema>, EditorState)> {
    let schema = Arc::new(Schema::full().ok()?);
    let kind = schema.node_kind("paragraph")?;
    let blocks = texts
        .iter()
        .map(|text| {
            Arc::new(Block::text(
                kind,
                Attrs::empty(),
                TextBlock::new(vec![InlineItem::Text(Run::new(
                    String::from(*text),
                    MarkSet::empty(),
                ))]),
            ))
        })
        .collect();
    let doc = Doc::from_blocks(Attrs::empty(), blocks).ok()?;
    Some((Arc::clone(&schema), EditorState::from_doc(schema, doc)))
}

#[test]
fn insert_inside_one_block_reports_only_that_block() {
    let Some((_schema, state)) = state_with(&["one", "two", "three"]) else {
        return;
    };
    let Some(range) = state.doc().block_range(1) else {
        return;
    };
    let at = Pos(range.start.0.saturating_add(1));
    let mut transaction = state.transaction();
    assert!(transaction.insert_text(at, "x").is_ok());
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert!(!applied.changes.full);
    assert_eq!(applied.changes.blocks, 1..2);
    assert!(applied.changes.inserted_blocks.is_empty());
    assert!(applied.changes.removed_blocks.is_empty());
}

#[test]
fn split_reports_the_inserted_block() {
    let Some((_schema, state)) = state_with(&["hello", "world"]) else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(transaction.split_block(Pos(3)).is_ok());
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert!(!applied.changes.full);
    assert_eq!(applied.changes.blocks, 0..2);
    assert_eq!(&*applied.changes.inserted_blocks, &[1]);
    assert!(applied.changes.removed_blocks.is_empty());
    assert_eq!(applied.state.doc().blocks().len(), 3);
}

#[test]
fn delete_across_blocks_reports_the_removed_block() {
    let Some((_schema, state)) = state_with(&["one", "two", "three"]) else {
        return;
    };
    let Some(first) = state.doc().block_range(0) else {
        return;
    };
    let Some(second) = state.doc().block_range(1) else {
        return;
    };
    let from = Pos(first.start.0.saturating_add(1));
    let to = Pos(second.start.0.saturating_add(1));
    let mut transaction = state.transaction();
    assert!(transaction.delete(Range::new(from, to)).is_ok());
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert!(!applied.changes.full);
    assert_eq!(applied.changes.blocks, 0..2);
    assert!(applied.changes.inserted_blocks.is_empty());
    assert_eq!(&*applied.changes.removed_blocks, &[1]);
    assert_eq!(applied.state.doc().blocks().len(), 2);
}

#[test]
fn replace_document_reports_a_full_redraw() {
    let Some((_schema, state)) = state_with(&["one", "two"]) else {
        return;
    };
    let Some(replacement) = state_with(&["fresh"]) else {
        return;
    };
    let Some(doc) = Doc::from_blocks(Attrs::empty(), replacement.1.doc().blocks().to_vec()).ok()
    else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(transaction.replace_document(doc).is_ok());
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert!(applied.changes.full);
    assert_eq!(applied.changes.blocks, 0..1);
    assert!(applied.changes.inserted_blocks.is_empty());
    assert!(applied.changes.removed_blocks.is_empty());
}

#[test]
fn undo_of_a_split_reports_the_removed_block() {
    let Some((_schema, state)) = state_with(&["hello", "world"]) else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(transaction.split_block(Pos(3)).is_ok());
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    let Some(undone) = applied.state.undo().ok().flatten() else {
        return;
    };
    assert!(!undone.changes.full);
    assert_eq!(&*undone.changes.removed_blocks, &[1]);
    assert_eq!(undone.state.doc().blocks().len(), 2);
}
