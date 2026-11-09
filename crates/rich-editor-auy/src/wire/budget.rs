//! Running decode budget.

use alloc::boxed::Box;

use crate::Limits;
use crate::error::ParseError;

use super::parser::path_string;

/// Running decode budget.
///
/// Counts blocks, inline atoms, and document bytes so a hostile input is rejected before the
/// corresponding blocks are materialized.
#[derive(Default)]
pub(super) struct DecodeBudget {
    nodes: u32,
    bytes: u32,
}

impl DecodeBudget {
    pub(super) fn count_node(
        &mut self,
        limits: &Limits,
        offset: u32,
        path: &[u32],
    ) -> Result<(), ParseError> {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > limits.max_nodes {
            return Err(limit_error(offset, path));
        }
        Ok(())
    }

    /// Accumulates one top-level block size, which already covers every descendant.
    pub(super) fn count_block(
        &mut self,
        size: u32,
        limits: &Limits,
        offset: u32,
        path: &[u32],
    ) -> Result<(), ParseError> {
        self.bytes = self.bytes.saturating_add(size);
        if self.bytes > limits.max_doc_bytes {
            return Err(limit_error(offset, path));
        }
        Ok(())
    }
}

pub(super) fn limit_error(offset: u32, path: &[u32]) -> ParseError {
    ParseError {
        code: Box::from("limit"),
        offset,
        path: path_string(path),
    }
}
