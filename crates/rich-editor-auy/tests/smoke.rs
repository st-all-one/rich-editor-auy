//! End-to-end smoke tests for the canonical model and wire format.

use std::sync::Arc;

use rich_editor_auy::a11y::A11yRole;
use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, Mark, MarkSet, Run, TextBlock};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json};

#[test]
fn basic_schema_round_trips_canonical_json() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let paragraph = schema.node_kind("paragraph");
    assert!(paragraph.is_some(), "paragraph must exist");
    let Some(paragraph) = paragraph else {
        return;
    };
    let strong = schema.mark_id("strong");
    assert!(strong.is_some(), "strong must exist");
    let Some(strong) = strong else {
        return;
    };
    let marked = Run::new(String::from("Olá"), MarkSet::single(Mark::new(strong)));
    let plain = Run::new(String::from(", mundo"), MarkSet::empty());
    let block = Block::text(
        paragraph,
        Attrs::empty(),
        TextBlock::new(vec![InlineItem::Text(marked), InlineItem::Text(plain)]),
    );
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]);
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let json = doc.to_json_string(&schema);
    assert_eq!(
        json,
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Olá","marks":[{"type":"strong"}]},{"type":"text","text":", mundo"}]}]}"#
    );
    let decoded = from_json(&json, &schema, &ParseOptions::default());
    assert!(decoded.is_ok(), "document must decode");
    let Ok(decoded) = decoded else {
        return;
    };
    assert_eq!(decoded.to_json_string(&schema), json);
    assert_eq!(decoded.plain_text(), "Olá, mundo");
    let tree = decoded.a11y_tree(&schema);
    assert_eq!(tree.root.children.len(), 1);
    assert_eq!(
        tree.root.children.first().map(|node| node.role),
        Some(A11yRole::Paragraph)
    );
    let report = decoded.validate_a11y(&schema);
    assert!(report.is_ok(), "basic document must pass a11y errors");
    let validation = decoded.validate(&schema);
    assert!(
        validation.is_ok(),
        "valid document must pass structural validation"
    );
    let tree_json = tree.to_json_string(&schema);
    assert!(tree_json.contains("\"role\":\"document\""));
    assert!(tree_json.contains("\"role\":\"paragraph\""));
    assert!(tree_json.contains("\"marks\":[\"strong\"]"));
}
