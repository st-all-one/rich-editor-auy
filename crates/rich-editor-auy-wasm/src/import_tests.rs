//! Import ingestion tests that exercise anchors, history, and limits natively.

use rich_editor_auy::model::{AttrId, AttrValue, InlineItem};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::Editor as CoreEditor;
use rich_editor_auy::wire::{ParseOptions, from_json as parse_document};
use serde_json::json;

use crate::dto::{BatchLimits, ImportPayload};
use crate::ids::{IdPolicy, rewrite_fragment};
use crate::import::{apply_replacement, check_table_limit, run_import};
use crate::ops::decode_fragment_with;

#[test]
fn regenerate_rewrites_references_to_imported_anchors() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let value = json!({"blocks": [
        {"type": "paragraph", "attrs": {"id": "old-1"},
         "content": [{"type": "text", "text": "a"}]},
        {"type": "paragraph",
         "content": [{"type": "text", "text": "b", "marks": [
            {"type": "ref", "target": {"doc": "pub", "node": "old-1"}}
         ]}]}
    ]});
    let Ok(fragment) = decode_fragment_with(&schema, &value, &ParseOptions::default()) else {
        return;
    };
    let policy = IdPolicy::Regenerate(String::from("seed"));
    let Ok((rewritten, map)) = rewrite_fragment(&fragment, &policy, &schema) else {
        return;
    };
    assert_eq!(map.len(), 1);
    let Some(pair) = map.first() else {
        return;
    };
    assert_eq!(pair.from, "old-1");
    assert_ne!(pair.to, "old-1");
    let new = pair.to.clone();
    let Some(block) = rewritten.blocks().first() else {
        return;
    };
    assert_eq!(
        block.id().map(|id| String::from(id.as_str())),
        Some(new.clone())
    );
    let Some(second) = rewritten.blocks().get(1) else {
        return;
    };
    let Some(text) = second.as_textblock() else {
        return;
    };
    let mut target = None;
    for item in text.items() {
        let InlineItem::Text(run) = item else {
            continue;
        };
        for mark in run.marks.iter_attrs() {
            if let Some(AttrValue::Ref(value)) = mark.attrs.get(AttrId(0)) {
                target = value.node.as_ref().map(|node| String::from(node.as_str()));
            }
        }
    }
    assert_eq!(target.as_deref(), Some(new.as_str()));
}

#[test]
fn import_can_skip_history() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(mut editor) = CoreEditor::from_json(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"a"}]}]}"#,
    ) else {
        return;
    };
    let Ok(replacement) = parse_document(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"b"}]}]}"#,
        &schema,
        &ParseOptions::default(),
    ) else {
        return;
    };
    assert!(apply_replacement(&mut editor, replacement, Some(false)).is_ok());
    assert_eq!(editor.history_info().undo_depth, 0);
    let Ok(other) = parse_document(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"c"}]}]}"#,
        &schema,
        &ParseOptions::default(),
    ) else {
        return;
    };
    assert!(apply_replacement(&mut editor, other, Some(true)).is_ok());
    assert_eq!(editor.history_info().undo_depth, 1);
}

#[test]
fn ops_limit_rejects_before_applying() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(mut editor) = CoreEditor::from_json(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abc"}]}]}"#,
    ) else {
        return;
    };
    let Ok(payload) = serde_json::from_value::<ImportPayload>(json!({
        "mode": "ops",
        "ops": [{"op": "insert_text", "at": 1, "text": "x"}],
        "options": {"limits": {"maxOps": 0}}
    })) else {
        return;
    };
    let result = run_import(&mut editor, &schema, &payload);
    let Err(problem) = result else {
        return;
    };
    assert_eq!(&*problem.code, "limit_exceeded");
    assert_eq!(problem.instance.as_deref(), Some("ops"));
    assert_eq!(editor.doc().plain_text(), "abc");
}

#[test]
fn table_cell_limit_counts_nested_cells() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(editor) = CoreEditor::from_json(
        r#"{"type":"doc","content":[{"type":"table","content":[
            {"type":"table_row","content":[
                {"type":"table_cell","content":[{"type":"paragraph","content":[]}]}
            ]}
        ]}]}"#,
    ) else {
        return;
    };
    let zero = BatchLimits {
        max_table_cells: Some(0),
        ..BatchLimits::default()
    };
    let one = BatchLimits {
        max_table_cells: Some(1),
        ..BatchLimits::default()
    };
    let rejected = check_table_limit(&editor, &schema, Some(&zero));
    let Err(problem) = rejected else {
        return;
    };
    assert_eq!(&*problem.code, "limit_exceeded");
    assert_eq!(problem.instance.as_deref(), Some("tables"));
    assert!(check_table_limit(&editor, &schema, Some(&one)).is_ok());
}
