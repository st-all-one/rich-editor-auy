//! Allocation budget for the hot path.
#![allow(
    unsafe_code,
    reason = "the counting allocator wraps the system allocator"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOC_LOCK: Mutex<()> = Mutex::new(());
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method forwards to the system allocator with the same pointer and layout
// contract, and the counters only observe the call.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller guarantees a valid layout; System forwards it unchanged.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: the pointer and layout belong to this allocator.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the pointer and old layout belong to this allocator; new_size is valid.
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if !new_pointer.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            LIVE_BYTES.fetch_add(new_size.saturating_sub(layout.size()), Ordering::Relaxed);
        }
        new_pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller guarantees a valid layout; System forwards it unchanged.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn reset() {
    ALLOCATIONS.store(0, Ordering::Relaxed);
}

fn allocations() -> usize {
    ALLOCATIONS.load(Ordering::Relaxed)
}

#[test]
fn one_character_insertion_has_a_bounded_allocation_count() {
    let _guard = ALLOC_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let schema = Arc::new(schema);
    let paragraph = schema.node_kind("paragraph");
    assert!(paragraph.is_some(), "paragraph must exist");
    let Some(paragraph) = paragraph else {
        return;
    };
    let text = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from("hello"),
        MarkSet::empty(),
    ))]);
    let doc = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(paragraph, Attrs::empty(), text))],
    );
    assert!(doc.is_ok(), "document must build");
    let Ok(doc) = doc else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    assert!(transaction.insert_text(Pos(1), "x").is_ok());
    reset();
    let applied = state.apply(transaction);
    assert!(applied.is_ok(), "insert must apply");
    // Calibrated ceiling for one-character insert into a one-block document. The remaining
    // sites, smallest stable set: the replacement `String`, the `Vec<InlineItem>`, the
    // `TextBlock`, the `Arc<Block>`, the `ChangeSet` range slice, the `Step` vector and the
    // `inverse` vector, the chunk slot for the replaced block, the `DocInner` `Arc`, the
    // `Group` with its copied step vector, and the cloned history stack. Reaching zero would
    // need an arena or `unsafe`, both forbidden in the core.
    assert!(
        allocations() <= 16,
        "hot-path insert allocated {} times",
        allocations()
    );
}

#[test]
fn owned_insert_avoids_directory_copies() {
    let _guard = ALLOC_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let schema = Arc::new(schema);
    let paragraph = schema.node_kind("paragraph");
    let Some(paragraph) = paragraph else {
        return;
    };
    let text = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from("hello"),
        MarkSet::empty(),
    ))]);
    let Ok(doc) = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(paragraph, Attrs::empty(), text))],
    ) else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    assert!(transaction.insert_text(Pos(1), "x").is_ok());
    reset();
    let applied = state.apply_owned(transaction);
    assert!(applied.is_ok(), "owned insert must apply");
    // The owned path drops the history-stack clone and the `Doc` snapshot copy, so it stays
    // strictly below `one_character_insertion_has_a_bounded_allocation_count`.
    assert!(
        allocations() <= 11,
        "owned hot-path insert allocated {} times",
        allocations()
    );
}

fn measure<T>(body: impl FnOnce() -> T) -> (T, usize) {
    reset();
    let value = body();
    (value, allocations())
}

/// Builds a state holding one paragraph with eight recorded undo groups.
fn state_with_groups(schema: &Arc<Schema>) -> Option<EditorState> {
    let paragraph = schema.node_kind("paragraph")?;
    let doc = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(
            paragraph,
            Attrs::empty(),
            TextBlock::empty(),
        ))],
    )
    .ok()?;
    let mut state = EditorState::from_doc(Arc::clone(schema), doc);
    for index in 0..8_u32 {
        let mut transaction = state.transaction();
        let at = Pos(Pos(1).0.saturating_add(index));
        assert!(transaction.insert_text(at, "x").is_ok());
        state = state.apply_owned(transaction).ok()?.state;
    }
    Some(state)
}

#[test]
fn state_clone_does_not_copy_history_groups() {
    let _guard = ALLOC_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let schema = Arc::new(schema);
    let Some(state) = state_with_groups(&schema) else {
        return;
    };
    let (cloned, count) = measure(|| state.clone());
    // A group stack of shared pointers is one allocation; the empty redo stack is none.
    assert!(
        count <= 2,
        "cloning state with 8 groups allocated {count} times"
    );
    assert_eq!(cloned.history_info().undo_depth, 8);
    assert_eq!(state.history_info().undo_depth, 8);
}

#[test]
fn owned_apply_is_no_worse_than_immutable() {
    let _guard = ALLOC_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let schema = Arc::new(schema);
    let paragraph = schema.node_kind("paragraph");
    assert!(paragraph.is_some(), "paragraph must exist");
    let Some(paragraph) = paragraph else {
        return;
    };
    let text = TextBlock::new(vec![InlineItem::Text(Run::new(
        String::from("hello"),
        MarkSet::empty(),
    ))]);
    let Ok(doc) = Doc::from_blocks(
        Attrs::empty(),
        vec![Arc::new(Block::text(paragraph, Attrs::empty(), text))],
    ) else {
        return;
    };
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut immutable_transaction = state.transaction();
    assert!(immutable_transaction.insert_text(Pos(1), "y").is_ok());
    let mut owned_transaction = state.transaction();
    assert!(owned_transaction.insert_text(Pos(1), "y").is_ok());

    let (immutable_applied, immutable) = measure(|| state.apply(immutable_transaction));
    let (owned_applied, owned) = measure(|| state.apply_owned(owned_transaction));
    assert!(immutable_applied.is_ok(), "immutable apply must succeed");
    assert!(owned_applied.is_ok(), "owned apply must succeed");
    assert!(
        owned <= immutable,
        "owned apply allocated {owned} times against {immutable}"
    );

    let Ok(owned_applied) = owned_applied else {
        return;
    };
    let state = owned_applied.state;
    let mut next = state.transaction();
    assert!(next.insert_text(Pos(2), "z").is_ok());
    let applied = state.apply_owned(next);
    assert!(applied.is_ok(), "owned state stays editable");
    let Ok(applied) = applied else {
        return;
    };
    assert_eq!(applied.state.doc().plain_text(), "yzhello");
}
