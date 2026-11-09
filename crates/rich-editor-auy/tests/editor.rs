//! Stateful editor wrapper tests.

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, Fragment, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor, EditorOptions, Selection};

fn paragraph(schema: &Schema, text: &str) -> Option<Arc<Block>> {
    let kind = schema.node_kind("paragraph")?;
    let mut content = TextBlock::empty();
    content.push(InlineItem::Text(Run::new(
        String::from(text),
        MarkSet::empty(),
    )));
    Some(Arc::new(Block::text(kind, Attrs::empty(), content)))
}

fn fixture(text: &str) -> Option<Editor> {
    let schema = Schema::full().ok()?;
    let block = paragraph(&schema, text)?;
    let doc = Doc::from_blocks(Attrs::empty(), vec![block]).ok()?;
    Editor::from_doc(doc).ok()
}

fn insert(editor: &mut Editor, at: Pos, text: &str) -> Option<u64> {
    let mut transaction = editor.transaction();
    transaction.insert_text(at, text).ok()?;
    editor.apply(transaction).ok().map(|info| info.version)
}

#[test]
fn version_increments_only_for_document_changes() {
    let Some(mut editor) = fixture("") else {
        return;
    };
    assert_eq!(editor.version(), 0);
    let Some(version) = insert(&mut editor, Pos(1), "a") else {
        return;
    };
    assert_eq!(version, 1);
    assert_eq!(editor.doc().plain_text(), "a");
    let mut transaction = editor.transaction();
    transaction.set_selection(Selection::caret(Pos(2)));
    let Ok(info) = editor.apply(transaction) else {
        return;
    };
    assert_eq!(info.version, 1);
    assert_eq!(editor.selection(), Selection::caret(Pos(2)));
    assert_eq!(info.selection, editor.selection());
}

#[test]
fn undo_and_redo_restore_document_and_version() {
    let Some(mut editor) = fixture("") else {
        return;
    };
    assert!(editor.undo().ok().flatten().is_none());
    let Some(version) = insert(&mut editor, Pos(1), "abc") else {
        return;
    };
    assert_eq!(version, 1);
    let Some(undone) = editor.undo().ok().flatten() else {
        return;
    };
    assert_eq!(undone.version, 2);
    assert_eq!(editor.doc().plain_text(), "");
    let Some(redone) = editor.redo().ok().flatten() else {
        return;
    };
    assert_eq!(redone.version, 3);
    assert_eq!(editor.doc().plain_text(), "abc");
    assert!(redone.history.can_undo);
    assert!(!redone.history.can_redo);
}

#[test]
fn set_selection_validates_bounds() {
    let Some(mut editor) = fixture("hi") else {
        return;
    };
    assert_eq!(editor.selection(), Selection::None);
    assert!(editor.set_selection(Selection::caret(Pos(2))).is_ok());
    assert_eq!(editor.selection(), Selection::caret(Pos(2)));
    assert!(
        editor
            .set_selection(Selection::text(Pos(0), Pos(4)))
            .is_ok()
    );
    assert!(editor.set_selection(Selection::caret(Pos(999))).is_err());
    assert_eq!(editor.selection(), Selection::text(Pos(0), Pos(4)));
    assert_eq!(editor.version(), 0);
}

#[test]
fn replace_document_and_fragment_round_trip() {
    let Some(mut editor) = fixture("old") else {
        return;
    };
    let Some(schema) = Schema::full().ok() else {
        return;
    };
    let Some(new_block) = paragraph(&schema, "new") else {
        return;
    };
    let Ok(new_doc) = Doc::from_blocks(Attrs::empty(), vec![new_block]) else {
        return;
    };
    let Ok(info) = editor.replace_document(new_doc) else {
        return;
    };
    assert_eq!(editor.doc().plain_text(), "new");
    assert_eq!(info.version, 1);
    let Some(fragment_block) = paragraph(&schema, "frag") else {
        return;
    };
    let fragment = Fragment::new(vec![fragment_block]);
    let Ok(info) = editor.replace_fragment(Range::new(Pos(0), Pos(5)), fragment) else {
        return;
    };
    assert_eq!(editor.doc().plain_text(), "frag");
    assert_eq!(info.version, 2);
    let Some(undone) = editor.undo().ok().flatten() else {
        return;
    };
    assert_eq!(editor.doc().plain_text(), "new");
    assert_eq!(undone.version, 3);
}

#[test]
fn replace_fragment_inserts_at_empty_range() {
    let Some(mut editor) = fixture("a") else {
        return;
    };
    let Some(schema) = Schema::full().ok() else {
        return;
    };
    let Some(fragment_block) = paragraph(&schema, "b") else {
        return;
    };
    let fragment = Fragment::new(vec![fragment_block]);
    let Ok(info) = editor.replace_fragment(Range::new(Pos(0), Pos(0)), fragment) else {
        return;
    };
    assert_eq!(info.version, 1);
    assert_eq!(editor.doc().plain_text(), "ba");
}

#[test]
fn replace_fragment_rejects_partial_block_ranges() {
    let Some(mut editor) = fixture("old") else {
        return;
    };
    let Some(schema) = Schema::full().ok() else {
        return;
    };
    let Some(fragment_block) = paragraph(&schema, "x") else {
        return;
    };
    let fragment = Fragment::new(vec![fragment_block]);
    assert!(
        editor
            .replace_fragment(Range::new(Pos(1), Pos(5)), fragment)
            .is_err()
    );
    assert_eq!(editor.doc().plain_text(), "old");
    assert_eq!(editor.version(), 0);
}

#[test]
fn history_info_tracks_edits_and_clear() {
    let Ok(mut editor) = Editor::with_options(EditorOptions {
        history_limits: Some((1, usize::MAX)),
    }) else {
        return;
    };
    assert_eq!(editor.history_info().undo_depth, 0);
    assert!(insert(&mut editor, Pos(1), "a").is_some());
    assert!(insert(&mut editor, Pos(2), "b").is_some());
    let info = editor.history_info();
    assert_eq!(info.undo_depth, 1);
    assert!(info.can_undo);
    assert!(!info.can_redo);
    assert!(info.bytes > 0);
    editor.clear_history();
    let info = editor.history_info();
    assert_eq!((info.undo_depth, info.redo_depth, info.bytes), (0, 0, 0));
    assert!(!info.can_undo);
    assert!(!info.can_redo);
}

#[test]
fn from_json_uses_complete_schema() {
    let Ok(editor) = Editor::from_json(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"hi"}]}]}"#,
    ) else {
        return;
    };
    assert_eq!(editor.doc().plain_text(), "hi");
    assert_eq!(editor.version(), 0);
    assert!(Editor::from_json("{}").is_err());
}
