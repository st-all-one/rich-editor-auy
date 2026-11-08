//! Deterministic, headless rich-text engine.
//!
//! The crate owns the canonical document, schema validation, positions, editing algebra,
//! canonical JSON, and the semantic accessibility tree. It performs no I/O, layout, styling,
//! or platform interaction.
//!
//! # Determinism
//!
//! Every collection that reaches a result is ordered. Hash maps, wall clocks, environment
//! variables, randomness, and floating point are absent from the data path.
//!
//! # Feature surface
//!
//! The MVP core is `no_std` with `alloc`; integrations live in separate crates.

#![no_std]
#![forbid(unsafe_code)]
#![allow(
    clippy::absolute_paths,
    reason = "explicit paths keep module ownership visible"
)]

extern crate alloc;

pub mod a11y;
pub mod command;
pub mod error;
pub mod input;
pub mod math;
pub mod model;
pub mod motion;
pub mod pos;
pub mod schema;
pub mod state;
pub mod step;
pub mod wire;

/// Configurable hard limits for documents and operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_field_names,
    reason = "all limits are maximums; prefix documents the unit"
)]
pub struct Limits {
    /// Maximum canonical document bytes.
    pub max_doc_bytes: u32,
    /// Maximum text bytes in one block.
    pub max_block_text_bytes: u32,
    /// Maximum structural depth.
    pub max_depth: u16,
    /// Maximum node count.
    pub max_nodes: u32,
    /// Maximum attributes per node.
    pub max_attrs_per_node: u8,
    /// Maximum runs per block.
    pub max_runs_per_block: u32,
    /// Maximum attributes per mark.
    pub max_mark_attrs: u8,
    /// Maximum bytes in one attribute string.
    pub max_attr_string_bytes: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_doc_bytes: 64 * 1024 * 1024,
            max_block_text_bytes: 1024 * 1024,
            max_depth: 64,
            max_nodes: 1_000_000,
            max_attrs_per_node: 32,
            max_runs_per_block: 4096,
            max_mark_attrs: 32,
            max_attr_string_bytes: 64 * 1024,
        }
    }
}
