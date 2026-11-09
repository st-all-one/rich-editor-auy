//! Cell spans, header flags, and attributes.

use core::mem::take;

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Attr, AttrId, AttrValue, Block, CellRef};
use crate::schema::Schema;

use super::super::TableKinds;
use super::{Cell, Grid, invalid};

impl Grid {
    /// Splits one merged cell into one cell per covered grid slot.
    ///
    /// The first slot keeps the original content; the remaining slots start empty.
    pub(in crate::state::table) fn split(
        &mut self,
        kinds: &TableKinds,
        cell: CellRef,
    ) -> Result<(u32, u32), StepError> {
        let index = self.index_of(cell).ok_or_else(|| invalid("table_cell"))?;
        let Some(origin) = self.cells.get(index) else {
            return Err(invalid("table_cell"));
        };
        if origin.colspan == 1 && origin.rowspan == 1 {
            return Err(invalid("table_split_single"));
        }
        let top = origin.row;
        let left = origin.column;
        let width = origin.colspan;
        let height = origin.rowspan;
        let header = origin.header;
        let attrs = origin.block.attrs().clone();
        let rest = attrs.without(AttrId::NODE_ID);
        let mut content = origin
            .block
            .children()
            .map(<[Arc<Block>]>::to_vec)
            .unwrap_or_default();
        let mut replacements = Vec::new();
        let mut first = true;
        for row in top..top.saturating_add(height) {
            for column in left..left.saturating_add(width) {
                let taken = if first {
                    take(&mut content)
                } else {
                    Vec::new()
                };
                let taken = if taken.is_empty() {
                    alloc::vec![kinds.empty_paragraph()]
                } else {
                    taken
                };
                let cell_attrs = if first { attrs.clone() } else { rest.clone() };
                let block = Arc::new(Block::container(
                    if header { kinds.header } else { kinds.cell },
                    cell_attrs,
                    taken,
                ));
                replacements.push(Cell {
                    row,
                    column,
                    colspan: 1,
                    rowspan: 1,
                    header,
                    block,
                });
                first = false;
            }
        }
        self.cells.remove(index);
        self.cells.extend(replacements);
        self.sort();
        Ok((top, left))
    }

    /// Sets whether one cell is a header.
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "header state is the documented single boolean operand"
    )]
    pub(in crate::state::table) fn set_header(
        &mut self,
        cell: CellRef,
        header: bool,
    ) -> Result<(u32, u32), StepError> {
        let index = self.index_of(cell).ok_or_else(|| invalid("table_cell"))?;
        let Some(entry) = self.cells.get_mut(index) else {
            return Err(invalid("table_cell"));
        };
        entry.header = header;
        Ok((entry.row, entry.column))
    }

    /// Sets one cell's span, absorbing fully covered cells and filling freed slots.
    ///
    /// Growing a span merges the cells it now covers; content is concatenated in row-major order.
    /// Shrinking it fills the freed grid slots with empty cells. A cell that is only partially
    /// covered is rejected.
    #[allow(
        clippy::too_many_lines,
        reason = "one explicit absorb-and-refill pass keeps span rules visible"
    )]
    pub(in crate::state::table) fn set_span(
        &mut self,
        kinds: &TableKinds,
        cell: CellRef,
        colspan: u32,
        rowspan: u32,
    ) -> Result<(u32, u32), StepError> {
        if colspan == 0 || rowspan == 0 {
            return Err(invalid("table_span"));
        }
        let index = self.index_of(cell).ok_or_else(|| invalid("table_cell"))?;
        let Some(origin) = self.cells.get(index) else {
            return Err(invalid("table_cell"));
        };
        let top = origin.row;
        let left = origin.column;
        let old_bottom = top.saturating_add(origin.rowspan);
        let old_right = left.saturating_add(origin.colspan);
        let bottom = top.saturating_add(rowspan);
        let right = left.saturating_add(colspan);
        if bottom > self.rows || right > self.columns {
            return Err(invalid("table_span_range"));
        }
        let attrs = origin.block.attrs().clone();
        let mut header = origin.header;
        let mut children = origin
            .block
            .children()
            .map(<[Arc<Block>]>::to_vec)
            .unwrap_or_default();
        let mut kept = Vec::with_capacity(self.cells.len());
        for (cell_index, entry) in take(&mut self.cells).into_iter().enumerate() {
            if cell_index == index {
                continue;
            }
            let overlaps = entry.row < bottom
                && top < entry.row.saturating_add(entry.rowspan)
                && entry.column < right
                && left < entry.column.saturating_add(entry.colspan);
            if !overlaps {
                kept.push(entry);
                continue;
            }
            if !entry.inside(top, left, bottom, right) {
                return Err(invalid("table_span_partial"));
            }
            header = header || entry.header;
            if let Some(items) = entry.block.children() {
                children.extend(items.iter().cloned());
            }
        }
        if children.is_empty() {
            children.push(kinds.empty_paragraph());
        }
        self.cells = kept;
        for row in top..old_bottom {
            for column in left..old_right {
                if row < bottom && column < right {
                    continue;
                }
                if self.column_covered(row, column) {
                    continue;
                }
                self.cells.push(Cell::empty(kinds, row, column));
            }
        }
        self.cells.push(Cell {
            row: top,
            column: left,
            colspan,
            rowspan,
            header,
            block: Arc::new(Block::container(
                if header { kinds.header } else { kinds.cell },
                attrs,
                children,
            )),
        });
        self.sort();
        Ok((top, left))
    }

    /// Sets one declared cell attribute after validating its value.
    pub(in crate::state::table) fn set_attr(
        &mut self,
        schema: &Schema,
        cell: CellRef,
        name: &str,
        value: AttrValue,
    ) -> Result<(u32, u32), StepError> {
        if name == "colspan" || name == "rowspan" {
            return Err(invalid("table_span_attribute"));
        }
        let index = self.index_of(cell).ok_or_else(|| invalid("table_cell"))?;
        let Some(entry) = self.cells.get_mut(index) else {
            return Err(invalid("table_cell"));
        };
        let kind = entry.block.kind();
        let Some(spec) = schema.node_attr(kind, name) else {
            return Err(StepError::Unknown {
                path: alloc::boxed::Box::from(name),
            });
        };
        let attrs = if matches!(value, AttrValue::Null) {
            entry.block.attrs().without(spec.id)
        } else {
            if !Schema::value_matches(&spec.kind, &value) {
                return Err(StepError::ContentMismatch {
                    path: alloc::boxed::Box::from(name),
                });
            }
            entry
                .block
                .attrs()
                .with(Attr { id: spec.id, value })
                .ok_or_else(|| invalid("table_attribute_conflict"))?
        };
        let children = entry
            .block
            .children()
            .map(<[Arc<Block>]>::to_vec)
            .unwrap_or_default();
        entry.block = Arc::new(Block::container(kind, attrs, children));
        Ok((entry.row, entry.column))
    }
}
