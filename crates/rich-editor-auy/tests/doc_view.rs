//! `DocView` render projection contract.

use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json, view_json};

#[test]
fn doc_view_exposes_names_ranges_runs_atoms_and_nested_blocks() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let input = r#"{"type":"doc","attrs":{},"content":[
        {"type":"paragraph","attrs":{"align":"center"},"content":[
            {"type":"text","text":"Hi","marks":[{"type":"strong"}]},
            {"type":"image","attrs":{"alt":"pic"}}
        ]},
        {"type":"table","content":[
            {"type":"table_row","content":[
                {"type":"table_cell","attrs":{"colspan":1},"content":[
                    {"type":"paragraph","content":[{"type":"text","text":"A"}]}
                ]}
            ]}
        ]}
    ]}"#;
    let Ok(doc) = from_json(input, &schema, &ParseOptions::default()) else {
        return;
    };
    let Ok(view) = view_json(&doc, 7, &schema) else {
        return;
    };
    for needle in [
        r#""version":7"#,
        r#""kind":"paragraph""#,
        r#""text":"Hi""#,
        r#""range":[0,5]"#,
        r#""align":"center""#,
        r#""marks":[{"type":"strong"}]"#,
        r#""kind":"image""#,
        r#""alt":"pic""#,
        r#""kind":"table""#,
        r#""kind":"table_row""#,
        r#""kind":"table_cell""#,
        r#""range":[5,14]"#,
        r#""range":[8,11]"#,
    ] {
        assert!(view.contains(needle), "view must contain {needle}: {view}");
    }
    assert!(!view.contains("AttrId"), "view exposes names, never ids");
}
