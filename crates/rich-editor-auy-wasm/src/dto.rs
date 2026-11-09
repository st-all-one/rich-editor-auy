//! JSON data transfer objects for the stateful WASM API.
//!
//! The shapes mirror the documented TypeScript contract: selections, batch operations,
//! batch envelopes, and import payloads. Unknown fields and unknown operation names are
//! rejected by `serde`, which keeps ingestion strict.

use serde::{Deserialize, Serialize};
use serde_json::Value;

mod import;
pub(crate) use import::{BatchLimits, ImportPayload};

/// Boundary association sign default.
fn default_assoc() -> i8 {
    1
}

/// Serialized selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum SelectionJson {
    /// No selection.
    None,
    /// Caret at one position.
    Caret {
        /// Position.
        pos: u32,
        /// Boundary association, `-1` before or `1` after.
        #[serde(default = "default_assoc")]
        assoc: i8,
    },
    /// Text selection.
    Text {
        /// Anchor position.
        anchor: u32,
        /// Head position.
        head: u32,
        /// Boundary association, `-1` before or `1` after.
        #[serde(default = "default_assoc")]
        assoc: i8,
    },
    /// Whole-node selection.
    Node {
        /// Position before the node.
        pos: u32,
    },
    /// Table-cell selection over one table's grid.
    Table {
        /// Table ordinal in document order.
        table: u32,
        /// Anchor cell.
        anchor: CellRefJson,
        /// Head cell.
        head: CellRefJson,
    },
}

/// Serialized table cell reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CellRefJson {
    /// Table ordinal in document order.
    pub(crate) table: u32,
    /// Zero-based grid row.
    pub(crate) row: u32,
    /// Zero-based grid column.
    pub(crate) column: u32,
}

/// One batch operation.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "op")]
pub(crate) enum Op {
    /// Insert plain text at a position.
    #[serde(rename = "insert_text", alias = "insertText")]
    InsertText {
        /// Position.
        at: u32,
        /// UTF-8 text.
        text: String,
    },
    /// Delete a byte range.
    #[serde(rename = "delete_range", alias = "deleteRange")]
    DeleteRange {
        /// Start position.
        from: u32,
        /// End position.
        to: u32,
    },
    /// Split the block at a position.
    #[serde(rename = "split_block", alias = "splitBlock")]
    SplitBlock {
        /// Split position.
        at: u32,
    },
    /// Join the block at a position with its previous sibling.
    #[serde(rename = "join_backward", alias = "joinBackward")]
    JoinBackward {
        /// Boundary position.
        at: u32,
    },
    /// Toggle a mark over a range.
    #[serde(rename = "toggle_mark", alias = "toggleMark")]
    ToggleMark {
        /// Start position.
        from: u32,
        /// End position.
        to: u32,
        /// Declared mark name.
        mark: String,
        /// Explicit desired state; absent toggles.
        #[serde(default)]
        active: Option<bool>,
    },
    /// Set an attribute on the top-level block containing a position.
    #[serde(
        rename = "set_block_attr",
        alias = "setBlockAttr",
        alias = "set_node_attr",
        alias = "setNodeAttr"
    )]
    SetBlockAttr {
        /// Position inside the block.
        at: u32,
        /// Declared attribute name.
        #[serde(alias = "attr")]
        name: String,
        /// Attribute value.
        value: Value,
    },
    /// Set a document-root attribute.
    #[serde(rename = "set_doc_attr", alias = "setDocAttr")]
    SetDocAttr {
        /// Declared document attribute name.
        #[serde(alias = "attr")]
        name: String,
        /// Attribute value.
        value: Value,
    },
    /// Set an attribute on one inline atom.
    #[serde(rename = "set_inline_attr", alias = "setInlineAttr")]
    SetInlineAttr {
        /// Atom start position.
        from: u32,
        /// Atom end position.
        to: u32,
        /// Declared attribute name.
        #[serde(alias = "attr")]
        name: String,
        /// Attribute value.
        value: Value,
    },
    /// Set an attribute on runs that already carry a mark.
    #[serde(rename = "set_mark_attr", alias = "setMarkAttr")]
    SetMarkAttr {
        /// Start position.
        from: u32,
        /// End position.
        to: u32,
        /// Declared mark name.
        mark: String,
        /// Declared attribute name.
        #[serde(alias = "attr")]
        name: String,
        /// Attribute value.
        value: Value,
    },
    /// Replace the whole document.
    #[serde(rename = "replace_document", alias = "replaceDocument")]
    ReplaceDocument {
        /// Canonical document JSON.
        doc: Value,
    },
    /// Replace top-level blocks with a fragment, or insert it when empty.
    #[serde(rename = "replace_fragment", alias = "replaceFragment")]
    ReplaceFragment {
        /// Replacement start; defaults to `at`.
        #[serde(default)]
        from: Option<u32>,
        /// Replacement end; defaults to `at`.
        #[serde(default)]
        to: Option<u32>,
        /// Insertion position for an empty range.
        #[serde(default)]
        at: Option<u32>,
        /// Canonical fragment JSON.
        fragment: Value,
    },
    /// Set the selection without creating history.
    #[serde(rename = "set_selection", alias = "setSelection")]
    SetSelection {
        /// New selection.
        selection: SelectionJson,
    },
    /// Change the type of the top-level block containing a position.
    #[serde(rename = "set_block_type", alias = "setBlockType")]
    SetBlockType {
        /// Position inside the block.
        #[serde(alias = "pos")]
        at: u32,
        /// Declared node kind name.
        kind: String,
        /// Block attributes by name.
        #[serde(default)]
        attrs: Option<Value>,
    },
    /// Set or clear the stable identifier of a block.
    #[serde(rename = "set_node_id", alias = "setNodeId")]
    SetNodeId {
        /// Position inside the block.
        at: u32,
        /// New identifier, or `null` to clear it.
        #[serde(default)]
        id: Option<String>,
    },
    /// Replace a range with a slice.
    #[serde(rename = "replace_slice", alias = "replaceSlice")]
    ReplaceSlice {
        /// Start position.
        from: u32,
        /// End position.
        to: u32,
        /// Slice JSON.
        slice: Value,
    },
}

/// Construction options accepted by `fromJSON` and `fromEnvelope`.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(crate) struct EditorOptionsJson {
    /// Replacement `(max_groups, max_bytes)` history budgets.
    #[serde(default, alias = "historyLimits")]
    pub(crate) history_limits: Option<(usize, usize)>,
}

/// Batch envelope accepted by `apply` and `dryRun`.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct BatchPayload {
    /// Ordered operations applied atomically.
    pub ops: Vec<Op>,
    /// Document version the batch was computed against.
    #[serde(default, alias = "baseVersion")]
    pub base_version: Option<u32>,
    /// History origin name (`user`, `paste`, `remote`, `api`, `import`).
    #[serde(default)]
    pub origin: Option<String>,
    /// Explicit history group identifier.
    #[serde(default)]
    pub group: Option<u32>,
    /// Whether the batch enters history; defaults to `true`.
    #[serde(default, alias = "addToHistory")]
    pub add_to_history: Option<bool>,
}
