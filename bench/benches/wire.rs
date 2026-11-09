//! Wall-clock wire scenarios: serialize, parse, validate, normalize, and map.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rich_editor_auy::model::TextBlock;
use rich_editor_auy::pos::{Assoc, Pos};
use rich_editor_auy::step::Mapping;
use rich_editor_auy::wire::{ParseOptions, check_json, from_json};
use rich_editor_auy_bench::{Fixture, Scenario};

fn bench_serialize(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S2);
    let mut sink = String::new();
    c.bench_function("to_json_into_s2", |b| {
        b.iter(|| {
            sink.clear();
            let written = black_box(fixture.doc.to_json_into(black_box(&fixture.schema), &mut sink));
            black_box(written.is_ok())
        });
    });
    c.bench_function("to_json_string_s2", |b| {
        b.iter(|| black_box(fixture.doc.to_json_string(black_box(&fixture.schema))));
    });
}

fn bench_parse(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S2);
    c.bench_function("check_json_s2", |b| {
        b.iter(|| {
            black_box(check_json(black_box(&fixture.json), &ParseOptions::default()).is_ok());
        });
    });
    c.bench_function("from_json_s2", |b| {
        b.iter(|| {
            let parsed = from_json(
                black_box(&fixture.json),
                &fixture.schema,
                &ParseOptions::default(),
            );
            black_box(parsed.is_ok())
        });
    });
}

fn bench_validate(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S2);
    c.bench_function("doc_validate_s2", |b| {
        b.iter(|| black_box(fixture.doc.validate(black_box(&fixture.schema)).is_ok()));
    });
    c.bench_function("a11y_tree_s2", |b| {
        b.iter(|| black_box(fixture.doc.a11y_tree(black_box(&fixture.schema))));
    });
    c.bench_function("validate_a11y_s2", |b| {
        b.iter(|| black_box(fixture.doc.validate_a11y(black_box(&fixture.schema))));
    });
}

fn bench_normalize(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S3);
    let Some(block) = fixture.doc.blocks().get(0) else {
        return;
    };
    let Some(text) = block.as_textblock() else {
        return;
    };
    c.bench_function("normalize_block_s3", |b| {
        b.iter_batched(
            || TextBlock::new(text.items().to_vec()),
            |mut block| black_box(block.normalize(4096).is_ok()),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_map_pos(c: &mut Criterion) {
    let fixture = Fixture::new(Scenario::S2);
    let map = Mapping::identity();
    let size = fixture.doc.size().0;
    c.bench_function("map_pos_s2", |b| {
        b.iter(|| black_box(map.map_pos(Pos(size / 2), Assoc::After).is_some()));
    });
}

criterion_group! {
    name = wire;
    config = Criterion::default().warm_up_time(std::time::Duration::from_millis(500)).measurement_time(std::time::Duration::from_secs(2));
    targets = bench_serialize, bench_parse, bench_validate, bench_normalize, bench_map_pos
}
criterion_main!(wire);
