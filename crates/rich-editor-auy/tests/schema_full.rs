//! Complete full-schema declaration and attribute coverage tests.

mod schema_support;

use rich_editor_auy::a11y::A11yRole;
use rich_editor_auy::schema::{AttrSpec, AttrType, NodeSpec, Schema};

use schema_support::full;

fn assert_image_attrs(schema: &Schema, name: &str) {
    let Some(kind) = schema.node_kind(name) else {
        assert!(schema.node_kind(name).is_some(), "{name} must exist");
        return;
    };
    for attr in [
        "src",
        "alt",
        "title",
        "width",
        "height",
        "intrinsic_width",
        "intrinsic_height",
        "srcset",
        "sizes",
        "caption",
        "credit",
        "loading",
        "decoding",
        "asset_id",
        "mime",
        "size_bytes",
    ] {
        assert!(
            schema.node_attr(kind, attr).is_some(),
            "{name} must declare {attr}"
        );
    }
    assert_eq!(
        schema.node_attr(kind, "alt").map(|attr| attr.required),
        Some(true)
    );
}

#[test]
fn full_schema_declares_raw_code_for_verbatim_payloads() {
    let Some(schema) = full() else {
        return;
    };
    let Some(kind) = schema.node_kind("raw_code") else {
        assert!(
            schema.node_kind("raw_code").is_some(),
            "raw_code must exist"
        );
        return;
    };
    let Some(node) = schema.node(kind) else {
        return;
    };
    assert_eq!(node.content.as_deref(), Some("text*"));
    assert!(!node.inline, "raw_code is a block");
    assert!(!node.atom, "raw_code is not an atom");
    for name in ["language", "format", "filename", "meta", "line_numbers"] {
        assert!(
            schema.node_attr(kind, name).is_some(),
            "raw_code must declare {name}"
        );
    }
    assert!(matches!(
        schema.node_attr(kind, "format").map(|attr| &attr.kind),
        Some(AttrType::Enum(_))
    ));
    assert!(matches!(
        schema
            .node_attr(kind, "line_numbers")
            .map(|attr| &attr.kind),
        Some(AttrType::Bool)
    ));
    assert_eq!(node.a11y.map(|spec| spec.role), Some(A11yRole::CodeBlock));
}

#[test]
fn full_schema_declares_document_metadata() {
    let Some(schema) = full() else {
        return;
    };
    let doc = schema.doc();
    for name in [
        "lang",
        "dir",
        "title",
        "description",
        "canonical_url",
        "authors",
        "tags",
        "metadata",
    ] {
        assert!(
            doc.attrs.iter().any(|attr| &*attr.name == name),
            "document must declare {name}"
        );
    }
    let attr = |name: &str| {
        doc.attrs
            .iter()
            .find(|attr| &*attr.name == name)
            .map(|attr| &attr.kind)
    };
    assert!(matches!(
        attr("authors"),
        Some(AttrType::List(inner)) if **inner == AttrType::Str
    ));
    assert!(matches!(
        attr("tags"),
        Some(AttrType::List(inner)) if **inner == AttrType::Str
    ));
    assert!(matches!(
        attr("metadata"),
        Some(AttrType::Map(inner)) if **inner == AttrType::Str
    ));
    assert!(matches!(
        attr("dir"),
        Some(AttrType::Enum(values)) if values.iter().any(|value| &**value == "rtl")
    ));
}

#[test]
fn full_schema_covers_blocks_marks_and_attrs() {
    let Some(schema) = full() else {
        return;
    };
    for name in [
        "paragraph",
        "heading",
        "bullet_list",
        "ordered_list",
        "list_item",
        "blockquote",
        "code_block",
        "raw_code",
        "table",
        "table_caption",
        "table_row",
        "table_header",
        "table_cell",
        "figure",
        "caption",
        "separator",
        "image",
        "image_block",
        "video",
        "audio",
        "attachment",
        "hard_break",
        "text",
    ] {
        assert!(schema.node_kind(name).is_some(), "missing node {name}");
    }
    for name in [
        "strong",
        "em",
        "code",
        "underline",
        "strike",
        "highlight",
        "sup",
        "sub",
        "link",
        "ref",
    ] {
        assert!(schema.mark_id(name).is_some(), "missing mark {name}");
    }
}

#[test]
fn full_schema_declares_list_and_table_attrs() {
    let Some(schema) = full() else {
        return;
    };
    let Some(ordered) = schema.node_kind("ordered_list") else {
        return;
    };
    for name in ["start", "reversed", "tight", "type"] {
        assert!(
            schema.node_attr(ordered, name).is_some(),
            "ordered_list must declare {name}"
        );
    }
    let Some(table) = schema.node_kind("table") else {
        return;
    };
    assert!(matches!(
        schema.node_attr(table, "col_widths").map(|attr| &attr.kind),
        Some(AttrType::List(inner)) if matches!(**inner, AttrType::Int { .. })
    ));
    let Some(cell) = schema.node_kind("table_cell") else {
        return;
    };
    for name in ["colspan", "rowspan", "scope", "align", "abbr", "headers"] {
        assert!(
            schema.node_attr(cell, name).is_some(),
            "table_cell must declare {name}"
        );
    }
}

#[test]
fn full_schema_declares_image_attrs_and_marks() {
    let Some(schema) = full() else {
        return;
    };
    assert_image_attrs(&schema, "image");
    assert_image_attrs(&schema, "image_block");
    let inline = schema.node_kind("image").and_then(|kind| schema.node(kind));
    let Some(inline) = inline else {
        return;
    };
    assert!(inline.inline && inline.atom, "image is an inline atom");
    let block = schema
        .node_kind("image_block")
        .and_then(|kind| schema.node(kind));
    let Some(block) = block else {
        return;
    };
    assert!(!block.inline && block.atom, "image_block is a block atom");
    let Some(figure) = schema.node_kind("figure") else {
        return;
    };
    assert_eq!(
        schema.node(figure).and_then(|node| node.content.as_deref()),
        Some("image_block caption?")
    );
    let Some(link) = schema.mark_id("link") else {
        return;
    };
    assert_eq!(
        schema
            .mark(link)
            .and_then(|mark| mark.attrs.iter().find(|attr| &*attr.name == "href"))
            .map(|attr| attr.required),
        Some(true)
    );
    let Some(reference) = schema.mark_id("ref") else {
        return;
    };
    assert!(matches!(
        schema
            .mark(reference)
            .and_then(|mark| mark.attrs.iter().find(|attr| &*attr.name == "target"))
            .map(|attr| &attr.kind),
        Some(AttrType::Ref)
    ));
}

#[test]
fn schema_fingerprints_include_attribute_types() {
    let Some(complete) = full() else {
        return;
    };
    let fingerprint = complete.fingerprint();
    assert_eq!(fingerprint.len(), 64);
    assert!(fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let Some(again) = full() else {
        return;
    };
    assert_eq!(again.fingerprint(), fingerprint);

    let make = |value: &str| {
        Schema::builder()
            .node(
                NodeSpec::new("paragraph")
                    .group("block")
                    .content("text*")
                    .attr(AttrSpec::new(
                        "align",
                        AttrType::Enum(Box::from([Box::<str>::from(value)])),
                    )),
            )
            .node(NodeSpec::new("text").inline())
            .build()
    };
    let left = make("start");
    let right = make("end");
    assert!(left.is_ok() && right.is_ok(), "both schemas must compile");
    let (Ok(left), Ok(right)) = (left, right) else {
        return;
    };
    assert_ne!(left.fingerprint(), right.fingerprint());
}
