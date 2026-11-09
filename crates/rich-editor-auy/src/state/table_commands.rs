//! Stateful editor commands for tables.
#![allow(
    clippy::fn_params_excessive_bools,
    reason = "the public API keeps the plan's boolean direction and header flags"
)]

use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::error::StepError;
use crate::model::{AttrValue, CellRef};
use crate::pos::Pos;

use super::editor::boundary_index;
use super::table::{self, Mutation, TableKinds, TableOp, TableShape};
use super::{AppliedInfo, ApplyError, Editor, Selection};

impl Editor {
    /// Inserts a table at a top-level block boundary.
    ///
    /// `header_rows` leading rows use `table_header` cells; every other cell is a `table_cell`.
    /// Each cell starts with one empty paragraph.
    ///
    /// # Errors
    ///
    /// Returns an out-of-bounds or unsupported error when `at` is not a top-level block boundary,
    /// the shape is empty or oversized, `header_rows` exceeds `rows`, or the resulting document
    /// exceeds configured limits.
    pub fn insert_table(
        &mut self,
        at: Pos,
        rows: usize,
        cols: usize,
        header_rows: usize,
    ) -> Result<AppliedInfo, ApplyError> {
        let index = boundary_index(self.doc(), at).map_err(|error| match error {
            StepError::Unsupported { .. } => StepError::Unsupported {
                operation: Box::from("table_position"),
            },
            other => other,
        })?;
        let rows = table_size(rows, "table_rows")?;
        let columns = table_size(cols, "table_columns")?;
        let header_rows = u32::try_from(header_rows).map_err(|_| StepError::Unsupported {
            operation: Box::from("table_header_rows"),
        })?;
        if header_rows > rows {
            return Err(StepError::Unsupported {
                operation: Box::from("table_header_rows"),
            }
            .into());
        }
        let schema = Arc::clone(self.state.schema());
        let kinds = TableKinds::resolve(&schema).ok_or_else(table_unknown)?;
        let shape = TableShape {
            rows,
            columns,
            header_rows,
        };
        let (doc, ordinal) = table::insert_table(self.doc(), &schema, &kinds, index, &shape)?;
        let cell = CellRef {
            table: ordinal,
            row: 0,
            column: 0,
        };
        self.commit(doc, ordinal, cell, cell)
    }

    /// Inserts one row before or after the row addressed by `cell`.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the table or row does not exist.
    pub fn insert_row(&mut self, cell: CellRef, after: bool) -> Result<AppliedInfo, ApplyError> {
        let at = if after {
            cell.row.saturating_add(1)
        } else {
            cell.row
        };
        self.mutate(cell.table, TableOp::InsertRow { at })
    }

    /// Deletes the row addressed by `cell`; the last remaining row is rejected.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the table or row does not exist or it is the only row.
    pub fn delete_row(&mut self, cell: CellRef) -> Result<AppliedInfo, ApplyError> {
        self.mutate(cell.table, TableOp::DeleteRow { row: cell.row })
    }

    /// Inserts one column before or after the column addressed by `cell`.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the table or column does not exist.
    pub fn insert_column(&mut self, cell: CellRef, after: bool) -> Result<AppliedInfo, ApplyError> {
        let at = if after {
            cell.column.saturating_add(1)
        } else {
            cell.column
        };
        self.mutate(cell.table, TableOp::InsertColumn { at })
    }

    /// Deletes the column addressed by `cell`; the last remaining column is rejected.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the table or column does not exist or it is the only column.
    pub fn delete_column(&mut self, cell: CellRef) -> Result<AppliedInfo, ApplyError> {
        self.mutate(
            cell.table,
            TableOp::DeleteColumn {
                column: cell.column,
            },
        )
    }

