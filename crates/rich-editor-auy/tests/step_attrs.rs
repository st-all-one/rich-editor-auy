//! Document replacement and inline atom attribute step tests.

use std::sync::Arc;

use rich_editor_auy::model::{
    Attr, AttrId, AttrValue, Attrs, Block, Doc, InlineAtom, InlineItem, MarkSet, Run, TextBlock,
};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

fn paragraph_doc(schema: &Schema, text: &str) -> Option<Doc> {
    let kind = schema.node_kind("paragraph")?;
    let content = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from(text),
        MarkSet::empty(),
    ))]);
    Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(kind, Attrs::empty(), content))],
    )
    .ok()
}

#[allow(
    clippy::type_complexity,
    reason = "fixtures return the schema and the state it owns"
)]
fn paragraph_state(text: &str) -> Option<(Arc<Schema>, EditorState)> {
    let schema = Arc::new(Schema::full().ok()?);
    let doc = paragraph_doc(&schema, text)?;
    Some((Arc::clone(&schema), EditorState::from_doc(schema, doc)))
}

#[allow(
    clippy::type_complexity,
    reason = "fixtures return the schema, the state, and the atom attribute key"
)]
fn image_state() -> Option<(Arc<Schema>, EditorState, AttrId)> {
    let schema = Arc::new(Schema::full().ok()?);
    let paragraph = schema.node_kind("paragraph")?;
    let image = schema.node_kind("image")?;
    let alt = schema.node_attr(image, "alt")?.id;
    let width = schema.node_attr(image, "width")?.id;
    let attrs = Attrs::new(vec![Attr {
        id: alt,
        value: AttrValue::Str(Box::from("logo")),
    }])?;
    let content = TextBlock::new(vec![InlineItem::Atom(InlineAtom { kind: image, attrs })]);
    let doc = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(paragraph, Attrs::empty(), content))],
    )
    .ok()?;
    Some((
        Arc::clone(&schema),
        EditorState::from_doc(schema, doc),
        width,
    ))
}

fn image_attr(doc: &Doc, attr: AttrId) -> Option<AttrValue> {
    let block = doc.block(0)?;
    let text = block.as_textblock()?;
    let item = text.items().first()?;
    let InlineItem::Atom(atom) = item else {
        return None;
    };
    atom.attrs.get(attr).cloned()
}

#[test]
fn replace_document_applies_inverts_and_maps() {
    let Some((schema, state)) = paragraph_state("old") else {
        return;
    };
    let Some(doc) = paragraph_doc(&schema, "newer") else {
        return;
    };
    let mut transaction = state.transaction();
    assert!(
        transaction.replace_document(doc).is_ok(),
        "replacement must queue"
    );
    let Some(step) = transaction.steps().first() else {
        return;
    };
    let mapped = step
        .step_map()
        .ranges()
        .first()
        .map(|range| (range.start, range.old_len, range.new_len));
    assert_eq!(mapped, Some((0, 5, 7)));
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "document replacement must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "newer");
    assert!(applied.changes.full, "whole-document change must be full");
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(undone.state.doc().plain_text(), "old");
    let redone = undone.state.redo();
    assert!(redone.is_ok(), "redo must apply");
    let Ok(Some(redone)) = redone else {
        return;
    };
    assert_eq!(redone.state.doc().plain_text(), "newer");
}

#[test]
fn inline_image_attribute_set_and_undo() {
    let Some((_schema, state, width)) = image_state() else {
        return;
    };
    let mut transaction = state.transaction();
    let range = Range::new(Pos(1), Pos(2));
    assert!(
        transaction
            .set_inline_attr(range, width, AttrValue::Int(320))
            .is_ok(),
        "inline attribute must queue"
    );
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "inline attribute must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(
        image_attr(applied.state.doc(), width),
        Some(AttrValue::Int(320))
    );
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(image_attr(undone.state.doc(), width), None);
}

#[test]
fn inline_attr_rejects_ranges_that_are_not_one_atom() {
    let Some((_schema, state, width)) = image_state() else {
        return;
    };
    let mut structural = state.transaction();
    assert!(
        structural
            .set_inline_attr(Range::new(Pos(0), Pos(1)), width, AttrValue::Int(1))
            .is_ok()
    );
    assert!(
        state.apply(structural).is_err(),
        "structural tokens are not atoms"
    );
    let mut wide = state.transaction();
    assert!(
        wide.set_inline_attr(Range::new(Pos(1), Pos(3)), width, AttrValue::Int(1))
            .is_ok()
    );
    assert!(state.apply(wide).is_err(), "ranges must be one byte wide");
    let mut typed = state.transaction();
    assert!(
        typed
            .set_inline_attr(
                Range::new(Pos(1), Pos(2)),
                width,
                AttrValue::Str(Box::from("x"))
            )
            .is_ok()
    );
    assert!(
        state.apply(typed).is_err(),
        "the declared width type is an integer"
    );
    let Some((_schema, text_state)) = paragraph_state("x") else {
        return;
    };
    let mut not_atom = text_state.transaction();
    assert!(
        not_atom
            .set_inline_attr(Range::new(Pos(1), Pos(2)), width, AttrValue::Int(1))
            .is_ok()
    );
    assert!(
        text_state.apply(not_atom).is_err(),
        "text runs are not inline atoms"
    );
}
