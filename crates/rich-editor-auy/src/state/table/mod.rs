//! Whole-table mutation shared by the editor's table commands.
//!
//! Every command locates one table block, rebuilds only that block from grid data, validates the
//! resulting grid, and replaces the document snapshot. Undo therefore restores the exact previous
//! table because the recorded inverse is a whole-document replacement.

mod emit;
mod grid;
mod op;
mod tree;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Attrs, Block, CellRef, Doc, NodeKind, TableInfo, TextBlock};
use crate::schema::Schema;

use grid::Grid;
pub(super) use op::{TableOp, TableShape};

/// Node kinds needed to build and rebuild table blocks.
pub(super) struct TableKinds {
    /// `table` kind.
    pub(super) table: NodeKind,
    /// `table_caption` kind.
    pub(super) caption: NodeKind,
    /// `table_row` kind.
    pub(super) row: NodeKind,
    /// `table_header` kind.
    pub(super) header: NodeKind,
    /// `table_cell` kind.
    pub(super) cell: NodeKind,
    /// `paragraph` kind used for empty cell content.
    pub(super) paragraph: NodeKind,
}

impl TableKinds {
    /// Resolves every table node kind, or `None` when the schema has no tables.
    pub(super) fn resolve(schema: &Schema) -> Option<Self> {
        Some(Self {
            table: schema.node_kind("table")?,
            caption: schema.node_kind("table_caption")?,
            row: schema.node_kind("table_row")?,
            header: schema.node_kind("table_header")?,
            cell: schema.node_kind("table_cell")?,
            paragraph: schema.node_kind("paragraph")?,
        })
    }

    /// Builds one empty paragraph block.
    pub(super) fn empty_paragraph(&self) -> Arc<Block> {
        Arc::new(Block::text(
            self.paragraph,
            Attrs::empty(),
            TextBlock::empty(),
        ))
    }

    /// Builds one empty data cell holding a single empty paragraph.
    pub(super) fn empty_cell(&self) -> Arc<Block> {
        Arc::new(Block::container(
            self.cell,
            Attrs::empty(),
            alloc::vec![self.empty_paragraph()],
        ))
    }

    /// Builds a complete table block with one empty paragraph per cell.
    pub(super) fn table_block(&self, rows: u32, columns: u32, header_rows: u32) -> Arc<Block> {
        let mut children = Vec::new();
        for row in 0..rows {
            let kind = if row < header_rows {
                self.header
            } else {
                self.cell
            };
            let mut cells = Vec::new();
            for _ in 0..columns {
                cells.push(Arc::new(Block::container(
                    kind,
                    Attrs::empty(),
                    alloc::vec![self.empty_paragraph()],
                )));
            }
            children.push(Arc::new(Block::container(self.row, Attrs::empty(), cells)));
        }
        Arc::new(Block::container(self.table, Attrs::empty(), children))
    }
}

/// Result of one table mutation.
pub(super) struct Mutation {
    /// Replacement document.
    pub(super) doc: Doc,
    /// Table ordinal that was rebuilt.
    pub(super) table: u32,
    /// Selection anchor cell.
    pub(super) anchor: CellRef,
    /// Selection head cell.
    pub(super) head: CellRef,
}

/// Applies one structural edit and returns the replacement document plus the new table selection.
///
/// # Errors
///
/// Returns a typed step error when the table, cell, span, or merge rectangle is invalid.
#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive command dispatcher keeps selection rules visible"
)]
pub(super) fn mutate(
    doc: &Doc,
    schema: &Schema,
    kinds: &TableKinds,
    table: u32,
    op: &TableOp<'_>,
) -> Result<Mutation, StepError> {
    let info = doc
        .table_info(schema, table)
        .ok_or_else(|| unknown("table"))?;
    let mut grid = Grid::from_info(&info);
    let (row, column) = match op {
        TableOp::InsertRow { at } => {
            grid.insert_row(kinds, *at)?;
            (*at, 0)
        }
        TableOp::DeleteRow { row } => {
            grid.delete_row(*row)?;
            ((*row).min(grid.rows().saturating_sub(1)), 0)
        }
        TableOp::InsertColumn { at } => {
            grid.insert_column(kinds, *at)?;
            (0, (*at).min(grid.columns().saturating_sub(1)))
        }
        TableOp::DeleteColumn { column } => {
            grid.delete_column(*column)?;
            (0, (*column).min(grid.columns().saturating_sub(1)))
        }
        TableOp::Merge { anchor, head } => {
            let first = resolve(&info, *anchor)?;
            let second = resolve(&info, *head)?;
            grid.merge(kinds, first, second)?
        }
        TableOp::Split { cell } => grid.split(kinds, resolve(&info, *cell)?)?,
        TableOp::SetHeader { cell, header } => grid.set_header(resolve(&info, *cell)?, *header)?,
        TableOp::SetSpan {
            cell,
            colspan,
            rowspan,
        } => grid.set_span(kinds, resolve(&info, *cell)?, *colspan, *rowspan)?,
        TableOp::SetAttr { cell, attr, value } => {
            grid.set_attr(schema, resolve(&info, *cell)?, attr, value.clone())?
        }
    };
    let (top, path) = tree::locate(doc, kinds, table)?;
    let original = tree::block_at(doc, top, &path).ok_or_else(|| unknown("table"))?;
    let block = emit::emit(original, schema, kinds, &grid)?;
    emit::check_table(&block, schema)?;
    let new_doc = tree::rebuild(doc, top, &path, block)?;
    let info = new_doc
        .table_info(schema, table)
        .ok_or_else(|| unknown("table"))?;
    let anchor = info
        .cell_ref(row, column)
        .ok_or_else(|| unknown("table_cell"))?;
    Ok(Mutation {
        doc: new_doc,
        table,
        anchor,
        head: anchor,
    })
}

/// Inserts one freshly built table at a top-level block boundary.
///
/// # Errors
///
/// Returns a typed step error when the position or the table shape is invalid.
pub(super) fn insert_table(
    doc: &Doc,
    schema: &Schema,
    kinds: &TableKinds,
    at: usize,
    shape: &TableShape,
) -> Result<(Doc, u32), StepError> {
    if at > doc.blocks().len() {
        return Err(invalid("table_position"));
    }
    let block = kinds.table_block(shape.rows, shape.columns, shape.header_rows);
    emit::check_table(&block, schema)?;
    let ordinal = tree::table_ordinal(doc, kinds, at);
    let mut blocks = doc.blocks().to_vec();
    blocks.insert(at, block);
    let doc = Doc::from_blocks(doc.attrs().clone(), blocks).map_err(StepError::Limit)?;
    Ok((doc, ordinal))
}

/// Normalizes a cell address to the origin of the merged cell that covers it.
fn resolve(info: &TableInfo<'_>, cell: CellRef) -> Result<CellRef, StepError> {
    info.cell_ref(cell.row, cell.column)
        .ok_or_else(|| unknown("table_cell"))
}

/// Builds a typed error for an unknown table node or cell.
fn unknown(path: &str) -> StepError {
    StepError::Unknown {
        path: Box::from(path),
    }
}

/// Builds a typed error for an unsupported table operation.
fn invalid(operation: &str) -> StepError {
    grid::invalid(operation)
}