    /// Merges the rectangular selection between `anchor` and `head`.
    ///
    /// The top-left cell keeps the address and receives the concatenated content of every merged
    /// cell; the result is a header when any merged cell was a header.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the cells belong to different tables, the selection is a single
    /// cell, the rectangle is crossed by an existing span, or the merged grid would be invalid.
    pub fn merge_cells(
        &mut self,
        anchor: CellRef,
        head: CellRef,
    ) -> Result<AppliedInfo, ApplyError> {
        if anchor.table != head.table {
            return Err(StepError::Unsupported {
                operation: Box::from("table_merge_tables"),
            }
            .into());
        }
        self.mutate(anchor.table, TableOp::Merge { anchor, head })
    }

    /// Splits one merged cell into one cell per covered grid slot.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the cell does not exist or does not span several slots.
    pub fn split_cell(&mut self, cell: CellRef) -> Result<AppliedInfo, ApplyError> {
        self.mutate(cell.table, TableOp::Split { cell })
    }

    /// Converts one cell between `table_header` and `table_cell`.
    ///
    /// A `scope` attribute is kept only while the cell remains a header.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the cell does not exist or the grid would become invalid.
    pub fn set_cell_header(
        &mut self,
        cell: CellRef,
        header: bool,
    ) -> Result<AppliedInfo, ApplyError> {
        self.mutate(cell.table, TableOp::SetHeader { cell, header })
    }

    /// Sets one cell's `colspan` and `rowspan`.
    ///
    /// # Errors
    ///
    /// Returns a typed error when a span is below one, the cell does not exist, or the resulting
    /// grid would have holes, overlaps, or an inconsistent width.
    pub fn set_cell_span(
        &mut self,
        cell: CellRef,
        colspan: u32,
        rowspan: u32,
    ) -> Result<AppliedInfo, ApplyError> {
        self.mutate(
            cell.table,
            TableOp::SetSpan {
                cell,
                colspan,
                rowspan,
            },
        )
    }

    /// Sets one declared cell attribute; `align`, `abbr`, `scope`, and `headers` are the
    /// documented targets.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the attribute is undeclared for the cell, the value has the
    /// wrong type, the attribute is structural (`colspan`/`rowspan`), or the grid would become
    /// invalid.
    pub fn set_cell_attr(
        &mut self,
        cell: CellRef,
        attr: &str,
        value: AttrValue,
    ) -> Result<AppliedInfo, ApplyError> {
        self.mutate(cell.table, TableOp::SetAttr { cell, attr, value })
    }

    /// Applies one structural table edit through a whole-document replacement.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "the command value is short-lived and consumed immediately"
    )]
    fn mutate(&mut self, table: u32, op: TableOp<'_>) -> Result<AppliedInfo, ApplyError> {
        let schema = Arc::clone(self.state.schema());
        let kinds = TableKinds::resolve(&schema).ok_or_else(table_unknown)?;
        let Mutation {
            doc,
            table,
            anchor,
            head,
        } = table::mutate(self.doc(), &schema, &kinds, table, &op)?;
        self.commit(doc, table, anchor, head)
    }

    /// Records one replacement plus the resulting table selection.
    fn commit(
        &mut self,
        doc: crate::model::Doc,
        table: u32,
        anchor: CellRef,
        head: CellRef,
    ) -> Result<AppliedInfo, ApplyError> {
        let mut transaction = self.transaction();
        transaction.replace_document(doc)?;
        transaction.set_selection(Selection::Table {
            table,
            anchor,
            head,
        });
        self.apply(transaction)
    }
}

/// Converts a requested dimension into a grid count.
fn table_size(value: usize, operation: &str) -> Result<u32, StepError> {
    let count = u32::try_from(value).map_err(|_| StepError::Unsupported {
        operation: Box::from(operation),
    })?;
    if count == 0 {
        return Err(StepError::Unsupported {
            operation: Box::from(operation),
        });
    }
    Ok(count)
}

/// Builds the typed error for a schema without tables.
fn table_unknown() -> StepError {
    StepError::Unknown {
        path: Box::from("table"),
    }
}
