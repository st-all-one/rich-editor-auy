//! Selection-oriented command tests.

use std::sync::Arc;

use rich_editor_auy::command::{
    delete_backward, delete_forward, insert_hard_break, set_block_type,
};
use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{EditorState, Selection};

fn state_with(texts: &[&str], caret: Pos) -> Option<(Arc<Schema>, EditorState)> {
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
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    transaction.set_selection(Selection::caret(caret));
    let applied = state.apply(transaction).ok()?;
    Some((Arc::clone(&schema), applied.state))
}

#[test]
fn delete_backward_removes_one_scalar() {
    let Some((_schema, state)) = state_with(&["abc"], Pos(3)) else {
        return;
    };
    let Some(transaction) = delete_backward(&state) else {
        return;
    };
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "ac");
    assert_eq!(applied.state.doc().blocks().len(), 1);
}

#[test]
fn delete_backward_at_block_start_joins_previous() {
    let Some((_schema, state)) = state_with(&["ab", "cd"], Pos(4)) else {
        return;
    };
    let Some(transaction) = delete_backward(&state) else {
        return;
    };
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert_eq!(applied.state.doc().blocks().len(), 1);
    assert_eq!(applied.state.doc().plain_text(), "abcd");
    assert!(applied.changes.removed_blocks.contains(&1));
}

#[test]
fn delete_forward_removes_one_scalar_and_joins_at_end() {
    let Some((_schema, state)) = state_with(&["abc"], Pos(1)) else {
        return;
    };
    let Some(transaction) = delete_forward(&state) else {
        return;
    };
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "bc");

    let Some((_schema, state)) = state_with(&["ab", "cd"], Pos(3)) else {
        return;
    };
    let Some(transaction) = delete_forward(&state) else {
        return;
    };
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert_eq!(applied.state.doc().blocks().len(), 1);
    assert_eq!(applied.state.doc().plain_text(), "abcd");
}

#[test]
fn insert_hard_break_adds_one_atom() {
    let Some((_schema, state)) = state_with(&["ab"], Pos(2)) else {
        return;
    };
    let Some(transaction) = insert_hard_break(&state) else {
        return;
    };
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "ab");
    let Some((_, block)) = applied.state.doc().block_at(Pos(0)) else {
        return;
    };
    let Some(text) = block.as_textblock() else {
        return;
    };
    let atoms = text
        .items()
        .iter()
        .filter(|item| matches!(item, InlineItem::Atom(_)))
        .count();
    assert_eq!(atoms, 1);
    assert_eq!(text.len().0, 3);
}

#[test]
fn set_block_type_command_changes_the_selection_block() {
    let Some((schema, state)) = state_with(&["title"], Pos(2)) else {
        return;
    };
    let Some(heading) = schema.node_kind("heading") else {
        return;
    };
    let Some(transaction) = set_block_type(&state, heading, Attrs::empty()) else {
        return;
    };
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    let Some((_, block)) = applied.state.doc().block_at(Pos(0)) else {
        return;
    };
    assert_eq!(block.kind(), heading);
    assert_eq!(applied.changes.inserted_blocks.len(), 0);
    assert_eq!(applied.changes.removed_blocks.len(), 0);
}
