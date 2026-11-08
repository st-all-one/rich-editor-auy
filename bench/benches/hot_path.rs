//! Informative wall-clock scenarios from plan/05 §3.4.

use std::sync::Arc;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, MarkSet, Run, TextBlock};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;
use rich_editor_auy::wire::{ParseOptions, check_json, from_json};

fn document(blocks: usize, text_len: usize) -> Doc {
    let schema = Schema::basic().expect("basic schema");
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

fn bench_apply(c: &mut Criterion) {
    let schema = Arc::new(Schema::basic().expect("basic schema"));
    let doc = document(10_000, 100);
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    transaction.insert_text(Pos(1), "x").expect("step");
    c.bench_function("apply_insert_s2", |b| {
        b.iter(|| {
            let applied = black_box(&state).apply(black_box(transaction.clone()));
            black_box(applied.is_ok())
        });
    });
}

fn bench_s1(c: &mut Criterion) {
    let schema = Arc::new(Schema::basic().expect("basic schema"));
    let doc = document(100, 100);
    let json = doc.to_json_string(&schema);
    let state = EditorState::from_doc(Arc::clone(&schema), doc);
    let mut transaction = state.transaction();
    transaction.insert_text(Pos(1), "x").expect("step");
    c.bench_function("apply_insert_s1", |b| {
        b.iter(|| {
            let applied = black_box(&state).apply(black_box(transaction.clone()));
            black_box(applied.is_ok())
        });
    });
    c.bench_function("from_json_s1", |b| {
        b.iter(|| {
            let parsed = from_json(black_box(&json), &schema, &ParseOptions::default());
            black_box(parsed.is_ok())
        });
    });
}

fn bench_wire(c: &mut Criterion) {
    let schema = Schema::basic().expect("basic schema");
    let doc = document(10_000, 100);
    let json = doc.to_json_string(&schema);
    c.bench_function("to_json_string_s2", |b| {
        b.iter(|| black_box(doc.to_json_string(black_box(&schema))));
    });
    c.bench_function("from_json_s2", |b| {
        b.iter(|| {
            let parsed = from_json(black_box(&json), &schema, &ParseOptions::default());
            black_box(parsed.is_ok())
        });
    });
}

fn bench_a11y(c: &mut Criterion) {
    let schema = Schema::basic().expect("basic schema");
    let doc = document(10_000, 100);
    c.bench_function("a11y_tree_s2", |b| {
        b.iter(|| black_box(doc.a11y_tree(black_box(&schema))));
    });
    c.bench_function("validate_a11y_s2", |b| {
        b.iter(|| black_box(doc.validate_a11y(black_box(&schema))));
    });
}

fn bench_serde(c: &mut Criterion) {
    let schema = Schema::basic().expect("basic schema");
    let doc = document(10_000, 100);
    let json = doc.to_json_string(&schema);
    let value: serde_json::Value = serde_json::from_str(&json).expect("serde parse");
    c.bench_function("serde_json_parse_s2", |b| {
        b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(black_box(&json)).is_ok()));
    });
    c.bench_function("auy_json_parse_s2", |b| {
        b.iter(|| {
            black_box(check_json(black_box(&json), &ParseOptions::default()).is_ok());
        });
    });
    c.bench_function("serde_json_serialize_s2", |b| {
        b.iter(|| black_box(serde_json::to_string(black_box(&value)).ok()));
    });
    c.bench_function("auy_json_serialize_s2", |b| {
        b.iter(|| black_box(doc.to_json_string(black_box(&schema))));
    });
}

criterion_group!(
    benches,
    bench_apply,
    bench_s1,
    bench_wire,
    bench_a11y,
    bench_serde
);
criterion_main!(benches);
