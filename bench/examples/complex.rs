#![allow(clippy::disallowed_methods, clippy::unit_arg, reason = "benchmark harness")]

//! Wall-clock harness for a mixed rich document.
//!
//! Run with: cargo run --release --manifest-path bench/Cargo.toml --example complex -- /tmp/complex.json

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;
use rich_editor_auy::wire::{ParseOptions, check_json, from_json};

fn measure(label: &str, mut op: impl FnMut()) {
    for _ in 0..5 {
        op();
    }
    let start = Instant::now();
    for _ in 0..100 {
        op();
    }
    let elapsed = start.elapsed();
    println!("{label}: {:.3} ms", elapsed.as_secs_f64() * 1000.0 / 100.0);
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "/tmp/complex.json".into());
    let input = std::fs::read_to_string(&path).expect("input");
    let schema = Arc::new(Schema::basic().expect("schema"));
    let options = ParseOptions::default();
    let doc = from_json(&input, &schema, &options).expect("parse");
    println!("blocks={} size={}", doc.blocks().len(), doc.size().0);
    measure("AUY check_json", || {
        black_box(check_json(black_box(&input), &options).is_ok());
    });
    measure("AUY from_json", || {
        black_box(from_json(black_box(&input), &schema, &options).expect("parse"));
    });
    measure("AUY to_json_string", || {
        black_box(doc.to_json_string(&schema));
    });
    measure("AUY validate", || {
        black_box(doc.validate(&schema));
    });
    measure("AUY a11y_tree", || {
        black_box(doc.a11y_tree(&schema));
    });
    measure("AUY a11y_validate", || {
        black_box(doc.validate_a11y(&schema));
    });
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    transaction.insert_text(Pos(1), "x").expect("insert");
    measure("AUY apply insert", || {
        black_box(state.apply(transaction.clone()).expect("apply"));
    });
}
