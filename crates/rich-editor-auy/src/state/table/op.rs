//! Command values for table edits.
#![allow(
    clippy::redundant_pub_crate,
    reason = "command values stay crate-scoped inside a private module"
)]

use crate::model::{AttrValue, CellRef};

/// Shape requested by `Editor::insert_table`.
pub(crate) struct TableShape {
    /// Number of body rows; at least one.
    pub(crate) rows: u32,
    /// Number of grid columns; at least one.
    pub(crate) columns: u32,
    /// Number of leading rows whose cells are headers.
    pub(crate) header_rows: u32,
}

/// One structural table edit addressed by grid cells.
pub(crate) enum TableOp<'a> {
    /// Insert a row at a grid row index.
    InsertRow {
        /// Insertion index; `rows` appends.
        at: u32,
    },
    /// Delete a grid row.
    DeleteRow {
        /// Row index.
        row: u32,
    },
    /// Insert a column at a grid column index.
    InsertColumn {
        /// Insertion index; `columns` appends.
        at: u32,
    },
    /// Delete a grid column.
    DeleteColumn {
        /// Column index.
        column: u32,
    },
    /// Merge a rectangular cell selection.
    Merge {
        /// Anchor cell.
        anchor: CellRef,
        /// Head cell.
        head: CellRef,
    },
    /// Split one merged cell.
    Split {
        /// Cell address.
        cell: CellRef,
    },
    /// Set whether a cell is a header.
    SetHeader {
        /// Cell address.
        cell: CellRef,
        /// New header state.
        header: bool,
    },
    /// Set a cell span.
    SetSpan {
        /// Cell address.
        cell: CellRef,
        /// New column span.
        colspan: u32,
        /// New row span.
        rowspan: u32,
    },
    /// Set one declared cell attribute.
    SetAttr {
        /// Cell address.
        cell: CellRef,
        /// Attribute name.
        attr: &'a str,
        /// New value.
        value: AttrValue,
    },
}
