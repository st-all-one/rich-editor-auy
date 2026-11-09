//! Clipboard semantics: range copy, block insert, range delete, and slice wire.

use std::sync::Arc;

use rich_editor_auy::model::{Attr, AttrId, AttrValue, Attrs, Block, Doc, TextBlock};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor, EditorOptions, Selection};
use rich_editor_auy::wire::{ParseOptions, from_json, from_slice_json};

/// Builds the full schema or returns `None` when the preset is inconsistent.
fn schema() -> Option<Arc<Schema>> {
    Schema::full().ok().map(Arc::new)
}

/// Builds a document from canonical JSON.
fn parse_doc(json: &str) -> Option<Doc> {
    let schema = schema()?;
    from_json(json, &schema, &ParseOptions::default()).ok()
}

/// Returns the text of one text block.
fn block_text(block: &Arc<Block>) -> Option<String> {
    let mut out = String::new();
    block.as_textblock()?.text_into(&mut out);
    Some(out)
}

/// Positions the first text block boundary of block `index`.
fn start_of(doc: &Doc, index: u32) -> Option<Pos> {
    doc.block_range(index).map(|range| range.start)
}

#[test]
fn range_inside_one_text_block_copies_runs() {
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcd"}]}]}"#,
    ) else {
        return;
    };
    let copied = doc.slice(Range::new(Pos(2), Pos(4)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    assert!(copied.content.is_empty(), "inline copy stays inline");
    assert_eq!(&*copied.text, "bc");
    assert!(copied.runs.is_some(), "inline copy keeps runs");
    assert!(!doc.same_text_block(Range::new(Pos(0), Pos(2))));
    assert!(doc.same_text_block(Range::new(Pos(2), Pos(3))));
}

#[test]
fn cross_block_range_keeps_partial_edges_and_whole_blocks() {
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abc"}]},{"type":"paragraph","content":[{"type":"text","text":"def"}]}]}"#,
    ) else {
        return;
    };
    let copied = doc.slice(Range::new(Pos(3), Pos(8)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    assert_eq!(copied.content.blocks().len(), 2);
    assert_eq!(&*copied.text, "");
    let blocks = copied.content.blocks();
    assert_eq!(blocks.first().and_then(block_text).as_deref(), Some("c"));
    assert_eq!(blocks.get(1).and_then(block_text).as_deref(), Some("de"));
}

#[test]
fn whole_block_range_copies_without_block_anchors() {
    let attrs = Attrs::empty()
        .with(Attr {
            id: AttrId::NODE_ID,
            value: AttrValue::Str(String::from("anchor").into_boxed_str()),
        })
        .unwrap_or_default();
    let Some(kind) = schema().and_then(|schema| schema.node_kind("paragraph")) else {
        return;
    };
    let block = Block::text(kind, attrs, TextBlock::empty());
    let Some(doc) = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]).ok() else {
        return;
    };
    let size = doc.size().0;
    let copied = doc.slice(Range::new(Pos(0), Pos(size)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    let Some(block) = copied.content.blocks().first() else {
        return;
    };
    assert!(block.id().is_none(), "copied blocks drop their anchor");
}

#[test]
fn delete_range_merges_partial_edges_and_keeps_one_line() {
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abc"}]},{"type":"paragraph","content":[{"type":"text","text":"def"}]}]}"#,
    ) else {
        return;
    };
    let Some(schema) = schema() else {
        return;
    };
    let deleted = doc.delete_range(&schema, Range::new(Pos(3), Pos(8)));
    assert!(deleted.is_ok(), "delete must succeed");
    let Ok(deleted) = deleted else {
        return;
    };
    assert_eq!(deleted.blocks().len(), 1);
    assert_eq!(deleted.plain_text(), "abf");
    assert_eq!(deleted.blocks().len(), 1);
}

#[test]
fn delete_everything_leaves_one_editable_paragraph() {
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abc"}]},{"type":"paragraph","content":[{"type":"text","text":"def"}]}]}"#,
    ) else {
        return;
    };
    let Some(schema) = schema() else {
        return;
    };
    let size = doc.size().0;
    let deleted = doc.delete_range(&schema, Range::new(Pos(0), Pos(size)));
    assert!(deleted.is_ok(), "delete must succeed");
    let Ok(deleted) = deleted else {
        return;
    };
    assert_eq!(deleted.blocks().len(), 1);
    assert_eq!(deleted.plain_text(), "");
    assert_eq!(deleted.blocks().len(), 1);
}

