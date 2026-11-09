//! Deterministic instruction gate: `Ir` per operation on S0/S1 documents.
//!
//! `soft_limits` compares `Ir` against the saved baseline and the runner exits non-zero (code 3)
//! when a regression lands, so the gate breaks CI instead of only printing a report. Absolute
//! `hard_limits` are not used because instruction counts differ across designs, not machines.

use iai_callgrind::{Callgrind, EventKind, LibraryBenchmarkConfig};
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use rich_editor_auy::model::Mark;
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::state::{EditorState, Transaction};
use rich_editor_auy::wire::{ParseOptions, from_json as parse_document};
use rich_editor_auy_bench::{Fixture, Scenario, block_content_start};

/// Prepared edit state: fixture, editor state, an insert transaction, and a mark transaction.
type Prepared = (Fixture, EditorState, Transaction, Transaction);

fn setup(scenario: Scenario) -> Prepared {
    let fixture = Fixture::new(scenario);
    let start = block_content_start(&fixture.doc, 0);
    let state = EditorState::from_doc(fixture.schema.clone(), fixture.doc.clone());
    let mut insert = state.transaction();
    insert.insert_text(Pos(start), "x").expect("insert step");
    let mut mark = state.transaction();
    if let Some(strong) = fixture.schema.mark_id("strong").map(Mark::new) {
        mark.add_mark(Range::new(Pos(start), Pos(start.saturating_add(4))), strong)
            .expect("mark step");
    }
    (fixture, state, insert, mark)
}

#[library_benchmark]
#[bench::s0(args = (Scenario::S0,), setup = setup)]
#[bench::s1(args = (Scenario::S1,), setup = setup)]
fn apply_insert(input: Prepared) -> bool {
    let (_, state, insert, _) = input;
    state.apply(insert).is_ok()
}

#[library_benchmark]
#[bench::s0(args = (Scenario::S0,), setup = setup)]
#[bench::s1(args = (Scenario::S1,), setup = setup)]
fn apply_owned_insert(input: Prepared) -> bool {
    let (_, state, insert, _) = input;
    state.apply_owned(insert).is_ok()
}

#[library_benchmark]
#[bench::s1(args = (Scenario::S1,), setup = setup)]
fn apply_add_mark(input: Prepared) -> bool {
    let (_, state, _, mark) = input;
    state.apply(mark).is_ok()
}

#[library_benchmark]
#[bench::s1(args = (Scenario::S1,), setup = setup)]
fn apply_split(input: Prepared) -> bool {
    let (fixture, state, _, _) = input;
    let start = block_content_start(&fixture.doc, 0);
    let mut transaction = state.transaction();
    transaction
        .split_block(Pos(start.saturating_add(1)))
        .expect("split step");
    state.apply(transaction).is_ok()
}

#[library_benchmark]
#[bench::s0(args = (Scenario::S0,), setup = setup)]
#[bench::s1(args = (Scenario::S1,), setup = setup)]
fn parse_json(input: Prepared) -> bool {
    let (fixture, _, _, _) = input;
    parse_document(&fixture.json, &fixture.schema, &ParseOptions::default()).is_ok()
}

#[library_benchmark]
#[bench::s0(args = (Scenario::S0,), setup = setup)]
#[bench::s1(args = (Scenario::S1,), setup = setup)]
fn write_json(input: Prepared) -> usize {
    let (fixture, _, _, _) = input;
    fixture.doc.to_json_string(&fixture.schema).len()
}

library_benchmark_group!(
    name = hot;
    benchmarks = apply_insert, apply_owned_insert, apply_add_mark, apply_split, parse_json, write_json
);

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)]));
    library_benchmark_groups = hot
);
