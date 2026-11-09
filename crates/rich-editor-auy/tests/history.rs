//! History grouping, limits, and IME composition semantics.

mod common;

use common::{fixture, insert_at, update, with_selection};
use rich_editor_auy::model::GroupId;
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::state::{CompositionEvent, Origin, Selection, Transaction};

#[test]
fn history_info_tracks_depths_and_clear() {
    let Some((_, initial)) = fixture("") else {
        return;
    };
    let Some(first) = insert_at(&initial, Pos(1), "a", None) else {
        return;
    };
    let Some(second) = insert_at(&first.state, Pos(2), "b", None) else {
        return;
    };
    let Some(third) = insert_at(&second.state, Pos(3), "c", None) else {
        return;
    };
    let info = third.state.history_info();
    assert_eq!(info.undo_depth, 3);
    assert!(info.can_undo && !info.can_redo);
    assert_eq!(info.last_origin, Some(Origin::User));
    assert!(info.bytes > 0, "history must report retained bytes");
    let Some(undone) = third.state.undo().ok().flatten() else {
        return;
    };
    let info = undone.state.history_info();
    assert_eq!((info.undo_depth, info.redo_depth), (2, 1));
    assert!(info.can_redo);
    let cleared = undone.state.clear_history();
    let info = cleared.history_info();
    assert_eq!((info.undo_depth, info.redo_depth, info.bytes), (0, 0, 0));
    assert!(!info.can_undo && !info.can_redo);
}

#[test]
fn history_limits_trim_oldest_groups_and_bytes() {
    let Some((_, initial)) = fixture("") else {
        return;
    };
    let limited = initial.with_history_limits(2, usize::MAX);
    assert_eq!(limited.history_limits(), (2, usize::MAX));
    let Some(first) = insert_at(&limited, Pos(1), "a", None) else {
        return;
    };
    let Some(second) = insert_at(&first.state, Pos(2), "b", None) else {
        return;
    };
    let Some(third) = insert_at(&second.state, Pos(3), "c", None) else {
        return;
    };
    assert_eq!(third.state.history_info().undo_depth, 2);
    let nested = third.state.with_history_limits(1, usize::MAX);
    assert_eq!(nested.history_info().undo_depth, 1);
    let Some(back) = third.state.undo().ok().flatten() else {
        return;
    };
    assert_eq!(back.state.doc().plain_text(), "ab");
    let Some(back) = back.state.undo().ok().flatten() else {
        return;
    };
    assert_eq!(back.state.doc().plain_text(), "a");
    assert!(!back.state.can_undo(), "trimmed groups must not return");

    let bytes_limited = initial.with_history_limits(10, 6);
    let Some(first) = insert_at(&bytes_limited, Pos(1), "abcde", None) else {
        return;
    };
    let Some(second) = insert_at(&first.state, Pos(6), "fghij", None) else {
        return;
    };
    assert_eq!(second.state.history_info().undo_depth, 1);
    let Some(back) = second.state.undo().ok().flatten() else {
        return;
    };
    assert_eq!(back.state.doc().plain_text(), "abcde");
}

#[test]
fn consecutive_groups_merge_into_one_undo_entry() {
    let Some((_, initial)) = fixture("") else {
        return;
    };
    let Some(first) = insert_at(&initial, Pos(1), "a", Some(GroupId(7))) else {
        return;
    };
    let Some(second) = insert_at(&first.state, Pos(2), "b", Some(GroupId(7))) else {
        return;
    };
    assert_eq!(second.state.history_info().undo_depth, 1);
    let Some(undone) = second.state.undo().ok().flatten() else {
        return;
    };
    assert_eq!(undone.state.doc().plain_text(), "");
    let Some(redone) = undone.state.redo().ok().flatten() else {
        return;
    };
    assert_eq!(redone.state.doc().plain_text(), "ab");
    let Some(other) = insert_at(&second.state, Pos(3), "c", Some(GroupId(8))) else {
        return;
    };
    assert_eq!(other.state.history_info().undo_depth, 2);
    assert_eq!(other.state.doc().plain_text(), "abc");
}

#[test]
fn undo_and_redo_restore_selections() {
    let Some((_, initial)) = fixture("abc") else {
        return;
    };
    let Some(selected) = with_selection(&initial, Selection::caret(Pos(1))) else {
        return;
    };
    let mut transaction = selected.transaction();
    assert!(transaction.insert_text(Pos(1), "Z").is_ok());
    transaction.set_selection(Selection::caret(Pos(2)));
    let Some(applied) = selected.apply(transaction).ok() else {
        return;
    };
    assert_eq!(applied.state.selection(), Selection::caret(Pos(2)));
    let Some(undone) = applied.state.undo().ok().flatten() else {
        return;
    };
    assert_eq!(undone.state.doc().plain_text(), "abc");
    assert_eq!(undone.state.selection(), Selection::caret(Pos(1)));
    let Some(redone) = undone.state.redo().ok().flatten() else {
        return;
    };
    assert_eq!(redone.state.doc().plain_text(), "Zabc");
    assert_eq!(redone.state.selection(), Selection::caret(Pos(2)));
}

