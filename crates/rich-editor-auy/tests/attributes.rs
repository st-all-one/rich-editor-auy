//! Attribute transaction tests.

use std::sync::Arc;

use rich_editor_auy::model::{AttrValue, Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

#[test]
fn undo_removes_a_document_attribute_added_by_a_transaction() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let schema = Arc::new(schema);
    let lang = schema
        .doc()
        .attrs
        .iter()
        .find(|attr| &*attr.name == "lang")
        .map(|attr| attr.id);
    let paragraph = schema.node_kind("paragraph");
    assert!(lang.is_some() && paragraph.is_some());
    let (Some(lang), Some(paragraph)) = (lang, paragraph) else {
        return;
    };
    let block = Block::text(
        paragraph,
        Attrs::empty(),
        TextBlock::new(vec![InlineItem::Text(Run::new(
            String::from("x"),
            MarkSet::empty(),
        ))]),
    );
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]);
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    assert!(
        transaction
            .set_doc_attr(lang, AttrValue::Str(String::from("pt").into_boxed_str()))
            .is_ok()
    );
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "attribute must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert!(applied.state.doc().attrs().get(lang).is_some());
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert!(undone.state.doc().attrs().get(lang).is_none());
}
