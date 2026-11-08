//! Compile-time schema front-end.

use rich_editor_auy::error::SchemaError;
use rich_editor_auy::schema::Schema;
use rich_editor_auy_macros::schema;

#[test]
fn declarative_schema_matches_the_builder_contract() {
    let schema: Result<Schema, SchemaError> = schema! {
        doc { content: "block+" }
        paragraph { group: "block", content: "text*", marks: "_" }
        heading { group: "block", content: "text*", marks: "_", attrs: { level: int(1..=6) = 1 } }
        text { inline: true }
        strong { mark }
        em { mark }
        link { mark, attrs: { href: str } }
    };
    assert!(schema.is_ok(), "macro schema must compile: {schema:?}");
    let Ok(schema) = schema else {
        return;
    };
    assert!(schema.node_kind("paragraph").is_some());
    assert!(schema.node_kind("heading").is_some());
    assert!(schema.mark_id("link").is_some());
    let heading = schema
        .node_kind("heading")
        .and_then(|kind| schema.node(kind));
    assert!(heading.is_some(), "heading must exist");
    let Some(heading) = heading else {
        return;
    };
    let level = heading.attrs.iter().find(|attr| &*attr.name == "level");
    assert!(level.is_some(), "level attr must exist");
    assert_eq!(heading.content.as_deref(), Some("text*"));
}
