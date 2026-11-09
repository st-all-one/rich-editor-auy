//! Wall-clock hot-path scenarios: edits into S0–S4 documents.

use std::sync::Arc;

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use rich_editor_auy::model::Mark;
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::state::{EditorState, Transaction};
use rich_editor_auy_bench::{Fixture, Scenario, block_content_start};

fn state(fixture: &Fixture) -> EditorState {
    EditorState::from_doc(Arc::clone(&fixture.schema), fixture.doc.clone())
}

fn insert_at(state: &EditorState, at: u32, text: &str) -> Transaction {
    let mut transaction = state.transaction();
    transaction.insert_text(Pos(at), text).expect("insert step");
    transaction
}

fn delete_at(state: &EditorState, from: u32, to: u32) -> Transaction {
    let mut transaction = state.transaction();
    transaction
        .delete(Range::new(Pos(from), Pos(to)))
        .expect("delete step");
    transaction
}

fn mark_at(state: &EditorState, from: u32, to: u32, mark: Mark) -> Transaction {
    let mut transaction = state.transaction();
    transaction
        .add_mark(Range::new(Pos(from), Pos(to)), mark)
        .expect("mark step");
    transaction
}

fn bench_insert(c: &mut Criterion) {
    for scenario in [Scenario::S0, Scenario::S1, Scenario::S2, Scenario::S3] {
        let fixture = Fixture::new(scenario);
        let start = block_content_start(&fixture.doc, 0);
        let end = start.saturating_add(
            fixture
                .doc
                .blocks()
                .get(0)
                .map_or(0, |block| block.size().0.saturating_sub(2)),
        );
        let middle = start.saturating_add(end.saturating_sub(start) / 2);
        for (name, at) in [("start", start), ("middle", middle), ("end", end)] {
            let state = state(&fixture);
            let transaction = insert_at(&state, at, "x");
            c.bench_function(
                &format!("insert_1char_{name}_{}", scenario.label()),
                |b| {
                    b.iter_batched(
                        || (state.clone(), transaction.clone()),
                        |(state, transaction)| black_box(state.apply(transaction).is_ok()),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
}

fn bench_delete(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S1);
    let start = block_content_start(&fixture.doc, 0);
    let state = state(&fixture);
    let one_char = delete_at(&state, start, start.saturating_add(1));

    c.bench_function("delete_1char_s1", |b| {
        b.iter_batched(
            || (state.clone(), one_char.clone()),
            |(state, transaction)| black_box(state.apply(transaction).is_ok()),
            BatchSize::SmallInput,
        );
    });
    // A single `Step::Replace` cannot cross a block boundary, and an S1 block holds 100 bytes,
    // so the 1 KiB deletion runs on the S3 single-block document.
    let large = Fixture::new(Scenario::S3);
    let large_state = EditorState::from_doc(Arc::clone(&large.schema), large.doc.clone());
    let kib = delete_at(&large_state, block_content_start(&large.doc, 0), 1025);
    c.bench_function("delete_1kib_s3", |b| {
        b.iter_batched(
            || (large_state.clone(), kib.clone()),
            |(state, transaction)| black_box(state.apply(transaction).is_ok()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_mark(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S1);
    let Some(strong) = fixture.schema.mark_id("strong").map(Mark::new) else {
        return;
    };
    let start = block_content_start(&fixture.doc, 0);
    let state = state(&fixture);
    let transaction = mark_at(&state, start, start.saturating_add(1024), strong);
    c.bench_function("add_mark_1kib_s1", |b| {
        b.iter_batched(
            || (state.clone(), transaction.clone()),
            |(state, transaction)| black_box(state.apply(transaction).is_ok()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_split_join(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S1);
    let start = block_content_start(&fixture.doc, 0);
    let state = state(&fixture);
    let mut split = state.transaction();
    split.split_block(Pos(start.saturating_add(1))).expect("split");
    c.bench_function("split_block_s1", |b| {
        b.iter_batched(
            || (state.clone(), split.clone()),
            |(state, transaction)| black_box(state.apply(transaction).is_ok()),
            BatchSize::SmallInput,
        );
    });
    let boundary = fixture
        .doc
        .blocks()
        .get(0)
        .map_or(0, |block| block.size().0);
    let mut join = state.transaction();
    join.join_backward(Pos(boundary)).expect("join");
    c.bench_function("join_block_s1", |b| {
        b.iter_batched(
            || (state.clone(), join.clone()),
            |(state, transaction)| black_box(state.apply(transaction).is_ok()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_multi_step(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S1);
    let start = block_content_start(&fixture.doc, 0);
    let state = state(&fixture);
    let mut transaction = state.transaction();
    for index in 0..10_u32 {
        let at = start.saturating_add(index);
        transaction.insert_text(Pos(at), "y").expect("insert step");
    }
    c.bench_function("transaction_10_steps_s1", |b| {
        b.iter_batched(
            || (state.clone(), transaction.clone()),
            |(state, transaction)| black_box(state.apply(transaction).is_ok()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_undo_redo(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S1);
    let start = block_content_start(&fixture.doc, 0);
    let state = state(&fixture);
    let transaction = insert_at(&state, start, "x");
    let Ok(applied) = state.apply(transaction) else {
        return;
    };
    let edited = applied.state;
    let undone = edited.undo().ok().flatten();
    let Some(undone) = undone else {
        return;
    };
    let undone_state = undone.state;
    c.bench_function("undo_s1", |b| {
        b.iter_batched(
            || edited.clone(),
            |state| black_box(state.undo().is_ok()),
            BatchSize::SmallInput,
        );
    });
    c.bench_function("redo_s1", |b| {
        b.iter_batched(
            || undone_state.clone(),
            |state| black_box(state.redo().is_ok()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_owned(c: &mut Criterion) {
    for scenario in [Scenario::S1, Scenario::S2] {
        let fixture = Fixture::new(scenario);
        let start = block_content_start(&fixture.doc, 0);
        let state = state(&fixture);
        let transaction = insert_at(&state, start, "x");
        c.bench_function(&format!("apply_owned_insert_{}", scenario.label()), |b| {
            b.iter_batched(
                || (state.clone(), transaction.clone()),
                |(state, transaction)| black_box(state.apply_owned(transaction).is_ok()),
                BatchSize::SmallInput,
            );
        });
    }
}

fn bench_s4_smoke(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S4);
    let start = block_content_start(&fixture.doc, 0);
    let state = state(&fixture);
    let transaction = insert_at(&state, start, "x");
    c.bench_function("insert_1char_s4", |b| {
        b.iter_batched(
            || (state.clone(), transaction.clone()),
            |(state, transaction)| {
                let applied = state.apply(transaction);
                black_box(applied.is_ok())
            },
            BatchSize::PerIteration,
        );
    });
}

criterion_group! {
    name = hot;
    config = Criterion::default().warm_up_time(std::time::Duration::from_millis(500)).measurement_time(std::time::Duration::from_secs(2));
    targets = bench_insert, bench_delete, bench_mark, bench_split_join, bench_multi_step, bench_undo_redo, bench_owned, bench_s4_smoke
}
criterion_main!(hot);
