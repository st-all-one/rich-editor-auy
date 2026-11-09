//! Shared helpers for state integration tests.
#![allow(clippy::redundant_pub_crate, reason = "helpers stay crate-scoped")]

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, GroupId, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Applied, CompositionEvent, EditorState, Selection};

/// Builds a state with one paragraph containing `text`.
#[must_use]
pub(crate) fn fixture(text: &str) -> Option<(Arc<Schema>, EditorState)> {
    let schema = Arc::new(Schema::basic().ok()?);
    let kind = schema.node_kind("paragraph")?;
    let mut content = TextBlock::empty();
    content.push(InlineItem::Text(Run::new(
        String::from(text),
        MarkSet::empty(),
    )));
    let block = Block::text(kind, Attrs::empty(), content);
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]).ok()?;
    Some((Arc::clone(&schema), EditorState::from_doc(schema, doc)))
}

/// Inserts text, optionally inside a host undo group.
#[must_use]
pub(crate) fn insert_at(
    state: &EditorState,
    at: Pos,
    text: &str,
    group: Option<GroupId>,
) -> Option<Applied> {
    let mut transaction = state.transaction();
    transaction.insert_text(at, text).ok()?;
    if let Some(group) = group {
        transaction.set_group(group);
    }
    state.apply(transaction).ok()
}

/// Applies an explicit selection through an empty transaction.
#[must_use]
pub(crate) fn with_selection(state: &EditorState, selection: Selection) -> Option<EditorState> {
    let mut transaction = state.transaction();
    transaction.set_selection(selection);
    state.apply(transaction).ok().map(|applied| applied.state)
}

/// Applies one composition update.
#[must_use]
pub(crate) fn update(state: &EditorState, text: &str) -> Option<Applied> {
    state
        .apply_composition(CompositionEvent::Update {
            text: Box::from(text),
        })
        .ok()
}
