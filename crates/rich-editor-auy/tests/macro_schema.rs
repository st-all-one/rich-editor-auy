//! Compile-time schema front-end.

use rich_editor_auy::a11y::{A11yRole, NamePolicy};
use rich_editor_auy::error::SchemaError;
use rich_editor_auy::schema::{AttrType, Schema};
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

rich_editor_auy_macros::schema! {
    pub mod kinds;
    doc { content: "block+", attrs: { lang: str }, a11y: { role: document } }
    paragraph {
        group: "block",
        content: "text*",
        marks: "_",
        a11y: { role: paragraph, name: text }
    }
    heading {
        group: "block",
        content: "text*",
        marks: "_",
        attrs: { level: int(1..=6) = 1 },
        a11y: { role: heading(2), name: text, level: "level" }
    }
    bullet_list { group: "block", content: "list_item+", a11y: { role: list } }
    list_item { group: "block", content: "block+", a11y: { role: list_item } }
    text { inline: true, a11y: { role: text, name: text } }
    strong { mark, a11y: { role: text } }
    link {
        mark,
        attrs: { href: str required, title: str },
        a11y: { role: link, name: text }
    }
}

#[test]
fn typed_accessors_match_the_builder() {
    let Ok(schema) = kinds::build() else {
        return;
    };
    assert_eq!(schema.node_kind("paragraph"), Some(kinds::NODE_PARAGRAPH));
    assert_eq!(schema.node_kind("heading"), Some(kinds::NODE_HEADING));
    let Some(heading) = schema.node(kinds::NODE_HEADING) else {
        return;
    };
    let Some(level) = heading.attrs.iter().find(|attr| &*attr.name == "level") else {
        return;
    };
    assert_eq!(level.id, kinds::NODE_HEADING_LEVEL);
    assert_eq!(kinds::NODE_HEADING_LEVEL.0, 1);
    let Some(lang) = schema.doc().attrs.iter().find(|attr| &*attr.name == "lang") else {
        return;
    };
    assert_eq!(lang.id, kinds::DOC_LANG);
    assert_eq!(schema.mark_id("link"), Some(kinds::MARK_LINK));
    let Some(link) = schema.mark(kinds::MARK_LINK) else {
        return;
    };
    let Some(href) = link.attrs.iter().find(|attr| &*attr.name == "href") else {
        return;
    };
    assert_eq!(href.id, kinds::MARK_LINK_HREF);
    assert_eq!(kinds::MARK_LINK_HREF.0, 0);
    assert_eq!(kinds::MARK_LINK_TITLE.0, 1);
    assert_eq!(kinds::NODE_PARAGRAPH.0, 0);
    assert_eq!(kinds::NODE_HEADING.0, 1);
}

#[test]
fn declarative_a11y_reaches_the_compiled_schema() {
    let Ok(schema) = kinds::build() else {
        return;
    };
    let Some(heading) = schema.node(kinds::NODE_HEADING) else {
        return;
    };
    let Some(a11y) = heading.a11y else {
        return;
    };
    assert_eq!(a11y.role, A11yRole::Heading { level: 2 });
    assert_eq!(a11y.name, NamePolicy::TextContent);
    assert_eq!(a11y.level, Some(kinds::NODE_HEADING_LEVEL));
    let Some(link) = schema.mark(kinds::MARK_LINK) else {
        return;
    };
    let Some(a11y) = link.a11y else {
        return;
    };
    assert_eq!(a11y.role, A11yRole::Link);
    let Some(href) = link.attrs.iter().find(|attr| &*attr.name == "href") else {
        return;
    };
    assert!(href.required, "href must be required");
}

#[test]
fn extended_attribute_types_compile() {
    let schema: Result<Schema, SchemaError> = schema! {
        doc { content: "block+" }
        paragraph { group: "block", content: "text*", marks: "_" }
        widget {
            inline: true,
            atom: true,
            attrs: {
                mode: enum("compact", "full") = "compact",
                tags: list(str),
                meta: map(str)
            }
        }
        text { inline: true }
    };
    assert!(schema.is_ok(), "extended types must compile: {schema:?}");
    let Ok(schema) = schema else {
        return;
    };
    let Some(widget) = schema
        .node_kind("widget")
        .and_then(|kind| schema.node(kind))
    else {
        return;
    };
    let Some(mode) = widget.attrs.iter().find(|attr| &*attr.name == "mode") else {
        return;
    };
    assert!(matches!(mode.kind, AttrType::Enum(_)));
    let Some(tags) = widget.attrs.iter().find(|attr| &*attr.name == "tags") else {
        return;
    };
    assert!(matches!(tags.kind, AttrType::List(_)));
    let Some(meta) = widget.attrs.iter().find(|attr| &*attr.name == "meta") else {
        return;
    };
    assert!(matches!(meta.kind, AttrType::Map(_)));
}
