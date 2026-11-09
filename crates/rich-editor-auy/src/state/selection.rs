//! Logical selections.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use crate::model::CellRef;
use crate::pos::{Assoc, Pos, Range};
use crate::step::Mapping;

/// A single selection in document order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    /// No selection.
    #[default]
    None,
    /// Caret at one position.
    Caret {
        /// Position.
        pos: Pos,
        /// Boundary association.
        assoc: Assoc,
    },
    /// Text selection.
    Text {
        /// Anchor position.
        anchor: Pos,
        /// Head position.
        head: Pos,
        /// Boundary association.
        assoc: Assoc,
    },
    /// Whole-node selection.
    Node {
        /// Position before the node.
        pos: Pos,
    },
    /// Table-cell selection over one table's effective grid.
    ///
    /// Grid positions covered by a merged cell resolve to that cell's origin, so a selection may
    /// address any slot of the rectangle.
    Table {
        /// Table ordinal in document order.
        table: u32,
        /// Anchor cell.
        anchor: CellRef,
        /// Head cell.
        head: CellRef,
    },
}

impl Selection {
    /// Creates a caret.
    #[must_use]
    pub const fn caret(pos: Pos) -> Self {
        Self::Caret {
            pos,
            assoc: Assoc::After,
        }
    }

    /// Creates a text selection.
    #[must_use]
    pub const fn text(anchor: Pos, head: Pos) -> Self {
        Self::Text {
            anchor,
            head,
            assoc: Assoc::After,
        }
    }

    /// Creates a table-cell selection; `table` comes from the anchor address.
    #[must_use]
    pub const fn table(anchor: CellRef, head: CellRef) -> Self {
        Self::Table {
            table: anchor.table,
            anchor,
            head,
        }
    }

    /// Returns the anchor position.
    #[must_use]
    pub const fn anchor(self) -> Option<Pos> {
        match self {
            Self::None | Self::Node { .. } | Self::Table { .. } => None,
            Self::Caret { pos, .. } => Some(pos),
            Self::Text { anchor, .. } => Some(anchor),
        }
    }

    /// Returns the head position.
    #[must_use]
    pub const fn head(self) -> Option<Pos> {
        match self {
            Self::None | Self::Node { .. } | Self::Table { .. } => None,
            Self::Caret { pos, .. } => Some(pos),
            Self::Text { head, .. } => Some(head),
        }
    }

    /// Returns the normalized range.
    ///
    /// Table selections address cells instead of positions and have no range.
    #[must_use]
    pub const fn range(self) -> Option<Range> {
        match self {
            Self::None | Self::Node { .. } | Self::Table { .. } => None,
            Self::Caret { pos, .. } => Some(Range::new(pos, pos)),
            Self::Text { anchor, head, .. } => {
                if anchor.0 <= head.0 {
                    Some(Range::new(anchor, head))
                } else {
                    Some(Range::new(head, anchor))
                }
            }
        }
    }

    /// Returns whether the selection is empty.
    ///
    /// A table selection is empty when it addresses one cell.
    #[must_use]
    pub fn is_empty(self) -> bool {
        match self {
            Self::Table { anchor, head, .. } => anchor == head,
            _ => self.range().is_none_or(Range::is_empty),
        }
    }

    /// Maps the selection through a composition of steps.
    ///
    /// Table selections address table cells, not document positions, so they are preserved.
    #[must_use]
    pub fn map(self, mapping: &Mapping) -> Self {
        match self {
            Self::None => Self::None,
            Self::Caret { pos, assoc } => Self::Caret {
                pos: mapping.map_pos(pos, assoc).unwrap_or(pos),
                assoc,
            },
            Self::Text {
                anchor,
                head,
                assoc,
            } => Self::Text {
                anchor: mapping.map_pos(anchor, assoc).unwrap_or(anchor),
                head: mapping.map_pos(head, assoc).unwrap_or(head),
                assoc,
            },
            Self::Node { pos } => Self::Node {
                pos: mapping.map_pos(pos, Assoc::After).unwrap_or(pos),
            },
            Self::Table { .. } => self,
        }
    }

    /// Returns whether every position is inside the document.
    ///
    /// A table selection is valid when both cells address the selected table.
    #[must_use]
    pub fn is_valid(self, size: Pos) -> bool {
        let table_valid = match self {
            Self::Table {
                table,
                anchor,
                head,
            } => anchor.table == table && head.table == table,
            _ => true,
        };
        table_valid
            && self.anchor().is_none_or(|pos| pos.0 <= size.0)
            && self.head().is_none_or(|pos| pos.0 <= size.0)
            && self
                .range()
                .is_none_or(|range| range.start.0 <= range.end.0 && range.end.0 <= size.0)
    }
}
