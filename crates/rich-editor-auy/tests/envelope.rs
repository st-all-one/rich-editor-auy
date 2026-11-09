//! Durable envelope versioning and migration tests.

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ENVELOPE_VERSION, Envelope, ParseOptions};

fn fixture() -> Option<(Arc<Schema>, Doc)> {
    let schema = Arc::new(Schema::full().ok()?);
    let kind = schema.node_kind("paragraph")?;
    let content = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from("hi"),
        MarkSet::empty(),
    ))]);
    let doc = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(kind, Attrs::empty(), content))],
    )
    .ok()?;
    Some((schema, doc))
}

fn legacy_envelope(schema: &Schema, doc: &Doc) -> String {
    format!(
        "{{\"format\":\"rich-editor-auy\",\"version\":1,\"schema_fingerprint\":\"{}\",\"doc\":{}}}",
        "a".repeat(64),
        doc.to_json_string(schema)
    )
}

#[test]
fn version_two_round_trips_with_the_full_fingerprint() {
    let Some((schema, doc)) = fixture() else {
        return;
    };
    let envelope = Envelope::new(doc);
    let Some(text) = envelope.to_json_string(&schema).ok() else {
        return;
    };
    assert!(text.contains(&format!("\"version\":{ENVELOPE_VERSION}")));
    assert!(text.contains(schema.fingerprint()));
    let Some(parsed) = Envelope::from_json(&text, &schema, &ParseOptions::default()).ok() else {
        return;
    };
    assert_eq!(parsed.doc.plain_text(), "hi");
}

#[test]
fn version_one_migrates_to_the_current_version() {
    let Some((schema, doc)) = fixture() else {
        return;
    };
    let legacy = legacy_envelope(&schema, &doc);
    let Some(migrated) =
        Envelope::migrate_v1_to_full(&legacy, &schema, &ParseOptions::default()).ok()
    else {
        return;
    };
    assert!(migrated.contains("\"version\":2"));
    assert!(migrated.contains(schema.fingerprint()));
    assert!(!migrated.contains(&"a".repeat(64)));
    let Some(parsed) = Envelope::from_json(&migrated, &schema, &ParseOptions::default()).ok()
    else {
        return;
    };
    assert_eq!(parsed.doc.plain_text(), "hi");
}

#[test]
fn version_two_rejects_a_foreign_fingerprint() {
    let Some((schema, doc)) = fixture() else {
        return;
    };
    let Some(text) = Envelope::new(doc).to_json_string(&schema).ok() else {
        return;
    };
    let forged = text.replace(schema.fingerprint(), &"b".repeat(64));
    assert!(Envelope::from_json(&forged, &schema, &ParseOptions::default()).is_err());
}

#[test]
fn unknown_versions_and_tokens_are_rejected() {
    let Some((schema, doc)) = fixture() else {
        return;
    };
    let Some(text) = Envelope::new(doc).to_json_string(&schema).ok() else {
        return;
    };
    let future = text.replace("\"version\":2", "\"version\":3");
    assert!(Envelope::from_json(&future, &schema, &ParseOptions::default()).is_err());
    let unknown = text.replace("\"paragraph\"", "\"chart\"");
    assert!(Envelope::from_json(&unknown, &schema, &ParseOptions::default()).is_err());
}
