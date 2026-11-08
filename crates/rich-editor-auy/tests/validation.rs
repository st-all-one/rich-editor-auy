//! Structural invariant tests.

use std::sync::Arc;

use rich_editor_auy::model::{
    Attr, AttrId, AttrValue, Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock,
};
use rich_editor_auy::schema::Schema;

fn paragraph(schema: &Schema, text: &str, attrs: Attrs) -> Option<Arc<Block>> {
    let kind = schema.node_kind("paragraph")?;
    let content = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from(text),
        MarkSet::empty(),
    ))]);
    Some(Arc::new(Block::text(kind, attrs, content)))
}

#[test]
fn validate_rejects_duplicate_node_ids() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let attrs = Attrs::empty()
        .with(Attr {
            id: AttrId::NODE_ID,
            value: AttrValue::Str(String::from("same").into_boxed_str()),
        })
        .unwrap_or_default();
    let first = paragraph(&schema, "a", attrs.clone());
    let second = paragraph(&schema, "b", attrs);
    assert!(first.is_some() && second.is_some(), "blocks must build");
    let (Some(first), Some(second)) = (first, second) else {
        return;
    };
    let doc = Doc::from_blocks(Attrs::empty(), vec![first, second]);
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let report = doc.validate(&schema);
    assert!(!report.is_ok(), "duplicate ids must fail");
    assert!(
        report.issues.iter().any(|issue| issue.invariant == "I16"),
        "expected I16, got {:?}",
        report.issues
    );
}
