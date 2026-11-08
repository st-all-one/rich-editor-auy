//! Memory budget for a typical large document.
#![allow(
    unsafe_code,
    reason = "the counting allocator wraps the system allocator"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::schema::Schema;

struct Counting;

static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method forwards to the system allocator with the same pointer and layout
// contract, and the counters only observe the call.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller guarantees a valid layout; System forwards it unchanged.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
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
            LIVE_BYTES.fetch_add(new_size.saturating_sub(layout.size()), Ordering::Relaxed);
        }
        new_pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller guarantees a valid layout; System forwards it unchanged.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn live() -> usize {
    LIVE_BYTES.load(Ordering::Relaxed)
}

#[test]
fn typical_large_document_stays_within_the_memory_formula() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let paragraph = schema.node_kind("paragraph");
    assert!(paragraph.is_some(), "paragraph must exist");
    let Some(paragraph) = paragraph else {
        return;
    };
    let before = live();
    let mut blocks = Vec::with_capacity(10_000);
    let text_body = "x".repeat(100);
    for _ in 0..10_000 {
        let text = TextBlock::new(vec![InlineItem::Text(Run::new(
            text_body.clone(),
            MarkSet::empty(),
        ))]);
        blocks.push(Arc::new(Block::text(paragraph, Attrs::empty(), text)));
    }
    let doc = Doc::from_blocks(Attrs::empty(), blocks);
    assert!(doc.is_ok(), "large document must build");
    let Ok(doc) = doc else {
        return;
    };
    let used = live().saturating_sub(before);
    let text_bytes = 100 * 10_000;
    let budget = 2 * text_bytes + 64 * 10_000 + 24 * 10_000 + 16 * 10_000 + 32 + 64 * 1024;
    assert!(
        used <= budget,
        "document used {used} bytes, budget {budget}"
    );
    drop(doc);
}
