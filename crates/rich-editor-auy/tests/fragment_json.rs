//! Fragment JSON decoding and canonical writing.

use rich_editor_auy::error::ParseError;
use rich_editor_auy::model::Fragment;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_fragment_json};

const VALID: &str =
    r#"{"blocks":[{"type":"paragraph","content":[{"type":"text","text":"hello"}]}]}"#;

fn schema() -> Option<Schema> {
    Schema::basic().ok()
}

fn decode(input: &str) -> Result<Fragment, ParseError> {
    let Some(schema) = schema() else {
        return Err(ParseError::new("schema", 0));
    };
    from_fragment_json(input, &schema, &ParseOptions::default())
}

fn code(input: &str) -> Option<String> {
    decode(input).err().map(|error| error.code.to_string())
}

#[test]
fn decodes_valid_fragment() {
    let Some(schema) = schema() else {
        return;
    };
    let Ok(fragment) = decode(VALID) else {
        return;
    };
    assert_eq!(fragment.blocks().len(), 1);
    assert_eq!(fragment.to_json_string(&schema), VALID);
}

#[test]
fn round_trips_canonical_fragment() {
    let Some(schema) = schema() else {
        return;
    };
    let Ok(fragment) = decode(VALID) else {
        return;
    };
    let canonical = fragment.to_json_string(&schema);
    let Ok(again) = from_fragment_json(&canonical, &schema, &ParseOptions::default()) else {
        return;
    };
    assert_eq!(again, fragment);
    assert_eq!(again.to_json_string(&schema), canonical);
}

#[test]
fn decodes_empty_fragment() {
    let Some(schema) = schema() else {
        return;
    };
    let Ok(fragment) = decode(r#"{"blocks":[]}"#) else {
        return;
    };
    assert!(fragment.is_empty());
    assert_eq!(fragment.to_json_string(&schema), r#"{"blocks":[]}"#);
}

#[test]
fn rejects_missing_blocks() {
    assert_eq!(code("{}").as_deref(), Some("missing_blocks"));
}

#[test]
fn rejects_non_array_blocks() {
    assert_eq!(
        code(r#"{"blocks":{"type":"paragraph"}}"#).as_deref(),
        Some("blocks_not_array")
    );
}

#[test]
fn rejects_non_object_fragment() {
    assert_eq!(
        code(r#"[{"type":"paragraph"}]"#).as_deref(),
        Some("fragment_not_object")
    );
}

#[test]
fn rejects_unknown_node() {
    let Some(error) = decode(r#"{"blocks":[{"type":"mystery"}]}"#).err() else {
        return;
    };
    assert_eq!(error.code.as_ref(), "unknown_node");
    assert_eq!(error.path.as_ref(), "0");
}

#[test]
fn rejects_unknown_mark() {
    let Some(error) = decode(
        r#"{"blocks":[{"type":"paragraph","content":[{"type":"text","text":"a","marks":[{"type":"mystery"}]}]}]}"#,
    )
    .err() else {
        return;
    };
    assert_eq!(error.code.as_ref(), "unknown_mark");
}

#[test]
fn rejects_unknown_attribute() {
    let Some(error) = decode(r#"{"blocks":[{"type":"paragraph","attrs":{"mystery":1}}]}"#).err()
    else {
        return;
    };
    assert_eq!(error.code.as_ref(), "unknown_attr");
    assert_eq!(error.path.as_ref(), "0");
}

#[test]
fn rejects_inline_atom_at_block_position() {
    let Some(error) = decode(r#"{"blocks":[{"type":"image","attrs":{"alt":"a"}}]}"#).err() else {
        return;
    };
    assert_eq!(error.code.as_ref(), "I1");
}

#[test]
fn rejects_missing_text() {
    assert_eq!(
        code(r#"{"blocks":[{"type":"paragraph","content":[{"type":"text"}]}]}"#).as_deref(),
        Some("missing_text")
    );
}
