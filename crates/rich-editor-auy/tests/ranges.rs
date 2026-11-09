//! Inverted ranges are invalid, not empty (edge P12 in `plan/brainstorm/02_bordas.md`).
//!
//! `Range::new` is a const constructor and does not check order. Every consumer validates at use:
//! steps and transactions reject `start > end` with `OutOfBounds`, selections normalize inverted
//! anchors, and only `start == end` counts as empty.

use core::fmt::Debug;
use std::sync::Arc;

use rich_editor_auy::error::{EditError, StepError};
use rich_editor_auy::model::{
    AttrId, AttrValue, Attrs, Block, Doc, InlineItem, Mark, MarkSet, Run, TextBlock,
};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{EditorState, Selection};
use rich_editor_auy::step::{Step, apply_step};

const TEXT: &str = "abcde";

/// Block size: opening token + five text bytes + closing token.
const SIZE: u32 = 7;

fn fixture() -> Option<(Arc<Schema>, EditorState)> {
    let schema = Arc::new(Schema::basic().ok()?);
    let paragraph = schema.node_kind("paragraph")?;
    let text = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from(TEXT),
        MarkSet::empty(),
    ))]);
    let block = Block::text(paragraph, Attrs::empty(), text);
    let doc = Doc::from_blocks(Attrs::empty(), vec![Arc::new(block)]).ok()?;
    Some((Arc::clone(&schema), EditorState::from_doc(schema, doc)))
}

fn out_of_bounds<T: Debug>(result: &Result<T, StepError>, position: u32) -> bool {
    matches!(result, Err(StepError::OutOfBounds { position: found, .. }) if *found == position)
}

fn apply_out_of_bounds<T: Debug>(result: &Result<T, EditError>, position: u32) -> bool {
    matches!(
        result,
        Err(EditError::Step(StepError::OutOfBounds { position: found, .. })) if *found == position
    )
}

#[test]
fn range_is_empty_only_when_exact() {
    assert!(Range::new(Pos(3), Pos(3)).is_empty());
    assert!(!Range::new(Pos(5), Pos(2)).is_empty());
    assert_eq!(Range::new(Pos(5), Pos(2)).len(), 0);
}

#[test]
fn selection_range_normalizes_inverted_anchor() {
    let selection = Selection::text(Pos(5), Pos(2));
    assert_eq!(selection.range(), Some(Range::new(Pos(2), Pos(5))));
    assert!(!selection.is_empty());
    assert_eq!(selection.head(), Some(Pos(2)));
    assert_eq!(selection.anchor(), Some(Pos(5)));
}

#[test]
fn document_operations_reject_inverted_ranges() {
    let Some((schema, state)) = fixture() else {
        return;
    };
    let doc = state.doc();
    let inverted = Range::new(Pos(5), Pos(2));
    assert!(out_of_bounds(&doc.slice(inverted), 5));
    assert!(out_of_bounds(&doc.delete_range(&schema, inverted), 5));
}

#[test]
fn steps_reject_inverted_ranges() {
    let Some((schema, state)) = fixture() else {
        return;
    };
    let Some(strong) = schema.mark_id("strong") else {
        return;
    };
    let doc = state.doc();
    let inverted = Range::new(Pos(5), Pos(2));
    let add = Step::AddMark {
        from: inverted.start,
        to: inverted.end,
        mark: Mark::new(strong),
    };
    assert!(out_of_bounds(&apply_step(doc, &schema, &add), 5));
    let remove = Step::RemoveMark {
        from: inverted.start,
        to: inverted.end,
        mark: Mark::new(strong),
    };
    assert!(out_of_bounds(&apply_step(doc, &schema, &remove), 5));
    let mark_attr = Step::SetMarkAttr {
        from: inverted.start,
        to: inverted.end,
        mark: strong,
        attr: AttrId(0),
        value: AttrValue::Bool(true),
    };
    assert!(out_of_bounds(&apply_step(doc, &schema, &mark_attr), 5));
    let inline_attr = Step::SetInlineAttr {
        from: inverted.start,
        to: inverted.end,
        attr: AttrId(0),
        value: AttrValue::Bool(true),
    };
    assert!(out_of_bounds(&apply_step(doc, &schema, &inline_attr), 5));
}

#[test]
fn steps_reject_ranges_past_the_document() {
    let Some((schema, state)) = fixture() else {
        return;
    };
    let Some(strong) = schema.mark_id("strong") else {
        return;
    };
    let add = Step::AddMark {
        from: Pos(1),
        to: Pos(99),
        mark: Mark::new(strong),
    };
    assert!(out_of_bounds(&apply_step(state.doc(), &schema, &add), 99));
    assert_eq!(state.doc().size().0, SIZE);
}

#[test]
fn transactions_reject_inverted_ranges() {
    let Some((schema, state)) = fixture() else {
        return;
    };
    let Some(strong) = schema.mark_id("strong") else {
        return;
    };
    let inverted = Range::new(Pos(5), Pos(2));
    let original = state.doc().to_json_string(&schema);

    let mut transaction = state.transaction();
    assert!(transaction.add_mark(inverted, Mark::new(strong)).is_ok());
    assert!(
        apply_out_of_bounds(&state.apply(transaction), 5),
        "add_mark must reject the inverted range at apply"
    );
    assert_eq!(state.doc().to_json_string(&schema), original);

    let mut transaction = state.transaction();
    assert!(transaction.remove_mark(inverted, Mark::new(strong)).is_ok());
    assert!(apply_out_of_bounds(&state.apply(transaction), 5));

    let mut transaction = state.transaction();
    assert!(transaction.delete(inverted).is_err());
    assert_eq!(transaction.steps().len(), 0);

    let mut transaction = state.transaction();
    assert!(
        transaction
            .set_mark_attr(inverted, strong, AttrId(0), AttrValue::Bool(true))
            .is_ok()
    );
    assert!(apply_out_of_bounds(&state.apply(transaction), 5));
    assert_eq!(state.doc().to_json_string(&schema), original);
}

#[test]
fn empty_mark_ranges_still_store_the_mark() {
    let Some((schema, state)) = fixture() else {
        return;
    };
    let Some(strong) = schema.mark_id("strong") else {
        return;
    };
    let caret = Range::new(Pos(3), Pos(3));
    let original = state.doc().to_json_string(&schema);
    let mut transaction = state.transaction();
    assert!(transaction.add_mark(caret, Mark::new(strong)).is_ok());
    assert_eq!(transaction.steps().len(), 0);
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "empty mark range must apply");
    let Ok(applied) = applied else {
        return;
    };
    assert!(applied.changes.is_empty());
    assert_eq!(applied.state.doc().to_json_string(&schema), original);
    assert!(applied.state.stored_marks().contains(strong));
}

#[test]
fn empty_ranges_are_noops_for_mark_steps() {
    let Some((schema, state)) = fixture() else {
        return;
    };
    let Some(strong) = schema.mark_id("strong") else {
        return;
    };
    let original = state.doc().to_json_string(&schema);
    let step = Step::AddMark {
        from: Pos(3),
        to: Pos(3),
        mark: Mark::new(strong),
    };
    let next = apply_step(state.doc(), &schema, &step);
    assert!(next.is_ok(), "empty mark range must be a no-op");
    let Ok(next) = next else {
        return;
    };
    assert_eq!(next.to_json_string(&schema), original);
}
