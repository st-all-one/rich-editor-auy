//! Accessibility invariant tests.

use rich_editor_auy::model::Doc;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json};

fn parse(json: &str) -> Option<Doc> {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return None;
    };
    from_json(json, &schema, &ParseOptions::default()).ok()
}

#[test]
fn a11y_accepts_a_complete_document() {
    let doc = parse(
        r#"{"type":"doc","attrs":{"lang":"pt-BR"},"content":[{"type":"paragraph","content":[{"type":"text","text":"Olá"}]}]}"#,
    );
    assert!(doc.is_some(), "document must parse");
    let Some(doc) = doc else {
        return;
    };
    let schema = Schema::basic();
    assert!(schema.is_ok());
    let Ok(schema) = schema else {
        return;
    };
    assert!(doc.validate_a11y(&schema).is_ok());
}

#[test]
fn a11y_reports_missing_image_alt() {
    let doc = parse(r#"{"type":"doc","content":[{"type":"image","attrs":{"src":"x.png"}}]}"#);
    let Some(doc) = doc else {
        return;
    };
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let report = doc.validate_a11y(&schema);
    let issue = report
        .errors
        .iter()
        .find(|issue| issue.code.as_str() == "A001");
    assert!(issue.is_some(), "A001 must be reported");
    assert_eq!(issue.map(|issue| issue.path.as_str()), Some("0"));
}

#[test]
fn a11y_reports_heading_without_text() {
    let doc =
        parse(r#"{"type":"doc","content":[{"type":"heading","attrs":{"level":2},"content":[]}]}"#);
    let Some(doc) = doc else {
        return;
    };
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let report = doc.validate_a11y(&schema);
    let issue = report
        .errors
        .iter()
        .find(|issue| issue.code.as_str() == "A003");
    assert!(issue.is_some(), "A003 must be reported");
    assert_eq!(issue.map(|issue| issue.path.as_str()), Some("0"));
}

#[test]
fn a11y_reports_empty_list() {
    let doc = parse(r#"{"type":"doc","content":[{"type":"bullet_list","content":[]}]}"#);
    assert!(doc.is_none(), "empty list is structurally invalid");
}

#[test]
fn a11y_maps_link_marks_to_link_role() {
    let doc = parse(
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"site","marks":[{"type":"link","href":"https://example.test"}]}]}]}"#,
    );
    let Some(doc) = doc else {
        return;
    };
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let tree = doc.a11y_tree(&schema);
    let text = tree
        .root
        .children
        .first()
        .and_then(|paragraph| paragraph.children.first());
    assert_eq!(text.map(|node| node.role.name()), Some("link"));
    assert_eq!(
        text.and_then(|node| node.href.as_deref()),
        Some("https://example.test")
    );
    assert!(doc.validate_a11y(&schema).is_ok());
}
