//! Allocation budget for the hot path.
#![allow(
    unsafe_code,
    reason = "the counting allocator wraps the system allocator"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
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
    assert!(
        allocations() <= 32,
        "hot-path insert allocated {} times",
        allocations()
    );
}
