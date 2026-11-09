//! Open-slice transport and step rejection.

use rich_editor_auy::error::StepError;
use rich_editor_auy::model::{Block, Doc};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::step::{Step, apply_step};
use rich_editor_auy::wire::{ParseOptions, from_json, from_slice_json};

use std::sync::Arc;

/// Builds the full schema or returns `None` when the preset is inconsistent.
fn schema() -> Option<Arc<Schema>> {
    Schema::full().ok().map(Arc::new)
}

/// Builds a document from canonical JSON.
fn parse_doc(json: &str) -> Option<Doc> {
    let schema = schema()?;
    from_json(json, &schema, &ParseOptions::default()).ok()
}

#[test]
fn partial_edges_are_marked_open_and_survive_wire() {
    let Some(schema) = schema() else {
        return;
    };
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcdef"}]},{"type":"paragraph","content":[{"type":"text","text":"ghij"}]}]}"#,
    ) else {
        return;
    };
    let copied = doc.slice(Range::new(Pos(3), Pos(10)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    assert_eq!(copied.open_start, 1, "partial start block is open");
    assert_eq!(copied.open_end, 1, "partial end block is open");
    let json = copied.to_json_string(&schema);
    assert!(
        json.contains("\"openStart\":1"),
        "open start is encoded: {json}"
    );
    assert!(
        json.contains("\"openEnd\":1"),
        "open end is encoded: {json}"
    );
    let decoded = from_slice_json(&json, &schema, &ParseOptions::default());
    assert!(decoded.is_ok(), "marked slice must decode: {json}");
    let Ok(decoded) = decoded else {
        return;
    };
    assert_eq!(decoded, copied, "open bounds round-trip");
}

#[test]
fn malformed_open_bounds_are_rejected() {
    let Some(schema) = schema() else {
        return;
    };
    for json in [
        r#"{"blocks":[],"openStart":2}"#,
        r#"{"text":"abc","openStart":1}"#,
        r#"{"text":"abc","openEnd":true}"#,
    ] {
        let decoded = from_slice_json(json, &schema, &ParseOptions::default());
        assert!(decoded.is_err(), "must reject: {json}");
        assert!(
            matches!(decoded, Err(ref error) if error.code.as_ref() == "invalid_open"),
            "typed rejection: {json}"
        );
    }
}

#[test]
fn step_algebra_rejects_open_slices() {
    let Some(schema) = schema() else {
        return;
    };
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcdef"}]},{"type":"paragraph","content":[{"type":"text","text":"ghij"}]}]}"#,
    ) else {
        return;
    };
    let copied = doc.slice(Range::new(Pos(3), Pos(10)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    let step = Step::Replace {
        from: Pos(3),
        to: Pos(8),
        slice: copied,
    };
    assert!(
        matches!(
            apply_step(&doc, &schema, &step),
            Err(StepError::Unsupported { ref operation })
                if operation.as_ref() == "open_slice"
        ),
        "open slices are not part of the step algebra"
    );
}

/// Returns the plain text of one top-level block.
fn block_text(block: &Block) -> Option<String> {
    let mut out = String::new();
    block.as_textblock()?.text_into(&mut out);
    Some(out)
}

#[test]
fn insert_open_slice_merges_both_edges() {
    let Some(schema) = schema() else {
        return;
    };
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcdef"}]}]}"#,
    ) else {
        return;
    };
    let Some(source) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"XYZ"}]},{"type":"paragraph","content":[{"type":"text","text":"UVW"}]}]}"#,
    ) else {
        return;
    };
    let copied = source.slice(Range::new(Pos(2), Pos(7)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    assert_eq!(copied.open_start, 1);
    assert_eq!(copied.open_end, 1);
    let inserted = doc.insert_slice(&schema, Pos(3), &copied);
    assert!(inserted.is_ok(), "open insert must succeed");
    let Ok(inserted) = inserted else {
        return;
    };
    let texts: Vec<_> = inserted
        .blocks()
        .iter()
        .filter_map(|block| block_text(block))
        .collect();
    assert_eq!(texts, vec![String::from("abYZ"), String::from("Ucdef")]);
}

#[test]
fn insert_open_slice_requires_an_inside_position() {
    let Some(schema) = schema() else {
        return;
    };
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcdef"}]}]}"#,
    ) else {
        return;
    };
    let Some(source) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"XYZ"}]}]}"#,
    ) else {
        return;
    };
    let copied = source.slice(Range::new(Pos(0), Pos(4)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    let inserted = doc.insert_slice(&schema, Pos(0), &copied);
    assert!(
        matches!(
            inserted,
            Err(StepError::Unsupported { ref operation }) if operation.as_ref() == "open_slice_boundary"
        ),
        "open edges cannot merge at a block boundary"
    );
}

#[test]
fn replace_open_slice_merges_partial_edges() {
    let Some(schema) = schema() else {
        return;
    };
    let Some(doc) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcdef"}]},{"type":"paragraph","content":[{"type":"text","text":"ghij"}]},{"type":"paragraph","content":[{"type":"text","text":"klmn"}]}]}"#,
    ) else {
        return;
    };
    let Some(source) = parse_doc(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"XYZ"}]},{"type":"paragraph","content":[{"type":"text","text":"UVW"}]}]}"#,
    ) else {
        return;
    };
    let copied = source.slice(Range::new(Pos(2), Pos(7)));
    assert!(copied.is_ok(), "copy must succeed");
    let Ok(copied) = copied else {
        return;
    };
    let replaced = doc.replace_slice(&schema, Range::new(Pos(3), Pos(11)), &copied);
    assert!(replaced.is_ok(), "open replace must succeed");
    let Ok(replaced) = replaced else {
        return;
    };
    let texts: Vec<_> = replaced
        .blocks()
        .iter()
        .filter_map(|block| block_text(block))
        .collect();
    assert_eq!(
        texts,
        vec![
            String::from("abYZ"),
            String::from("Uij"),
            String::from("klmn")
        ]
    );
}
