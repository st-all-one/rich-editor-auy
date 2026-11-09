//! Table location and snapshot rebuilding.

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Block, Doc};

use super::{TableKinds, unknown};

/// Locates a table block by document-order ordinal, returning its top-level index and child path.
pub(super) fn locate(
    doc: &Doc,
    kinds: &TableKinds,
    target: u32,
) -> Result<(usize, Vec<usize>), StepError> {
    let mut seen = 0_u32;
    for (index, block) in doc.blocks().iter().enumerate() {
        if let Some(path) = find(block, kinds, target, &mut seen) {
            return Ok((index, path));
        }
    }
    Err(unknown("table"))
}

/// Depth-first search for one table ordinal; tables are not searched inside tables.
fn find(block: &Arc<Block>, kinds: &TableKinds, target: u32, seen: &mut u32) -> Option<Vec<usize>> {
    if block.kind() == kinds.table {
        if *seen == target {
            return Some(Vec::new());
        }
        *seen = seen.saturating_add(1);
        return None;
    }
    for (index, child) in block.children()?.iter().enumerate() {
        if let Some(mut path) = find(child, kinds, target, seen) {
            path.insert(0, index);
            return Some(path);
        }
    }
    None
}

/// Returns the block at a top-level index and child path.
pub(super) fn block_at<'a>(doc: &'a Doc, top: usize, path: &[usize]) -> Option<&'a Arc<Block>> {
    let mut block = doc.blocks().get(top)?;
    for index in path {
        block = block.children()?.get(*index)?;
    }
    Some(block)
}

/// Replaces one table block and returns the new document snapshot.
pub(super) fn rebuild(
    doc: &Doc,
    top: usize,
    path: &[usize],
    table: Arc<Block>,
) -> Result<Doc, StepError> {
    let mut blocks = doc.blocks().to_vec();
    let Some(slot) = blocks.get_mut(top) else {
        return Err(unknown("table"));
    };
    let Some(replaced) = replace_at(slot, path, table) else {
        return Err(unknown("table"));
    };
    *slot = replaced;
    Doc::from_blocks(doc.attrs().clone(), blocks).map_err(StepError::Limit)
}

/// Replaces a nested block, rebuilding every ancestor container.
fn replace_at(block: &Arc<Block>, path: &[usize], table: Arc<Block>) -> Option<Arc<Block>> {
    let Some((first, rest)) = path.split_first() else {
        return Some(table);
    };
    let children = block.children()?;
    let child = children.get(*first)?;
    let replaced = replace_at(child, rest, table)?;
    let mut rebuilt = children.to_vec();
    let slot = rebuilt.get_mut(*first)?;
    *slot = replaced;
    Some(Arc::new(Block::container(
        block.kind(),
        block.attrs().clone(),
        rebuilt,
    )))
}

/// Counts tables before a top-level block index.
pub(super) fn table_ordinal(doc: &Doc, kinds: &TableKinds, before: usize) -> u32 {
    let mut total = 0_u32;
    for block in doc.blocks().iter().take(before) {
        total = total.saturating_add(count_tables(block, kinds));
    }
    total
}

/// Counts one table occurrence without descending into table content.
fn count_tables(block: &Block, kinds: &TableKinds) -> u32 {
    if block.kind() == kinds.table {
        return 1;
    }
    let mut total = 0_u32;
    if let Some(children) = block.children() {
        for child in children {
            total = total.saturating_add(count_tables(child, kinds));
        }
    }
    total
}
