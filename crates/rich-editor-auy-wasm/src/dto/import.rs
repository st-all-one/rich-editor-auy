//! Import envelope and ingestion options.

use serde::Deserialize;
use serde_json::Value;

use super::Op;

/// Import envelope accepted by `import`.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ImportPayload {
    /// `document`, `fragment`, `replace`, `insert`, or `ops`.
    ///
    /// An empty mode is accepted for a bare `{"ops": [...]}` batch, which `dryRun` also takes.
    #[serde(default)]
    pub mode: String,
    /// Optional schema fingerprint the payload was built against.
    #[serde(default)]
    pub schema: Option<String>,
    /// Document token tree for `document` mode.
    #[serde(default)]
    pub doc: Option<Value>,
    /// Fragment token tree for `fragment`, `replace`, and `insert` modes.
    #[serde(default)]
    pub fragment: Option<Value>,
    /// Replacement start for `replace` mode.
    #[serde(default)]
    pub from: Option<u32>,
    /// Replacement end for `replace` mode.
    #[serde(default)]
    pub to: Option<u32>,
    /// Insertion position for `insert` mode.
    #[serde(default)]
    pub at: Option<u32>,
    /// Document version the batch was computed against.
    #[serde(default, alias = "baseVersion")]
    pub base_version: Option<u32>,
    /// Operations for `ops` mode.
    #[serde(default)]
    pub ops: Option<Vec<Op>>,
    /// Ingestion policy for anchors and resource limits.
    #[serde(default)]
    pub options: ImportOptions,
    /// Host correlation identifier echoed by the report.
    #[serde(default, rename = "requestId", alias = "operationId")]
    pub request_id: Option<String>,
    /// Free-form origin label for host audit logs.
    #[serde(default)]
    pub source: Option<String>,
    /// Free-form destination label for host audit logs.
    #[serde(default)]
    pub destination: Option<String>,
}

/// Ingestion options accepted by `import`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct ImportOptions {
    /// `preserve` (default), `regenerate`, or `reject`.
    pub id_policy: Option<String>,
    /// Deterministic seed used by `idPolicy: "regenerate"`.
    pub id_seed: Option<String>,
    /// Whether the import enters history; defaults to `true`.
    pub add_to_history: Option<bool>,
    /// Tightening overrides for the decoder limits.
    pub limits: Option<BatchLimits>,
}

/// Resource limits applied to one import payload.
#[allow(
    clippy::struct_field_names,
    reason = "the wire keys are maxBytes, maxBlocks, maxDepth, maxNodes, and maxRunsPerBlock"
)]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct BatchLimits {
    /// Maximum payload bytes.
    pub max_bytes: Option<u32>,
    /// Maximum top-level blocks after the import.
    pub max_blocks: Option<u32>,
    /// Maximum structural depth.
    pub max_depth: Option<u16>,
    /// Maximum node count.
    pub max_nodes: Option<u32>,
    /// Maximum runs per text block.
    pub max_runs_per_block: Option<u32>,
    /// Maximum operations in one `ops` import.
    pub max_ops: Option<u32>,
    /// Maximum table cells in the resulting document.
    pub max_table_cells: Option<u32>,
}
