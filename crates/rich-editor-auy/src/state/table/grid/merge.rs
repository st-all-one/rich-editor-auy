//! Cell merging.

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Attrs, Block, CellRef};

use super::super::TableKinds;
use super::{Cell, Grid, invalid};

impl Grid {
    /// Merges the rectangle spanned by two cells into its top-left cell.
    ///
    /// Merged content is concatenated in row-major order, the result is a header when any merged
    /// cell was a header, and a rectangle crossed by an existing span is rejected.
    pub(in crate::state::table) fn merge(
        &mut self,
        kinds: &TableKinds,
        anchor: CellRef,
        head: CellRef,
    ) -> Result<(u32, u32), StepError> {
        let first = self.index_of(anchor).ok_or_else(|| invalid("table_cell"))?;
        let second = self.index_of(head).ok_or_else(|| invalid("table_cell"))?;
        let (Some(first), Some(second)) = (self.cells.get(first), self.cells.get(second)) else {
            return Err(invalid("table_cell"));
        };
        let top = first.row.min(second.row);
        let left = first.column.min(second.column);
        let bottom = first
            .row
            .saturating_add(first.rowspan)
            .max(second.row.saturating_add(second.rowspan));
        let right = first
            .column
            .saturating_add(first.colspan)
            .max(second.column.saturating_add(second.colspan));
        if bottom.saturating_sub(top) == 1 && right.saturating_sub(left) == 1 {
            return Err(invalid("table_merge_single"));
        }
        let mut inside = Vec::new();
        for (index, cell) in self.cells.iter().enumerate() {
            let overlaps = cell.row < bottom
                && top < cell.row.saturating_add(cell.rowspan)
                && cell.column < right
                && left < cell.column.saturating_add(cell.colspan);
            if !overlaps {
                continue;
            }
            if !cell.inside(top, left, bottom, right) {
                return Err(invalid("table_merge_partial"));
            }
            inside.push(index);
        }
        if inside.is_empty() {
            return Err(invalid("table_merge_empty"));
        }
        inside.sort_by_key(|index| self.cells.get(*index).map(|cell| (cell.row, cell.column)));
        let (header, template, children) = self.merged_content(&inside, kinds);
        let block = Arc::new(Block::container(
            if header { kinds.header } else { kinds.cell },
            template,
            children,
        ));
        for index in inside.iter().rev() {
            if *index < self.cells.len() {
                self.cells.remove(*index);
            }
        }
        self.cells.push(Cell {
            row: top,
            column: left,
            colspan: right.saturating_sub(left),
            rowspan: bottom.saturating_sub(top),
            header,
            block,
        });
        self.sort();
        Ok((top, left))
    }

    /// Combines merged cell content in row-major order.
    fn merged_content(
        &self,
        inside: &[usize],
        kinds: &TableKinds,
    ) -> (bool, Attrs, Vec<Arc<Block>>) {
        let mut header = false;
        let mut template = None;
        let mut children = Vec::new();
        for index in inside {
            let Some(cell) = self.cells.get(*index) else {
                continue;
            };
            header = header || cell.header;
            if template.is_none() {
                template = Some(cell.block.attrs().clone());
            }
            if let Some(items) = cell.block.children() {
                children.extend(items.iter().cloned());
            }
        }
        if children.is_empty() {
            children.push(kinds.empty_paragraph());
        }
        (header, template.unwrap_or_default(), children)
    }
}