#[test]
#[allow(clippy::too_many_lines, reason = "one composition lifecycle")]
fn composition_commit_records_one_undo_entry() {
    let Some((_, initial)) = fixture("hello") else {
        return;
    };
    let Some(selected) = with_selection(&initial, Selection::text(Pos(2), Pos(5))) else {
        return;
    };
    let at = Range::new(Pos(2), Pos(5));
    let Some(begun) = selected
        .apply_composition(CompositionEvent::Start { at })
        .ok()
    else {
        return;
    };
    let Some(id) = begun.state.composition_id() else {
        return;
    };
    assert!(begun.state.is_composing());
    assert_eq!(begun.state.history_info().undo_depth, 0);
    let Some(first) = update(&begun.state, "X") else {
        return;
    };
    assert_eq!(first.state.doc().plain_text(), "hXo");
    assert_eq!(first.state.selection(), Selection::caret(Pos(3)));
    assert_eq!(first.state.history_info().undo_depth, 0);
    let Some(second) = update(&first.state, "YZ") else {
        return;
    };
    assert_eq!(second.state.doc().plain_text(), "hYZo");
    assert_eq!(second.state.history_info().undo_depth, 0);
    let Some(committed) = second.state.commit_composition().ok() else {
        return;
    };
    assert!(!committed.state.is_composing());
    let info = committed.state.history_info();
    assert_eq!(info.undo_depth, 1);
    assert_eq!(info.last_origin, Some(Origin::Composition(id)));
    assert_eq!(committed.state.selection(), Selection::caret(Pos(4)));
    let Some(undone) = committed.state.undo().ok().flatten() else {
        return;
    };
    assert_eq!(undone.state.doc().plain_text(), "hello");
    assert_eq!(undone.state.selection(), Selection::text(Pos(2), Pos(5)));
    let Some(redone) = undone.state.redo().ok().flatten() else {
        return;
    };
    assert_eq!(redone.state.doc().plain_text(), "hYZo");
    assert_eq!(redone.state.selection(), Selection::caret(Pos(4)));
}

#[test]
fn composition_cancel_restores_original_text_and_selection() {
    let Some((_, initial)) = fixture("hello") else {
        return;
    };
    let Some(selected) = with_selection(&initial, Selection::text(Pos(2), Pos(5))) else {
        return;
    };
    let at = Range::new(Pos(2), Pos(5));
    let Some(begun) = selected
        .apply_composition(CompositionEvent::Start { at })
        .ok()
    else {
        return;
    };
    let Some(first) = update(&begun.state, "X") else {
        return;
    };
    let Some(second) = update(&first.state, "YZ") else {
        return;
    };
    assert_eq!(second.state.doc().plain_text(), "hYZo");
    let Some(cancelled) = second.state.cancel_composition().ok() else {
        return;
    };
    assert_eq!(cancelled.state.doc().plain_text(), "hello");
    assert_eq!(cancelled.state.selection(), Selection::text(Pos(2), Pos(5)));
    assert!(!cancelled.state.is_composing());
    assert!(!cancelled.state.can_undo());
    assert_eq!(cancelled.state.history_info().undo_depth, 0);
}

#[test]
fn composition_requires_an_active_session() {
    let Some((_, state)) = fixture("hi") else {
        return;
    };
    assert!(state.update_composition("x").is_err());
    assert!(state.commit_composition().is_err());
    assert!(state.cancel_composition().is_err());
    let at = Range::new(Pos(1), Pos(1));
    let Some(begun) = state.begin_composition(at).ok() else {
        return;
    };
    assert!(begun.state.begin_composition(at).is_err());
    assert!(state.begin_composition(Range::new(Pos(0), Pos(1))).is_err());
}

#[test]
fn transaction_composition_commit_and_cancel() {
    let Some((_, initial)) = fixture("hello") else {
        return;
    };
    let at = Range::new(Pos(2), Pos(5));
    let mut transaction: Transaction = initial.transaction();
    assert!(
        transaction
            .apply_composition(CompositionEvent::Start { at })
            .is_ok()
    );
    assert!(
        transaction
            .apply_composition(CompositionEvent::Update {
                text: Box::from("X"),
            })
            .is_ok()
    );
    assert!(
        transaction
            .apply_composition(CompositionEvent::Commit)
            .is_ok()
    );
    let Some(committed) = initial.apply(transaction).ok() else {
        return;
    };
    assert_eq!(committed.state.doc().plain_text(), "hXo");
    assert_eq!(committed.state.history_info().undo_depth, 1);
    let Some(undone) = committed.state.undo().ok().flatten() else {
        return;
    };
    assert_eq!(undone.state.doc().plain_text(), "hello");

    let mut transaction = initial.transaction();
    assert!(
        transaction
            .apply_composition(CompositionEvent::Start { at })
            .is_ok()
    );
    assert!(
        transaction
            .apply_composition(CompositionEvent::Update {
                text: Box::from("XYZ"),
            })
            .is_ok()
    );
    assert!(
        transaction
            .apply_composition(CompositionEvent::Cancel)
            .is_ok()
    );
    let Some(cancelled) = initial.apply(transaction).ok() else {
        return;
    };
    assert_eq!(cancelled.state.doc().plain_text(), "hello");
    assert_eq!(cancelled.state.history_info().undo_depth, 0);
}
