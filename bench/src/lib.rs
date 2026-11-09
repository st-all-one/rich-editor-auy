//! Deterministic benchmark corpus for `plan/05_desempenho.md` §8.
//!
//! Every fixture is generated from a pure function, so all benches measure the same documents
//! and no random source enters the measurement path.
#![allow(
    clippy::arithmetic_side_effects,
    reason = "benchmark corpus arithmetic on bounded constants"
)]

use std::sync::Arc;

use rich_editor_auy::model::{Attrs, Block, Doc, InlineItem, Mark, MarkSet, Run, TextBlock};
use rich_editor_auy::schema::Schema;

/// The five benchmark scenarios of `plan/05` §3.4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    /// One block, 100 bytes.
    S0,
    /// 100 blocks, 10 KiB, 200 runs.
    S1,
    /// 10 000 blocks, 1 MiB, 5 000 runs.
    S2,
    /// One block, 1 MiB, 256 runs of 4 KiB.
    S3,
    /// 100 000 blocks, 10 MiB smoke corpus.
    S4,
}

impl Scenario {
    /// Returns the scenario label used in benchmark identifiers.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::S0 => "s0",
            Self::S1 => "s1",
            Self::S2 => "s2",
            Self::S3 => "s3",
            Self::S4 => "s4",
        }
    }
}

/// A generated document plus its canonical JSON.
pub struct Fixture {
    /// The schema used to generate and serialize the document.
    pub schema: Arc<Schema>,
    /// The generated document.
    pub doc: Doc,
    /// The canonical JSON of `doc`.
    pub json: String,
}

impl Fixture {
    /// Builds the fixture for one scenario.
    ///
    /// # Panics
    ///
    /// Panics when the basic preset or the generated document is inconsistent, which would be a
    /// defect in the preset or the corpus generator.
    #[must_use]
    pub fn new(scenario: Scenario) -> Self {
        let schema = Arc::new(Schema::basic().expect("basic schema"));
        let doc = match scenario {
            Scenario::S0 => document(&schema, 1, 1, 100),
            Scenario::S1 => document(&schema, 100, 2, 50),
            Scenario::S2 => sparse_document(&schema, 10_000, 2_500, 200),
            Scenario::S3 => document(&schema, 1, 256, 4_096),
            Scenario::S4 => document(&schema, 100_000, 1, 100),
        };
        let json = doc.to_json_string(&schema);
        Self { schema, doc, json }
    }
}

/// Builds a document whose blocks each hold `runs` runs of `run_bytes` bytes.
#[must_use]
pub fn document(schema: &Schema, blocks: usize, runs: usize, run_bytes: usize) -> Doc {
    let paragraph = schema.node_kind("paragraph").expect("paragraph kind");
    let strong = schema.mark_id("strong").expect("strong mark");
    let mut children = Vec::with_capacity(blocks);
    for _ in 0..blocks {
        children.push(Arc::new(Block::text(
            paragraph,
            Attrs::empty(),
            block_text(runs, run_bytes, strong),
        )));
    }
    Doc::from_blocks(Attrs::empty(), children).expect("generated document")
}

/// Builds a document where only the first `populated` blocks hold runs.
#[must_use]
pub fn sparse_document(schema: &Schema, blocks: usize, populated: usize, run_bytes: usize) -> Doc {
    let paragraph = schema.node_kind("paragraph").expect("paragraph kind");
    let strong = schema.mark_id("strong").expect("strong mark");
    let mut children = Vec::with_capacity(blocks);
    for index in 0..blocks {
        let text = if index < populated {
            block_text(2, run_bytes, strong)
        } else {
            TextBlock::empty()
        };
        children.push(Arc::new(Block::text(
            paragraph,
            Attrs::empty(),
            text,
        )));
    }
    Doc::from_blocks(Attrs::empty(), children).expect("generated document")
}

/// Builds a text block with `runs` runs, alternating marks so the block stays canonical.
fn block_text(runs: usize, run_bytes: usize, strong: rich_editor_auy::model::MarkId) -> TextBlock {
    let body = "x".repeat(run_bytes);
    let mut items = Vec::with_capacity(runs);
    for index in 0..runs {
        let marks = if index % 2 == 0 {
            MarkSet::empty()
        } else {
            MarkSet::single(Mark::new(strong))
        };
        items.push(InlineItem::Text(Run::new(body.clone(), marks)));
    }
    TextBlock::new(items)
}

/// Returns the byte offset of the content start of block `index`.
#[must_use]
pub fn block_content_start(doc: &Doc, index: usize) -> u32 {
    let mut offset = 0_u32;
    for (position, block) in doc.blocks().iter().enumerate() {
        if position == index {
            return offset.saturating_add(1);
        }
        offset = offset.saturating_add(block.size().0);
    }
    offset
}
