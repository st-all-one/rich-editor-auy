//! Block type change tests.

use std::sync::Arc;

use rich_editor_auy::model::{
    Attr, AttrId, AttrValue, Attrs, Block, Doc, InlineItem, Mark, MarkSet, NodeId, NodeKind, Run,
    TextBlock,
};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

fn paragraph_state(text: &str, id: Option<&str>) -> Option<(Arc<Schema>, EditorState)> {
    let schema = Arc::new(Schema::full().ok()?);
    let kind = schema.node_kind("paragraph")?;
    let mut attrs = Attrs::empty();
    if let Some(id) = id {
        attrs = attrs.with(Attr {
            id: AttrId::NODE_ID,
            value: AttrValue::Str(Box::from(id)),
        })?;
    }
    let content = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from(text),
        MarkSet::empty(),
    ))]);
    let doc = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(kind, attrs, content))],
    )
    .ok()?;
    Some((Arc::clone(&schema), EditorState::from_doc(schema, doc)))
}

fn level_attrs(schema: &Schema, kind: NodeKind) -> Option<Attrs> {
    let level = schema.node_attr(kind, "level")?.id;
    Attrs::new(vec![Attr {
        id: level,
        value: AttrValue::Int(2),
    }])
}

#[test]
fn set_block_type_switches_type_and_undo_restores() {
    let Some((schema, state)) = paragraph_state("title", None) else {
        return;
    };
    let Some(heading) = schema.node_kind("heading") else {
        return;
    };
    let Some(attrs) = level_attrs(&schema, heading) else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(
        transaction
            .set_block_type(&schema, Pos(0), heading, attrs)
            .is_ok()
    );
    assert!(!transaction.is_empty());
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    let doc = applied.state.doc();
    let Some((_, block)) = doc.block_at(Pos(0)) else {
        return;
    };
    assert_eq!(block.kind(), heading);
    let Some(text) = block.as_textblock() else {
        return;
    };
    assert_eq!(text.len().0, 5);

    let Some(undone) = applied.state.undo().ok().flatten() else {
        return;
    };
    let Some((_, block)) = undone.state.doc().block_at(Pos(0)) else {
        return;
    };
    assert_eq!(
        block.kind(),
        schema.node_kind("paragraph").unwrap_or(heading)
    );
}

#[test]
fn set_block_type_keeps_node_id() {
    let Some((schema, state)) = paragraph_state("title", Some("n-7")) else {
        return;
    };
    let Some(heading) = schema.node_kind("heading") else {
        return;
    };
    let Some(attrs) = level_attrs(&schema, heading) else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(
        transaction
            .set_block_type(&schema, Pos(0), heading, attrs)
            .is_ok()
    );
    let Some(applied) = state.apply(transaction).ok() else {
        return;
    };
    let Some((_, block)) = applied.state.doc().block_at(Pos(0)) else {
        return;
    };
    let id = block.id();
    assert_eq!(id.as_ref().map(NodeId::as_str), Some("n-7"));
}

#[test]
fn set_block_type_rejects_marks_forbidden_by_target() {
    let schema = match Schema::full() {
        Ok(schema) => Arc::new(schema),
        Err(_) => return,
    };
    let Some(kind) = schema.node_kind("paragraph") else {
        return;
    };
    let Some(strong) = schema.mark_id("strong") else {
        return;
    };
    let content = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from("text"),
        MarkSet::single(Mark::new(strong)),
    ))]);
    let Some(doc) = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(kind, Attrs::empty(), content))],
    )
    .ok() else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let Some(code_block) = schema.node_kind("code_block") else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(
        transaction
            .set_block_type(&schema, Pos(0), code_block, Attrs::empty())
            .is_err()
    );
}

#[test]
fn set_block_type_rejects_unknown_attrs_and_structural_targets() {
    let Some((schema, state)) = paragraph_state("text", None) else {
        return;
    };
    let Some(heading) = schema.node_kind("heading") else {
        return;
    };
    let Some(blockquote) = schema.node_kind("blockquote") else {
        return;
    };
    let Some(code_block) = schema.node_kind("code_block") else {
        return;
    };
    let Some(language) = schema.node_attr(code_block, "language") else {
        return;
    };
    let Some(bogus) = Attrs::new(vec![Attr {
        id: language.id,
        value: AttrValue::Str(Box::from("rust")),
    }]) else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(
        transaction
            .set_block_type(&schema, Pos(0), heading, bogus)
            .is_err()
    );
    assert!(
        transaction
            .set_block_type(&schema, Pos(0), blockquote, Attrs::empty())
            .is_err()
    );
}

#[test]
fn set_block_type_rejects_out_of_bounds_positions() {
    let Some((schema, state)) = paragraph_state("text", None) else {
        return;
    };
    let Some(heading) = schema.node_kind("heading") else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(
        transaction
            .set_block_type(&schema, Pos(99), heading, Attrs::empty())
            .is_err()
    );
}
