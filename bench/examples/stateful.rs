#![allow(clippy::disallowed_methods, reason = "benchmark harness")]

//! A/B wall-clock harness: legacy JSON-per-operation path vs stateful `Editor`.
//!
//! Run: cargo run --release --manifest-path bench/Cargo.toml --example stateful

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use rich_editor_auy::model::{Attrs, Block, Doc, Fragment, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor, EditorState};
use rich_editor_auy::wire::{ParseOptions, from_json};

fn document(blocks: usize, text_len: usize, schema: &Schema) -> Doc {
    let paragraph = schema.node_kind("paragraph").expect("paragraph");
    let body = "x".repeat(text_len);
    let mut children = Vec::with_capacity(blocks);
    for _ in 0..blocks {
        let text = TextBlock::new(vec![InlineItem::Text(Run::new(
            body.clone(),
            MarkSet::empty(),
        ))]);
        children.push(Arc::new(Block::text(paragraph, Attrs::empty(), text)));
    }
    Doc::from_blocks(Attrs::empty(), children).expect("document")
}

fn measure(label: &str, iterations: usize, mut op: impl FnMut()) -> f64 {
    for _ in 0..3 {
        op();
    }
    let start = Instant::now();
    for _ in 0..iterations {
        op();
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let per_op = elapsed / iterations as f64;
    println!("{label:<38} {per_op:>10.4} ms/op ({iterations} ops)");
    per_op
}

fn main() {
    let schema = Arc::new(Schema::full().expect("full schema"));
    let doc = document(1_000, 100, &schema);
    let json = doc.to_json_string(&schema);
    let options = ParseOptions::default();
    println!(
        "document: {} blocks, {} bytes, schema fingerprint {}",
        doc.blocks().len(),
        json.len(),
        &schema.fingerprint()[..12]
    );

    let legacy = measure("legacy parse+apply+serialize", 200, || {
        let parsed = from_json(black_box(&json), &schema, &options).expect("parse");
        let state = EditorState::from_doc(Arc::clone(&schema), parsed);
        let mut transaction = state.transaction();
        transaction.insert_text(Pos(1), "x").expect("insert");
        let applied = state.apply(transaction).expect("apply");
        black_box(applied.state.doc().to_json_string(&schema));
    });

    let mut stateful_editor = Editor::from_json(&json).expect("editor");
    let stateful = measure("stateful apply (no JSON)", 2_000, || {
        let mut transaction = stateful_editor.transaction();
        transaction.insert_text(Pos(1), "x").expect("insert");
        let info = stateful_editor.apply(transaction).expect("apply");
        black_box(info.version);
    });

    let serialized = measure("stateful serialize once", 100, || {
        black_box(stateful_editor.doc().to_json_string(&schema));
    });

    let import = measure("import document (from_json)", 200, || {
        black_box(Editor::from_json(black_box(&json)).expect("editor"));
    });

    let mut editor = Editor::from_json(&json).expect("editor");
    let fragment = Fragment::new(editor.doc().blocks().to_vec());
    let fragment_import = measure("import fragment at 0", 200, || {
        editor
            .replace_fragment(Range::new(Pos(0), Pos(0)), fragment.clone())
            .expect("fragment");
    });

    println!();
    println!("speedup apply: {:.2}x", legacy / stateful);
    println!("speedup full roundtrip: {:.2}x", legacy / serialized);
    println!("document import: {import:.3} ms; fragment import: {fragment_import:.3} ms");
}