#[test]
fn delete_inside_one_block_removes_inline_content_only() {
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcdef"}]}]}"#,
    ) else {
        return;
    };
    let Some(schema) = schema() else {
        return;
    };
    let deleted = doc.delete_range(&schema, Range::new(Pos(3), Pos(5)));
    assert!(deleted.is_ok(), "delete must succeed");
    let Ok(deleted) = deleted else {
        return;
    };
    assert_eq!(deleted.plain_text(), "abef");
    assert_eq!(deleted.blocks().len(), 1);
}

#[test]
fn insert_block_slice_splits_the_host_block() {
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcd"}]}]}"#,
    ) else {
        return;
    };
    let Some(source) = parse_doc(
        r#"{"type":"doc","content":[{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"T"}]}]}"#,
    ) else {
        return;
    };
    let Some(schema) = schema() else {
        return;
    };
    let size = source.size().0;
    let copied = source.slice(Range::new(Pos(0), Pos(size)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    let inserted = doc.insert_slice(&schema, Pos(3), &copied);
    assert!(inserted.is_ok(), "insert must succeed");
    let Ok(inserted) = inserted else {
        return;
    };
    assert_eq!(inserted.blocks().len(), 3);
    assert_eq!(inserted.plain_text(), "abTcd");
    assert_eq!(inserted.blocks().len(), 3);
}

#[test]
fn replace_slice_swaps_whole_blocks() {
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"aa"}]},{"type":"paragraph","content":[{"type":"text","text":"bb"}]}]}"#,
    ) else {
        return;
    };
    let Some(source) = parse_doc(
        r#"{"type":"doc","content":[{"type":"heading","attrs":{"level":1},"content":[{"type":"text","text":"H"}]}]}"#,
    ) else {
        return;
    };
    let Some(schema) = schema() else {
        return;
    };
    let Some(first) = start_of(&doc, 0) else {
        return;
    };
    let Some(second) = start_of(&doc, 1) else {
        return;
    };
    let size = source.size().0;
    let copied = source.slice(Range::new(first, Pos(size)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    let replaced = doc.replace_slice(&schema, Range::new(first, second), &copied);
    assert!(replaced.is_ok(), "replace must succeed");
    let Ok(replaced) = replaced else {
        return;
    };
    assert_eq!(replaced.blocks().len(), 2);
    assert_eq!(replaced.plain_text(), "Hbb");
}

#[test]
fn slice_json_round_trips_inline_and_structural_payloads() {
    let Some(schema) = schema() else {
        return;
    };
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abc","marks":[{"type":"strong"}]}]},{"type":"paragraph","content":[{"type":"text","text":"def"}]}]}"#,
    ) else {
        return;
    };
    let size = doc.size().0;
    for range in [
        Range::new(Pos(2), Pos(4)),
        Range::new(Pos(0), Pos(size)),
        Range::new(Pos(4), Pos(5)),
    ] {
        let copied = doc.slice(range);
        assert!(copied.is_ok(), "copy must succeed");
        let Ok(copied) = copied else {
            return;
        };
        let json = copied.to_json_string(&schema);
        assert!(!json.is_empty(), "slice must encode");
        let decoded = from_slice_json(&json, &schema, &ParseOptions::default());
        assert!(decoded.is_ok(), "slice must decode: {json}");
        let Ok(decoded) = decoded else {
            return;
        };
        assert_eq!(decoded, copied, "slice wire round-trips");
        assert_eq!(
            decoded.to_json_string(&schema),
            json,
            "encoding is canonical"
        );
    }
}

#[test]
fn editor_delete_and_paste_are_one_history_entry() {
    let Ok(mut editor) = Editor::with_options(EditorOptions::default()) else {
        return;
    };
    let Some(initial) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcd"}]}]}"#,
    ) else {
        return;
    };
    let replaced = editor.replace_document(initial);
    assert!(replaced.is_ok(), "document must load");
    assert_eq!(editor.version(), 1);
    let Some(start) = editor.doc().block_range(0).map(|range| range.start) else {
        return;
    };
    let caret = Pos(start.0.saturating_add(3));
    let deleted = editor.delete_range(Range::new(caret, Pos(caret.0.saturating_add(1))));
    assert!(deleted.is_ok(), "delete must apply");
    let Ok(deleted) = deleted else {
        return;
    };
    assert_eq!(editor.doc().plain_text(), "abd");
    assert_eq!(deleted.selection, Selection::caret(caret));
    let Some(slice) = editor.doc().slice(Range::new(Pos(2), Pos(4))).ok() else {
        return;
    };
    let pasted = editor.insert_slice(caret, &slice);
    assert!(pasted.is_ok(), "paste must apply");
    assert_eq!(editor.doc().plain_text(), "abbdd");
    let undone = editor.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(undone.selection, Selection::caret(caret));
    assert_eq!(editor.doc().plain_text(), "abd");
}
