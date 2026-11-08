//! Editing, mapping, and history smoke tests.

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, Mark, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{EditorState, Selection};

#[test]
fn insert_undo_and_redo_round_trip() {
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
    let block = Block::text(paragraph, Attrs::empty(), TextBlock::empty());
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]);
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let initial = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = initial.transaction();
    let inserted = transaction.insert_text(Pos(1), "abc");
    assert!(inserted.is_ok(), "insert must be queued");
    transaction.set_selection(Selection::caret(Pos(4)));
    let applied = initial.apply(transaction);
    assert!(applied.is_ok(), "transaction must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "abc");
    assert_eq!(
        applied.state.doc().to_json_string(&schema),
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abc"}]}]}"#
    );
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(undone.state.doc().plain_text(), "");
    let redone = undone.state.redo();
    assert!(redone.is_ok(), "redo must apply");
    let Ok(Some(redone)) = redone else {
        return;
    };
    assert_eq!(redone.state.doc().plain_text(), "abc");
}

#[test]
#[allow(clippy::too_many_lines, reason = "single end-to-end scenario")]
fn join_and_split_round_trip_through_undo() {
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
    let first = Block::text(
        paragraph,
        Attrs::empty(),
        TextBlock::new(vec![InlineItem::Text(Run::new(
            String::from("a"),
            MarkSet::empty(),
        ))]),
    );
    let second = Block::text(
        paragraph,
        Attrs::empty(),
        TextBlock::new(vec![InlineItem::Text(Run::new(
            String::from("b"),
            MarkSet::empty(),
        ))]),
    );
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(first), Arc::new(second)]);
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let boundary = state.doc().block_range(1).map(|range| range.start);
    assert_eq!(boundary, Some(Pos(3)));
    let Some(boundary) = boundary else {
        return;
    };
    let mut join = state.transaction();
    assert!(join.join_backward(boundary).is_ok());
    let joined = state.apply(join);
    assert!(joined.is_ok(), "join must apply");
    let Ok(joined) = joined else {
        return;
    };
    assert_eq!(joined.state.doc().blocks().len(), 1);
    assert_eq!(joined.state.doc().plain_text(), "ab");
    let undone = joined.state.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(undone.state.doc().blocks().len(), 2);
    let redone = undone.state.redo();
    assert!(redone.is_ok(), "redo must apply");
    let Ok(Some(redone)) = redone else {
        return;
    };
    assert_eq!(redone.state.doc().blocks().len(), 1);

    let mut split = redone.state.transaction();
    assert!(split.split_block(Pos(2)).is_ok());
    let split = redone.state.apply(split);
    assert!(split.is_ok(), "split must apply");
    let Ok(split) = split else {
        return;
    };
    assert_eq!(split.state.doc().blocks().len(), 2);
    assert_eq!(split.state.doc().plain_text(), "ab");
    let undone = split.state.undo();
    assert!(undone.is_ok(), "undo split must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(undone.state.doc().blocks().len(), 1);
    assert_eq!(undone.state.doc().plain_text(), "ab");
}

#[test]
fn undo_restores_marks_removed_by_delete() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let schema = Arc::new(schema);
    let paragraph = schema.node_kind("paragraph");
    let strong = schema.mark_id("strong");
    assert!(
        paragraph.is_some() && strong.is_some(),
        "fixtures must exist"
    );
    let (Some(paragraph), Some(strong)) = (paragraph, strong) else {
        return;
    };
    let marks = MarkSet::single(Mark::new(strong));
    let block = Block::text(
        paragraph,
        Attrs::empty(),
        TextBlock::new(vec![InlineItem::Text(Run::new(String::from("abc"), marks))]),
    );
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]);
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let original = state.doc().to_json_string(&schema);
    assert!(original.contains("\"strong\""));
    let mut transaction = state.transaction();
    assert!(transaction.delete(Range::new(Pos(1), Pos(4))).is_ok());
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "delete must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "");
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(undone.state.doc().to_json_string(&schema), original);
}

#[test]
fn multi_step_undo_restores_original_text() {
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
    let block = Block::text(
        paragraph,
        Attrs::empty(),
        TextBlock::new(vec![InlineItem::Text(Run::new(
            String::from("abcd"),
            MarkSet::empty(),
        ))]),
    );
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]);
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    assert!(transaction.insert_text(Pos(1), "A").is_ok());
    assert!(transaction.insert_text(Pos(1), "B").is_ok());
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "multi-step apply must work");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "BAabcd");
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "undo must work");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(undone.state.doc().plain_text(), "abcd");
}

#[test]
fn fresh_editor_state_accepts_the_first_insertion() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let state = EditorState::new(Arc::new(schema));
    assert!(state.is_ok(), "empty state must build");
    let Ok(state) = state else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(transaction.insert_text(Pos(1), "Olá").is_ok());
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "first insertion must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "Olá");
}
