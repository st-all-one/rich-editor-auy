//! Table grid data and invariants.
//!
//! Cells are placed on an effective grid with the canonical HTML algorithm: each row fills its
//! first free column, a cell keeps its `colspan`/`rowspan` rectangle, and every row reports the
//! same effective width. Grid invariants use the stable codes below and report the child indices
//! needed for a structural path.
//!
//! - `I17`: the table has no rows.
//! - `I18`: `colspan`/`rowspan` below one.
//! - `I19`: effective row widths differ.
//! - `I20`: `rowspan` extends past the last row.
//! - `I21`: two cells overlap on the grid.
//! - `I22`: `scope` is invalid for the cell.
//! - `I23`: `headers` references an unknown identifier.
//! - `I24`: the caption is duplicated or not the first child.
//! - `I25`: the grid exceeds the resolution bound and the summary is a prefix.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::schema::Schema;

use super::attrs::AttrValue;
use super::block::Block;
use super::doc::Doc;
use super::ids::{NodeId, NodeKind};

pub(crate) mod grid;

pub(crate) use grid::scan_table;

/// Address of one cell inside a document table.
///
/// The table ordinal counts `table` blocks in document order. `row` and `column` address the
/// effective grid, so a position covered by a `colspan`/`rowspan` resolves to the origin cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellRef {
    /// Table ordinal in document order.
    pub table: u32,
    /// Zero-based grid row.
    pub row: u32,
    /// Zero-based grid column.
    pub column: u32,
}

/// One cell placed on a table grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableCell<'a> {
    /// Origin address of the cell.
    pub cell: CellRef,
    /// Whether the cell is a `table_header`.
    pub header: bool,
    /// Number of columns covered; always at least one.
    pub colspan: u32,
    /// Number of rows covered; always at least one.
    pub rowspan: u32,
    /// Underlying cell block.
    pub block: &'a Block,
}

/// Deterministic summary of one table's effective grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableInfo<'a> {
    table: u32,
    block: &'a Block,
    caption: Option<&'a Block>,
    rows: u32,
    columns: u32,
    cells: Vec<TableCell<'a>>,
    grid: Vec<Vec<Option<u32>>>,
    truncated: bool,
}

/// One grid violation addressed by child indices inside a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TableProblem {
    /// Stable invariant code.
    pub(crate) invariant: &'static str,
    /// Child index of the row, when the violation is row-scoped.
    pub(crate) row: Option<u32>,
    /// Child index of the cell inside that row, when the violation is cell-scoped.
    pub(crate) cell: Option<u32>,
}

/// Grid summary plus the problems found while building it.
pub(crate) struct TableScan<'a> {
    /// Resolved grid.
    pub(crate) info: TableInfo<'a>,
    /// Violations in document order.
    pub(crate) problems: Vec<TableProblem>,
}

/// Node kinds that make up a table.
struct TableKinds {
    table: NodeKind,
    caption: NodeKind,
    row: NodeKind,
    header: NodeKind,
}

impl<'a> TableInfo<'a> {
    /// Returns the table ordinal in document order.
    #[must_use]
    pub const fn table_index(&self) -> u32 {
        self.table
    }

    /// Returns the number of grid rows.
    #[must_use]
    pub const fn rows(&self) -> u32 {
        self.rows
    }

    /// Returns the number of grid columns.
    #[must_use]
    pub const fn columns(&self) -> u32 {
        self.columns
    }

    /// Returns whether the table declares a caption.
    #[must_use]
    pub const fn has_caption(&self) -> bool {
        self.caption.is_some()
    }

    /// Returns the placed cells in document order.
    #[must_use]
    pub fn cells(&self) -> &[TableCell<'a>] {
        &self.cells
    }

    /// Returns whether the summary stopped before covering every cell.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Resolves a grid position to the cell that occupies it.
    ///
    /// Positions covered by a merged cell resolve to that cell's origin.
    #[must_use]
    pub fn cell(&self, row: u32, column: u32) -> Option<&TableCell<'a>> {
        let row_index = usize::try_from(row).ok()?;
        let column_index = usize::try_from(column).ok()?;
        let slot = self.grid.get(row_index)?.get(column_index)?;
        self.cells.get(usize::try_from((*slot)?).ok()?)
    }

    /// Returns the origin address of the cell occupying a grid position.
    #[must_use]
    pub fn cell_ref(&self, row: u32, column: u32) -> Option<CellRef> {
        self.cell(row, column).map(|found| found.cell)
    }
}

