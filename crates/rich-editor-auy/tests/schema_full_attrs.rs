//! Full-schema attribute value and canonical round-trip tests.

mod schema_support;

use rich_editor_auy::model::AttrValue;
use rich_editor_auy::schema::{AttrType, Schema};
use rich_editor_auy::wire::{ParseOptions, from_json};

use schema_support::full;

#[test]
fn attribute_types_validate_recursively() {
    let enum_type = AttrType::Enum(Box::from([
        Box::<str>::from("ltr"),
        Box::<str>::from("rtl"),
    ]));
    assert!(Schema::value_matches(
        &enum_type,
        &AttrValue::Str(Box::from("ltr"))
    ));
    assert!(!Schema::value_matches(
        &enum_type,
        &AttrValue::Str(Box::from("sideways"))
    ));

    let list_type = AttrType::List(Box::new(AttrType::Int { min: 1, max: 10 }));
    let valid_list = AttrValue::Array(Box::from([AttrValue::Int(1), AttrValue::Int(10)]));
    assert!(Schema::value_matches(&list_type, &valid_list));
    let wrong_list = AttrValue::Array(Box::from([AttrValue::Str(Box::from("x"))]));
    assert!(!Schema::value_matches(&list_type, &wrong_list));

    let map_type = AttrType::Map(Box::new(AttrType::Str));
    let built = AttrValue::map(vec![
        (Box::from("b"), AttrValue::Str(Box::from("2"))),
        (Box::from("a"), AttrValue::Str(Box::from("1"))),
    ]);
    let Some(sorted) = built else {
        assert!(built.is_some(), "unique map keys must build");
        return;
    };
    assert!(Schema::value_matches(&map_type, &sorted));
    let unsorted = AttrValue::Map(Box::from([
        (Box::from("b"), AttrValue::Str(Box::from("2"))),
        (Box::from("a"), AttrValue::Str(Box::from("1"))),
    ]));
    assert!(!Schema::value_matches(&map_type, &unsorted));
    let wrong_map = AttrValue::Map(Box::from([(Box::from("a"), AttrValue::Int(1))]));
    assert!(!Schema::value_matches(&map_type, &wrong_map));
    assert!(Schema::value_matches(&AttrType::Any, &wrong_map));
    assert!(
        AttrValue::map(vec![
            (Box::from("a"), AttrValue::Int(1)),
            (Box::from("a"), AttrValue::Int(2)),
        ])
        .is_none()
    );
}

#[test]
fn media_nodes_declare_mime_size_and_intrinsic_dimensions() {
    let Some(schema) = full() else {
        return;
    };
    for name in ["image", "image_block", "video", "audio", "attachment"] {
        let Some(kind) = schema.node_kind(name) else {
            assert!(schema.node_kind(name).is_some(), "{name} must exist");
            continue;
        };
        for attr in [
            "mime",
            "size_bytes",
            "width",
            "height",
            "intrinsic_width",
            "intrinsic_height",
        ] {
            assert!(
                schema.node_attr(kind, attr).is_some(),
                "{name} must declare {attr}"
            );
        }
    }
}

#[test]
fn full_schema_round_trips_document_metadata_and_blocks() {
    let Some(schema) = full() else {
        return;
    };
    let input = concat!(
        r#"{"type":"doc","attrs":{"lang":"pt-BR","dir":"ltr","authors":["Ana","Bruno"],"tags":["import"],"metadata":{"b":"2","a":"1"}},"content":["#,
        r#"{"type":"paragraph","attrs":{"align":"center"},"content":[{"type":"text","text":"Hello "},{"type":"hard_break"},{"type":"image","attrs":{"alt":"logo","src":"logo.png","width":320,"height":200,"loading":"lazy"}}]},"#,
        r#"{"type":"ordered_list","attrs":{"start":3,"reversed":true,"tight":true,"type":"a"},"content":[{"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"item"}]}]}]},"#,
        r#"{"type":"table","attrs":{"summary":"Data","col_widths":[40,60]},"content":[{"type":"table_caption","content":[{"type":"text","text":"Stocks"}]},{"type":"table_row","content":[{"type":"table_header","attrs":{"scope":"col","colspan":1,"headers":[]},"content":[{"type":"paragraph","content":[{"type":"text","text":"Name"}]}]},{"type":"table_cell","attrs":{"align":"end"},"content":[{"type":"paragraph","content":[{"type":"text","text":"10"}]}]}]}]},"#,
        r#"{"type":"figure","content":[{"type":"image_block","attrs":{"alt":"chart"}},{"type":"caption","content":[{"type":"text","text":"Chart"}]}]},"#,
        r#"{"type":"figure","content":[{"type":"image_block","attrs":{"alt":"solo","src":"solo.png"}}]},"#,
        r#"{"type":"raw_code","attrs":{"language":"xml","format":"xml","filename":"a.xml","line_numbers":true},"content":[{"type":"text","text":"<x/>"}]}"#,
        r#"]}"#,
    );
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_ok(), "full document must parse: {parsed:?}");
    let Ok(doc) = parsed else {
        return;
    };
    assert!(doc.validate(&schema).is_ok(), "document must validate");
    let a11y = doc.validate_a11y(&schema);
    assert!(a11y.is_ok(), "document must be accessible: {a11y:?}");
    let canonical = doc.to_json_string(&schema);
    assert!(
        canonical.contains(r#""metadata":{"a":"1","b":"2"}"#),
        "map keys must be sorted: {canonical}"
    );
    let reparsed = from_json(&canonical, &schema, &ParseOptions::default());
    assert!(
        reparsed.is_ok(),
        "canonical output must parse: {reparsed:?}"
    );
    let Ok(reparsed) = reparsed else {
        return;
    };
    assert_eq!(reparsed.to_json_string(&schema), canonical);
}
