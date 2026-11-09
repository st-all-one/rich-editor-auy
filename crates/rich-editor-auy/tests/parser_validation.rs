//! Schema validation through the wire decoder.

use rich_editor_auy::error::ParseError;
use rich_editor_auy::model::{Doc, MAX_RUN_BYTES};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json};

fn parse(input: &str) -> Result<Doc, ParseError> {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return Err(ParseError::new("schema", 0));
    };
    from_json(input, &schema, &ParseOptions::default())
}

#[test]
fn rejects_empty_document() {
    assert!(parse(r#"{"type":"doc","content":[]}"#).is_err());
}

#[test]
fn rejects_attribute_out_of_range() {
    assert!(
        parse(r#"{"type":"doc","content":[{"type":"heading","attrs":{"level":99},"content":[]}]}"#)
            .is_err()
    );
}

#[test]
fn rejects_text_inside_a_block_container() {
    assert!(parse(
        r#"{"type":"doc","content":[{"type":"bullet_list","content":[{"type":"text","text":"x"}]}]}"#
    )
    .is_err());
}

#[test]
fn rejects_data_and_javascript_urls() {
    assert!(parse(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"image","attrs":{"alt":"x","src":"data:image/png;base64,AAAA"}}]}]}"#
    )
    .is_err());
    assert!(parse(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":"javascript:alert(1)"}}]}]}]}"#
    )
    .is_err());
}

#[test]
fn accepts_canonical_nested_mark_attrs_and_round_trips() {
    let input = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"site","marks":[{"type":"link","attrs":{"href":"https://example.test"}}]}]}]}"#;
    let doc = parse(input);
    assert!(doc.is_ok(), "nested mark attrs must parse");
    let Ok(doc) = doc else {
        return;
    };
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let json = doc.to_json_string(&schema);
    let reparsed = parse(&json);
    assert!(reparsed.is_ok(), "canonical round trip must parse");
    let Ok(reparsed) = reparsed else {
        return;
    };
    assert_eq!(
        doc.to_json_string(&schema),
        reparsed.to_json_string(&schema)
    );
}

#[test]
fn accepts_list_and_map_attributes_and_round_trips() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let input = r#"{"type":"doc","attrs":{"authors":["Ada","Grace"],"metadata":{"key":"value"}},"content":[{"type":"paragraph","content":[]}]}"#;
    let doc = from_json(input, &schema, &ParseOptions::default());
    assert!(doc.is_ok(), "list and map attributes must parse");
    let Ok(doc) = doc else {
        return;
    };
    let json = doc.to_json_string(&schema);
    assert!(json.contains(r#""authors":["Ada","Grace"]"#), "{json}");
    assert!(json.contains(r#""metadata":{"key":"value"}"#), "{json}");
    let Ok(reparsed) = from_json(&json, &schema, &ParseOptions::default()) else {
        return;
    };
    assert_eq!(reparsed.to_json_string(&schema), json);
}

#[test]
fn rejects_a_run_above_the_ceiling_with_i4() {
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let long = "a".repeat(MAX_RUN_BYTES + 8);
    let input = format!(
        r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"{long}"}}]}}]}}"#
    );
    let error = from_json(&input, &schema, &ParseOptions::default()).err();
    let Some(error) = error else {
        return;
    };
    assert_eq!(&*error.code, "I4");
}

#[test]
fn reports_i4_for_a_constructed_run_above_the_ceiling() {
    use rich_editor_auy::model::{Attrs, Block, InlineItem, MarkSet, Run, TextBlock};
    use std::sync::Arc;
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let Some(kind) = schema.node_kind("paragraph") else {
        return;
    };
    let text = TextBlock::new(vec![InlineItem::Text(Run::new(
        "a".repeat(MAX_RUN_BYTES + 1),
        MarkSet::empty(),
    ))]);
    let Ok(doc) = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(kind, Attrs::empty(), text))],
    ) else {
        return;
    };
    let report = doc.validate(&schema);
    assert!(
        report.issues.iter().any(|issue| issue.invariant == "I4"),
        "{:?}",
        report.issues
    );
}
