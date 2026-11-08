//! Hostile input limits.

use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json};

#[test]
fn deep_json_is_rejected_before_recursion_exhausts_the_stack() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let input = "[".repeat(1_000);
    let parsed = from_json(&input, &schema, &ParseOptions::default());
    assert!(parsed.is_err(), "deep JSON must fail");
    let Err(error) = parsed else {
        return;
    };
    assert_eq!(&*error.code, "depth_limit");
}

#[test]
fn decodes_unicode_surrogate_pairs() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let input = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"\uD83D\uDE00"}]}]}"#;
    let doc = from_json(input, &schema, &ParseOptions::default());
    assert!(doc.is_ok(), "surrogate pair must decode");
    let Ok(doc) = doc else {
        return;
    };
    assert_eq!(doc.plain_text(), "😀");
}

#[test]
fn rejects_a_lone_surrogate_with_an_offset() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let input = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"\uD83D"}]}]}"#;
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_err(), "lone surrogate must fail");
    let Err(error) = parsed else {
        return;
    };
    assert_eq!(&*error.code, "invalid_surrogate_pair");
    assert!(error.offset > 0, "error must carry a source offset");
}

#[test]
fn reports_the_offset_of_an_unknown_child() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let input = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"nope"}]}]}"#;
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_err(), "unknown child must fail");
    let Err(error) = parsed else {
        return;
    };
    assert_eq!(&*error.code, "unknown_node");
    assert!(error.offset > 0, "error must carry a source offset");
}

#[test]
fn enforces_block_text_and_run_limits() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let options = ParseOptions {
        limits: rich_editor_auy::Limits {
            max_block_text_bytes: 1,
            ..rich_editor_auy::Limits::default()
        },
    };
    let input = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"ab"}]}]}"#;
    assert!(
        from_json(input, &schema, &options).is_err(),
        "text limit must apply"
    );

    let options = ParseOptions {
        limits: rich_editor_auy::Limits {
            max_runs_per_block: 1,
            ..rich_editor_auy::Limits::default()
        },
    };
    let input = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"a","marks":[{"type":"strong"}]},{"type":"text","text":"b"}]}]}"#;
    assert!(
        from_json(input, &schema, &options).is_err(),
        "run limit must apply"
    );
}

#[test]
fn enforces_attribute_string_limits() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let options = ParseOptions {
        limits: rich_editor_auy::Limits {
            max_attr_string_bytes: 1,
            ..rich_editor_auy::Limits::default()
        },
    };
    let input =
        r#"{"type":"doc","attrs":{"lang":"pt"},"content":[{"type":"paragraph","content":[]}]}"#;
    assert!(
        from_json(input, &schema, &options).is_err(),
        "attribute string limit must apply"
    );
}
