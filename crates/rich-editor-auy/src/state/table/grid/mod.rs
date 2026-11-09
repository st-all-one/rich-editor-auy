//! Editable table grid shared by structural and span commands.

mod merge;
mod rows;
mod span;

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::Block;

use super::TableKinds;

/// One cell placed on the editable grid.
pub(super) struct Cell {
    /// Zero-based origin row.
    pub(super) row: u32,
    /// Zero-based origin column.
    pub(super) column: u32,
    /// Width in columns; always at least one.
    pub(super) colspan: u32,
    /// Height in rows; always at least one.
    pub(super) rowspan: u32,
    /// Whether the cell is a header.
    pub(super) header: bool,
    /// Underlying cell block.
    pub(super) block: Arc<Block>,
}

impl Cell {
    /// Creates an empty data cell at one grid position.
    fn empty(kinds: &TableKinds, row: u32, column: u32) -> Self {
        Self {
            row,
            column,
            colspan: 1,
            rowspan: 1,
            header: false,
            block: kinds.empty_cell(),
        }
    }

    /// Returns whether the cell covers a grid position.
    fn covers(&self, row: u32, column: u32) -> bool {
        row >= self.row
            && row < self.row.saturating_add(self.rowspan)
            && column >= self.column
            && column < self.column.saturating_add(self.colspan)
    }

    /// Returns whether the cell rectangle lies entirely inside another rectangle.
    fn inside(&self, top: u32, left: u32, bottom: u32, right: u32) -> bool {
        self.row >= top
            && self.column >= left
            && self.row.saturating_add(self.rowspan) <= bottom
            && self.column.saturating_add(self.colspan) <= right
    }
}

/// Editable table grid derived from a scanned table.
pub(super) struct Grid {
    rows: u32,
    columns: u32,
    cells: Vec<Cell>,
}

impl Grid {
    /// Builds an editable grid from a table summary.
    pub(super) fn from_info(info: &crate::model::TableInfo<'_>) -> Self {
        let mut cells = info
            .cells()
            .iter()
            .map(|cell| Cell {
                row: cell.cell.row,
                column: cell.cell.column,
                colspan: cell.colspan,
                rowspan: cell.rowspan,
                header: cell.header,
                block: Arc::new(cell.block.clone()),
            })
            .collect::<Vec<_>>();
        cells.sort_by_key(|cell| (cell.row, cell.column));
        Self {
            rows: info.rows(),
            columns: info.columns(),
            cells,
        }
    }

    /// Returns the row count.
    pub(super) fn rows(&self) -> u32 {
        self.rows
    }

    /// Returns the column count.
    pub(super) fn columns(&self) -> u32 {
        self.columns
    }

    /// Returns the cells in row-major origin order.
    pub(super) fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Returns the index of a cell by origin address.
    fn index_of(&self, cell: crate::model::CellRef) -> Option<usize> {
        self.cells
            .iter()
            .position(|entry| entry.row == cell.row && entry.column == cell.column)
    }

    /// Returns which columns a row already covers.
    fn row_coverage(&self, row: u32) -> Vec<bool> {
        let width = usize::try_from(self.columns).unwrap_or(usize::MAX);
        let mut covered = alloc::vec![false; width];
        for cell in &self.cells {
            if row < cell.row || row >= cell.row.saturating_add(cell.rowspan) {
                continue;
            }
            let start = usize::try_from(cell.column).unwrap_or(usize::MAX);
            let end =
                usize::try_from(cell.column.saturating_add(cell.colspan)).unwrap_or(usize::MAX);
            for index in start..end {
                if let Some(slot) = covered.get_mut(index) {
                    *slot = true;
                }
            }
        }
        covered
    }

    /// Returns whether a cell already covers one grid position.
    fn column_covered(&self, row: u32, column: u32) -> bool {
        self.cells.iter().any(|cell| cell.covers(row, column))
    }

    /// Restores row-major origin order.
    fn sort(&mut self) {
        self.cells.sort_by_key(|cell| (cell.row, cell.column));
    }
}

/// Builds a typed error for an unsupported table operation.
pub(super) fn invalid(operation: &str) -> StepError {
    StepError::Unsupported {
        operation: alloc::boxed::Box::from(operation),
    }
}
