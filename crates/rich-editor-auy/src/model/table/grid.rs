//! Deterministic table grid placement.
#![allow(
    clippy::excessive_nesting,
    clippy::redundant_pub_crate,
    reason = "one explicit deterministic grid walk that stays crate-scoped"
)]

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::schema::Schema;

use super::super::attrs::AttrValue;
use super::super::block::Block;
use super::{
    CellRef, TableCell, TableInfo, TableKinds, TableProblem, TableScan, header_problems, read_span,
    scope_matches,
};

/// Maximum grid slots resolved for one table.
const MAX_GRID_SLOTS: u64 = 1 << 21;

/// One cell being placed on the grid.
struct Pending<'a> {
    block: &'a Block,
    header: bool,
    wide: u32,
    tall: u32,
    row: u32,
    row_child: u32,
    cell_child: u32,
    column: u32,
    ordinal: u32,
}

/// Mutable grid construction state.
struct GridBuilder<'a, 's> {
    table: u32,
    block: &'a Block,
    kinds: &'s TableKinds,
    schema: &'s Schema,
    cells: Vec<TableCell<'a>>,
    addresses: Vec<(u32, u32)>,
    grid: Vec<Vec<Option<u32>>>,
    caption: Option<&'a Block>,
    rows: u32,
    columns: u32,
    expected: Option<u32>,
    problems: Vec<TableProblem>,
    slots: u64,
    truncated: bool,
}

/// Builds the grid summary and invariant problems for one table block.
pub(crate) fn scan_table<'a>(
    table: u32,
    block: &'a Block,
    schema: &Schema,
) -> Option<TableScan<'a>> {
    let kinds = TableKinds::resolve(schema)?;
    if block.kind() != kinds.table {
        return None;
    }
    let mut builder = GridBuilder::new(table, block, &kinds, schema);
    builder.scan();
    Some(builder.finish())
}

impl<'a, 's> GridBuilder<'a, 's> {
    fn new(table: u32, block: &'a Block, kinds: &'s TableKinds, schema: &'s Schema) -> Self {
        let mut rows = 0_u32;
        if let Some(children) = block.children() {
            for child in children {
                if child.kind() == kinds.row {
                    rows = rows.saturating_add(1);
                }
            }
        }
        let mut grid = Vec::new();
        for _ in 0..rows {
            grid.push(Vec::new());
        }
        Self {
            table,
            block,
            kinds,
            schema,
            cells: Vec::new(),
            addresses: Vec::new(),
            grid,
            caption: None,
            rows,
            columns: 0,
            expected: None,
            problems: Vec::new(),
            slots: 0,
            truncated: false,
        }
    }

