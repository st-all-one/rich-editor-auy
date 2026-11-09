//! Row and column insertion and removal.

use core::mem::take;

use alloc::vec::Vec;

use crate::error::StepError;

use super::super::TableKinds;
use super::{Cell, Grid, invalid};

impl Grid {
    /// Inserts one empty row at `at`, extending spans that cross the insertion point.
    pub(in crate::state::table) fn insert_row(
        &mut self,
        kinds: &TableKinds,
        at: u32,
    ) -> Result<(), StepError> {
        if at > self.rows {
            return Err(invalid("table_row_range"));
        }
        for cell in &mut self.cells {
            if cell.row >= at {
                cell.row = cell.row.saturating_add(1);
            } else if cell.row.saturating_add(cell.rowspan) > at {
                cell.rowspan = cell.rowspan.saturating_add(1);
            }
        }
        self.rows = self.rows.saturating_add(1);
        let covered = self.row_coverage(at);
        for (index, filled) in covered.iter().enumerate() {
            if *filled {
                continue;
            }
            let Ok(column) = u32::try_from(index) else {
                continue;
            };
            self.cells.push(Cell::empty(kinds, at, column));
        }
        self.sort();
        Ok(())
    }

    /// Deletes one grid row; the last remaining row cannot be deleted.
    pub(in crate::state::table) fn delete_row(&mut self, row: u32) -> Result<(), StepError> {
        if self.rows <= 1 {
            return Err(invalid("table_last_row"));
        }
        if row >= self.rows {
            return Err(invalid("table_row_range"));
        }
        let mut kept = Vec::new();
        for mut cell in take(&mut self.cells) {
            if cell.row == row {
                if cell.rowspan > 1 {
                    cell.rowspan = cell.rowspan.saturating_sub(1);
                    kept.push(cell);
                }
            } else if cell.row < row && cell.row.saturating_add(cell.rowspan) > row {
                cell.rowspan = cell.rowspan.saturating_sub(1);
                kept.push(cell);
            } else {
                if cell.row > row {
                    cell.row = cell.row.saturating_sub(1);
                }
                kept.push(cell);
            }
        }
        self.cells = kept;
        self.rows = self.rows.saturating_sub(1);
        self.sort();
        Ok(())
    }

    /// Inserts one empty column at `at`, extending spans that cross the insertion point.
    pub(in crate::state::table) fn insert_column(
        &mut self,
        kinds: &TableKinds,
        at: u32,
    ) -> Result<(), StepError> {
        if at > self.columns {
            return Err(invalid("table_column_range"));
        }
        for cell in &mut self.cells {
            if cell.column >= at {
                cell.column = cell.column.saturating_add(1);
            } else if cell.column.saturating_add(cell.colspan) > at {
                cell.colspan = cell.colspan.saturating_add(1);
            }
        }
        self.columns = self.columns.saturating_add(1);
        for row in 0..self.rows {
            if self.column_covered(row, at) {
                continue;
            }
            self.cells.push(Cell::empty(kinds, row, at));
        }
        self.sort();
        Ok(())
    }

    /// Deletes one grid column; the last remaining column cannot be deleted.
    pub(in crate::state::table) fn delete_column(&mut self, column: u32) -> Result<(), StepError> {
        if self.columns <= 1 {
            return Err(invalid("table_last_column"));
        }
        if column >= self.columns {
            return Err(invalid("table_column_range"));
        }
        let mut kept = Vec::new();
        for mut cell in take(&mut self.cells) {
            if cell.column == column {
                if cell.colspan > 1 {
                    cell.colspan = cell.colspan.saturating_sub(1);
                    kept.push(cell);
                }
            } else if cell.column < column && cell.column.saturating_add(cell.colspan) > column {
                cell.colspan = cell.colspan.saturating_sub(1);
                kept.push(cell);
            } else {
                if cell.column > column {
                    cell.column = cell.column.saturating_sub(1);
                }
                kept.push(cell);
            }
        }
        self.cells = kept;
        self.columns = self.columns.saturating_sub(1);
        self.sort();
        Ok(())
    }
}
