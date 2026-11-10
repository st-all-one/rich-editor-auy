//! Full-schema accessibility projection tests.

mod schema_support;

use rich_editor_auy::a11y::A11yRole;
use rich_editor_auy::wire::{ParseOptions, from_json};

use schema_support::full;

#[test]
fn raw_code_a11y_exposes_language_and_text() {
    let Some(schema) = full() else {
        return;
    };
    let input = r#"{"type":"doc","content":[{"type":"raw_code","attrs":{"language":"sql","format":"sql","filename":"q.sql"},"content":[{"type":"text","text":"select 1;"}]}]}"#;
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_ok(), "raw_code document must parse: {parsed:?}");
    let Ok(doc) = parsed else {
        return;
    };
    let tree = doc.a11y_tree(&schema);
    let node = tree.root.children.first();
    assert_eq!(node.map(|node| node.role), Some(A11yRole::CodeBlock));
    assert_eq!(
        node.and_then(|node| node.code_language.as_deref()),
        Some("sql")
    );
    assert_eq!(
        node.and_then(|node| node.text.as_deref()),
        Some("select 1;")
    );
    let run = node.and_then(|node| node.children.first());
    assert_eq!(run.map(|run| run.role), Some(A11yRole::Text));
    assert_eq!(run.and_then(|run| run.text.as_deref()), Some("select 1;"));
    assert!(doc.validate_a11y(&schema).is_ok());
}

#[test]
fn figure_uses_block_image_and_optional_caption() {
    let Some(schema) = full() else {
        return;
    };
    let input = concat!(
        r#"{"type":"doc","content":["#,
        r#"{"type":"figure","content":[{"type":"image_block","attrs":{"alt":"chart"}}]},"#,
        r#"{"type":"figure","content":[{"type":"image_block","attrs":{"alt":"gap","src":"gap.png"}},{"type":"caption","content":[{"type":"text","text":"Gap"}]}]}"#,
        r#"]}"#,
    );
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_ok(), "figures must parse: {parsed:?}");
    let Ok(doc) = parsed else {
        return;
    };
    assert!(doc.validate(&schema).is_ok(), "figures must validate");
    let tree = doc.a11y_tree(&schema);
    let figures = tree
        .root
        .children
        .iter()
        .filter(|node| node.role == A11yRole::Figure)
        .collect::<Vec<_>>();
    assert_eq!(figures.len(), 2, "both figures must be projected");
    for figure in figures {
        assert_eq!(
            figure.children.first().map(|node| node.role),
            Some(A11yRole::Image)
        );
    }
}

#[test]
fn list_nodes_project_ordered_flag() {
    let Some(schema) = full() else {
        return;
    };
    let input = r#"{"type":"doc","content":[
        {"type":"ordered_list","content":[{"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"one"}]}]}]},
        {"type":"bullet_list","content":[{"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"two"}]}]}]}
    ]}"#;
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_ok(), "lists must parse: {parsed:?}");
    let Ok(doc) = parsed else {
        return;
    };
    let tree = doc.a11y_tree(&schema);
    let ordered = tree
        .root
        .children
        .iter()
        .find(|node| node.role == A11yRole::List && node.ordered == Some(true));
    assert!(ordered.is_some(), "ordered_list must project ordered=true");
    let bullet = tree
        .root
        .children
        .iter()
        .find(|node| node.role == A11yRole::List && node.ordered == Some(false));
    assert!(bullet.is_some(), "bullet_list must project ordered=false");
}

#[test]
fn table_cells_accept_block_content() {
    let Some(schema) = full() else {
        return;
    };
    let input = r#"{"type":"doc","content":[{"type":"table","content":[{"type":"table_row","content":[{"type":"table_header","content":[{"type":"paragraph","content":[{"type":"text","text":"Head"}]}]},{"type":"table_cell","content":[{"type":"paragraph","content":[{"type":"text","text":"Value"}]}]}]}]}]}"#;
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_ok(), "cells must parse: {parsed:?}");
    let Ok(doc) = parsed else {
        return;
    };
    assert!(doc.validate(&schema).is_ok(), "cells must validate");
    let tree = doc.a11y_tree(&schema);
    let table = tree
        .root
        .children
        .iter()
        .find(|node| node.role == A11yRole::Table);
    assert!(table.is_some(), "table node must exist");
    let Some(table) = table else {
        return;
    };
    let Some(row) = table.children.first() else {
        return;
    };
    assert_eq!(row.role, A11yRole::TableRow);
    let Some(header) = row.children.first() else {
        return;
    };
    assert_eq!(header.role, A11yRole::TableCell { header: true });
    assert_eq!(
        header.children.first().map(|node| node.role),
        Some(A11yRole::Paragraph)
    );
    assert_eq!(header.name.as_deref(), Some("Head"));
    let Some(cell) = row.children.get(1) else {
        return;
    };
    assert_eq!(cell.role, A11yRole::TableCell { header: false });
    assert_eq!(
        cell.children.first().map(|node| node.role),
        Some(A11yRole::Paragraph)
    );
    assert_eq!(cell.name.as_deref(), Some("Value"));
}

#[test]
fn table_with_header_is_not_reported_as_headerless() {
    let Some(schema) = full() else {
        return;
    };
    let input = r#"{"type":"doc","content":[{"type":"table","content":[
        {"type":"table_caption","content":[{"type":"text","text":"T"}]},
        {"type":"table_row","content":[
            {"type":"table_header","attrs":{"scope":"col"},"content":[{"type":"paragraph","content":[{"type":"text","text":"H"}]}]},
            {"type":"table_cell","content":[{"type":"paragraph","content":[{"type":"text","text":"V"}]}]}
        ]}
    ]}]}"#;
    let parsed = from_json(input, &schema, &ParseOptions::default());
    assert!(parsed.is_ok(), "table must parse: {parsed:?}");
    let Ok(doc) = parsed else {
        return;
    };
    let report = doc.validate_a11y(&schema);
    assert!(
        report
            .warnings
            .iter()
            .all(|issue| issue.code.as_str() != "A006"),
        "table with header must not emit A006: {report:?}"
    );
}