    fn scan(&mut self) {
        let block = self.block;
        let children: &'a [Arc<Block>] = match block.children() {
            Some(children) => children,
            None => &[],
        };
        let mut captions = 0_u32;
        let mut row = 0_u32;
        for (index, child) in children.iter().enumerate() {
            let child_index = u32::try_from(index).unwrap_or(u32::MAX);
            if child.kind() == self.kinds.caption {
                captions = captions.saturating_add(1);
                if captions == 1 {
                    let caption: &'a Block = child;
                    self.caption = Some(caption);
                }
                if index != 0 || captions > 1 {
                    self.problem("I24", Some(child_index), None);
                }
            } else if child.kind() == self.kinds.row {
                self.scan_row(row, child_index, child);
                self.finish_row(child_index, row);
                row = row.saturating_add(1);
            }
        }
        header_problems(
            &self.cells,
            &self.addresses,
            self.schema,
            &mut self.problems,
        );
        if self.rows == 0 {
            self.problem("I17", None, None);
        }
    }

    fn scan_row(&mut self, row: u32, row_child: u32, block: &'a Block) {
        let Some(children) = block.children() else {
            return;
        };
        for (index, cell) in children.iter().enumerate() {
            let cell_child = u32::try_from(index).unwrap_or(u32::MAX);
            self.place(row, row_child, cell_child, cell);
        }
    }

    /// Records the effective width of a completed row.
    fn finish_row(&mut self, row_child: u32, row: u32) {
        let index = usize::try_from(row).unwrap_or(usize::MAX);
        let Some(slots) = self.grid.get(index) else {
            return;
        };
        let width = u32::try_from(slots.len()).unwrap_or(u32::MAX);
        self.columns = self.columns.max(width);
        match self.expected {
            None => self.expected = Some(width),
            Some(first) => {
                if first != width {
                    self.problem("I19", Some(row_child), None);
                }
            }
        }
    }

    fn place(&mut self, row: u32, row_child: u32, cell_child: u32, block: &'a Block) {
        let span_columns = read_span(block, self.schema, "colspan");
        let span_rows = read_span(block, self.schema, "rowspan");
        let pending = Pending {
            block,
            header: block.kind() == self.kinds.header,
            wide: span_columns.0,
            tall: span_rows.0,
            row,
            row_child,
            cell_child,
            column: self.first_free(row),
            ordinal: u32::try_from(self.cells.len()).unwrap_or(u32::MAX),
        };
        if span_columns.1 || span_rows.1 {
            self.problem("I18", Some(row_child), Some(cell_child));
        }
        self.check_scope(&pending);
        self.cells.push(TableCell {
            cell: CellRef {
                table: self.table,
                row: pending.row,
                column: pending.column,
            },
            header: pending.header,
            colspan: pending.wide,
            rowspan: pending.tall,
            block: pending.block,
        });
        self.addresses.push((row_child, cell_child));
        self.fill(&pending);
    }

    fn check_scope(&mut self, pending: &Pending<'a>) {
        let schema = self.schema;
        let Some(spec) = schema.node_attr(pending.block.kind(), "scope") else {
            return;
        };
        let Some(AttrValue::Str(value)) = pending.block.attrs().get(spec.id) else {
            return;
        };
        if !pending.header || !scope_matches(value, pending.wide, pending.tall) {
            self.problem("I22", Some(pending.row_child), Some(pending.cell_child));
        }
    }

    fn fill(&mut self, pending: &Pending<'a>) {
        let end_row = pending.row.saturating_add(pending.tall);
        if end_row > self.rows {
            self.problem("I20", Some(pending.row_child), Some(pending.cell_child));
        }
        let area = u64::from(pending.wide).saturating_mul(u64::from(pending.tall));
        if self.slots.saturating_add(area) > MAX_GRID_SLOTS {
            if !self.truncated {
                self.truncated = true;
                self.problem("I25", Some(pending.row_child), Some(pending.cell_child));
            }
            return;
        }
        let last_row = end_row.min(self.rows);
        let end_column = pending.column.saturating_add(pending.wide);
        let mut overlap = false;
        for row in pending.row..last_row {
            for column in pending.column..end_column {
                if !self.write(row, column, pending.ordinal) {
                    overlap = true;
                }
            }
        }
        if overlap {
            self.problem("I21", Some(pending.row_child), Some(pending.cell_child));
        }
    }

    fn write(&mut self, row: u32, column: u32, ordinal: u32) -> bool {
        let row_index = usize::try_from(row).unwrap_or(usize::MAX);
        let Some(slots) = self.grid.get_mut(row_index) else {
            return false;
        };
        let column_index = usize::try_from(column).unwrap_or(usize::MAX);
        if column_index >= slots.len() {
            slots.resize(column_index.saturating_add(1), None);
        }
        let Some(slot) = slots.get_mut(column_index) else {
            return false;
        };
        if slot.is_some() {
            return false;
        }
        *slot = Some(ordinal);
        self.slots = self.slots.saturating_add(1);
        true
    }

    fn first_free(&self, row: u32) -> u32 {
        let index = usize::try_from(row).unwrap_or(usize::MAX);
        let Some(slots) = self.grid.get(index) else {
            return 0;
        };
        let column = slots
            .iter()
            .position(Option::is_none)
            .unwrap_or(slots.len());
        u32::try_from(column).unwrap_or(u32::MAX)
    }

    fn problem(&mut self, invariant: &'static str, row: Option<u32>, cell: Option<u32>) {
        self.problems.push(TableProblem {
            invariant,
            row,
            cell,
        });
    }

    fn finish(self) -> TableScan<'a> {
        TableScan {
            info: TableInfo {
                table: self.table,
                block: self.block,
                caption: self.caption,
                rows: self.rows,
                columns: self.columns,
                cells: self.cells,
                grid: self.grid,
                truncated: self.truncated,
            },
            problems: self.problems,
        }
    }
}
