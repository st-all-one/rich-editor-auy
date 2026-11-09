//! Mark attribute step tests.

use std::sync::Arc;

use rich_editor_auy::model::{
    Attr, AttrId, AttrValue, Attrs, Block, Doc, InlineItem, Mark, MarkId, MarkSet, Run, TextBlock,
};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

#[allow(
    clippy::type_complexity,
    reason = "fixtures return the schema, the state, and the mark keys used by assertions"
)]
fn link_state(runs: &[(&str, &str)]) -> Option<(Arc<Schema>, EditorState, MarkId, AttrId)> {
    let schema = Arc::new(Schema::full().ok()?);
    let paragraph = schema.node_kind("paragraph")?;
    let link = schema.mark_id("link")?;
    let href = schema
        .mark(link)?
        .attrs
        .iter()
        .find(|attr| &*attr.name == "href")?
        .id;
    let mut items = Vec::new();
    for (text, target) in runs {
        let attrs = Attrs::new(vec![Attr {
            id: href,
            value: AttrValue::Str(Box::from(*target)),
        }])?;
        items.push(InlineItem::Text(Run::new(
            String::from(*text),
            MarkSet::single(Mark { id: link, attrs }),
        )));
    }
    let doc = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(
            paragraph,
            Attrs::empty(),
            TextBlock::new(items),
        ))],
    )
    .ok()?;
    Some((
        Arc::clone(&schema),
        EditorState::from_doc(schema, doc),
        link,
        href,
    ))
}

fn run_attrs(doc: &Doc, mark: MarkId, attr: AttrId) -> Vec<AttrValue> {
    let Some(text) = doc.block(0).and_then(|block| block.as_textblock()) else {
        return Vec::new();
    };
    text.items()
        .iter()
        .filter_map(|item| match item {
            InlineItem::Text(run) if run.marks.contains(mark) => Some(
                run.marks
                    .attrs_for(mark)
                    .and_then(|attrs| attrs.get(attr))
                    .cloned()
                    .unwrap_or(AttrValue::Null),
            ),
            InlineItem::Text(_) | InlineItem::Atom(_) => None,
        })
        .collect()
}

#[test]
fn mark_attribute_set_and_undo() {
    let Some((_schema, state, link, href)) = link_state(&[("text", "a")]) else {
        return;
    };
    let mut transaction = state.transaction();
    let range = Range::new(Pos(1), Pos(5));
    assert!(
        transaction
            .set_mark_attr(range, link, href, AttrValue::Str(Box::from("b")))
            .is_ok(),
        "mark attribute must queue"
    );
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "mark attribute must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(
        run_attrs(applied.state.doc(), link, href),
        vec![AttrValue::Str(Box::from("b"))]
    );
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(
        run_attrs(undone.state.doc(), link, href),
        vec![AttrValue::Str(Box::from("a"))]
    );
}

#[test]
fn mark_attribute_across_mixed_runs_undoes_exactly() {
    let Some((_schema, state, link, href)) = link_state(&[("ab", "a"), ("cd", "b")]) else {
        return;
    };
    let mut transaction = state.transaction();
    let range = Range::new(Pos(1), Pos(5));
    assert!(
        transaction
            .set_mark_attr(range, link, href, AttrValue::Str(Box::from("c")))
            .is_ok()
    );
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "mixed mark attribute must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(
        run_attrs(applied.state.doc(), link, href),
        vec![AttrValue::Str(Box::from("c"))]
    );
    let undone = applied.state.undo();
    assert!(undone.is_ok(), "mixed undo must apply");
    let Ok(Some(undone)) = undone else {
        return;
    };
    assert_eq!(
        run_attrs(undone.state.doc(), link, href),
        vec![
            AttrValue::Str(Box::from("a")),
            AttrValue::Str(Box::from("b"))
        ]
    );
    assert_eq!(undone.state.doc().plain_text(), "abcd");
}