impl TableKinds {
    /// Resolves every table node kind, or `None` when the schema has no tables.
    fn resolve(schema: &Schema) -> Option<Self> {
        Some(Self {
            table: schema.node_kind("table")?,
            caption: schema.node_kind("table_caption")?,
            row: schema.node_kind("table_row")?,
            header: schema.node_kind("table_header")?,
        })
    }
}

impl Doc {
    /// Returns the effective grid of one table, counted in document order.
    ///
    /// Returns `None` when the schema declares no tables or the ordinal is out of range.
    #[must_use]
    pub fn table_info(&self, schema: &Schema, table_index: u32) -> Option<TableInfo<'_>> {
        let kinds = TableKinds::resolve(schema)?;
        let block = table_block(self, &kinds, table_index)?;
        scan_table(table_index, block, schema).map(|scan| scan.info)
    }

    /// Resolves a cell address to its block.
    ///
    /// A position covered by a merged cell resolves to the origin cell's block.
    #[must_use]
    pub fn cell(&self, schema: &Schema, cell: CellRef) -> Option<&Block> {
        self.table_info(schema, cell.table)?
            .cell(cell.row, cell.column)
            .map(|found| found.block)
    }
}

/// Reads one integer span attribute, defaulting to one and flagging values below one.
fn read_span(block: &Block, schema: &Schema, name: &str) -> (u32, bool) {
    let Some(spec) = schema.node_attr(block.kind(), name) else {
        return (1, false);
    };
    let Some(AttrValue::Int(raw)) = block.attrs().get(spec.id) else {
        return (1, false);
    };
    if *raw < 1 {
        return (1, true);
    }
    (u32::try_from(*raw).unwrap_or(u32::MAX), false)
}

/// Returns whether a `scope` value is valid for a cell covering the given span.
fn scope_matches(value: &str, wide: u32, tall: u32) -> bool {
    match value {
        "col" | "row" => true,
        "colgroup" => wide > 1,
        "rowgroup" => tall > 1,
        _ => false,
    }
}

/// Returns whether one `headers` entry names a cell of the same table.
fn known_header(value: &AttrValue, ids: &BTreeSet<NodeId>) -> bool {
    match value {
        AttrValue::Str(text) => NodeId::new(text).is_some_and(|id| ids.contains(&id)),
        _ => false,
    }
}

/// Collects `I23` violations for every cell of one table.
pub(super) fn header_problems(
    cells: &[TableCell<'_>],
    addresses: &[(u32, u32)],
    schema: &Schema,
    problems: &mut Vec<TableProblem>,
) {
    let mut ids = BTreeSet::new();
    for cell in cells {
        if let Some(id) = cell.block.id() {
            ids.insert(id);
        }
    }
    for (index, cell) in cells.iter().enumerate() {
        let Some(spec) = schema.node_attr(cell.block.kind(), "headers") else {
            continue;
        };
        let Some(AttrValue::Array(values)) = cell.block.attrs().get(spec.id) else {
            continue;
        };
        let unknown = values.iter().any(|value| !known_header(value, &ids));
        if unknown && let Some((row, column)) = addresses.get(index) {
            problems.push(TableProblem {
                invariant: "I23",
                row: Some(*row),
                cell: Some(*column),
            });
        }
    }
}

/// Finds one table block by document-order ordinal.
fn table_block<'a>(doc: &'a Doc, kinds: &TableKinds, target: u32) -> Option<&'a Block> {
    let mut seen = 0_u32;
    for block in doc.blocks().iter() {
        if let Some(found) = find_table(block, kinds, target, &mut seen) {
            return Some(found);
        }
    }
    None
}

/// Walks blocks in document order, counting tables.
fn find_table<'a>(
    block: &'a Block,
    kinds: &TableKinds,
    target: u32,
    seen: &mut u32,
) -> Option<&'a Block> {
    if block.kind() == kinds.table {
        if *seen == target {
            return Some(block);
        }
        *seen = seen.saturating_add(1);
        return None;
    }
    for child in block.children()? {
        if let Some(found) = find_table(child, kinds, target, seen) {
            return Some(found);
        }
    }
    None
}
