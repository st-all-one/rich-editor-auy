//! Schema validation through the wire decoder.

use rich_editor_auy::error::ParseError;
use rich_editor_auy::model::Doc;
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
