//! Rebuilds one table block from grid data and validates its grid.

use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::error::StepError;
use crate::model::{Attr, AttrValue, Attrs, Block, Doc, NodeKind};
use crate::schema::Schema;

use super::grid::{Cell, Grid};
use super::{TableKinds, invalid, unknown};

/// Rebuilds one table block from grid data, preserving caption and row attributes.
pub(super) fn emit(
    original: &Arc<Block>,
    schema: &Schema,
    kinds: &TableKinds,
    grid: &Grid,
) -> Result<Arc<Block>, StepError> {
    let mut children = Vec::new();
    if let Some(caption) = original
        .children()
        .and_then(|items| items.iter().find(|child| child.kind() == kinds.caption))
    {
        children.push(Arc::clone(caption));
    }
    let rows = original
        .children()
        .map(|items| {
            items
                .iter()
                .filter(|child| child.kind() == kinds.row)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for row in 0..grid.rows() {
        let attrs = rows
            .get(usize::try_from(row).unwrap_or(usize::MAX))
            .map_or_else(Attrs::empty, |block| block.attrs().clone());
        let cells = grid
            .cells()
            .iter()
            .filter(|cell| cell.row == row)
            .map(|cell| cell_block(schema, kinds, cell))
            .collect::<Result<Vec<_>, StepError>>()?;
        if cells.is_empty() {
            return Err(invalid("table_empty_row"));
        }
        children.push(Arc::new(Block::container(kinds.row, attrs, cells)));
    }
    Ok(Arc::new(Block::container(
        kinds.table,
        original.attrs().clone(),
        children,
    )))
}

/// Rebuilds one cell block: kind, span attributes, normalized scope, and content.
fn cell_block(schema: &Schema, kinds: &TableKinds, cell: &Cell) -> Result<Arc<Block>, StepError> {
    let kind = if cell.header {
        kinds.header
    } else {
        kinds.cell
    };
    let attrs = span_attr(schema, kind, cell.block.attrs(), "colspan", cell.colspan)?;
    let attrs = span_attr(schema, kind, &attrs, "rowspan", cell.rowspan)?;
    let attrs = scope_attr(schema, kind, &attrs, cell);
    let mut children = cell
        .block
        .children()
        .map(<[Arc<Block>]>::to_vec)
        .unwrap_or_default();
    if children.is_empty() {
        children.push(kinds.empty_paragraph());
    }
    Ok(Arc::new(Block::container(kind, attrs, children)))
}

/// Sets or removes one span attribute, omitting the canonical default of one.
fn span_attr(
    schema: &Schema,
    kind: NodeKind,
    attrs: &Attrs,
    name: &str,
    span: u32,
) -> Result<Attrs, StepError> {
    let Some(spec) = schema.node_attr(kind, name) else {
        return if span > 1 {
            Err(unknown(name))
        } else {
            Ok(attrs.clone())
        };
    };
    if span <= 1 {
        return Ok(attrs.without(spec.id));
    }
    let value = AttrValue::Int(i64::from(span));
    attrs
        .with(Attr { id: spec.id, value })
        .ok_or_else(|| invalid("table_attribute_conflict"))
}

/// Drops `scope` unless it matches the cell role and current span.
fn scope_attr(schema: &Schema, kind: NodeKind, attrs: &Attrs, cell: &Cell) -> Attrs {
    let Some(spec) = schema.node_attr(kind, "scope") else {
        return attrs.clone();
    };
    let Some(AttrValue::Str(value)) = attrs.get(spec.id) else {
        return attrs.clone();
    };
    let valid = cell.header
        && match &**value {
            "col" | "row" => true,
            "colgroup" => cell.colspan > 1,
            "rowgroup" => cell.rowspan > 1,
            _ => false,
        };
    if valid {
        attrs.clone()
    } else {
        attrs.without(spec.id)
    }
}

/// Rejects a rebuilt table whose grid violates the canonical table invariants.
pub(super) fn check_table(block: &Arc<Block>, schema: &Schema) -> Result<(), StepError> {
    let Ok(probe) = Doc::from_blocks(Attrs::empty(), alloc::vec![Arc::clone(block)]) else {
        return Err(invalid("table_probe"));
    };
    let report = probe.validate(schema);
    if report
        .issues
        .iter()
        .any(|issue| is_table_invariant(issue.invariant))
    {
        return Err(invalid("table_invalid_grid"));
    }
    Ok(())
}

/// Returns whether an invariant code is a table-grid invariant.
fn is_table_invariant(code: &str) -> bool {
    matches!(
        code,
        "I17" | "I18" | "I19" | "I20" | "I21" | "I22" | "I23" | "I24" | "I25"
    )
}
