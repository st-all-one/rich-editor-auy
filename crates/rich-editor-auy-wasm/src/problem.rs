//! RFC 9457 problem constructors for every WASM boundary rejection.
//!
//! Hosts never receive implementation strings: every failure is serialized as a
//! [`ProblemDetails`] JSON object with a stable machine-readable `code`, an `instance`
//! pointer, and corrective advice when one is known.

use rich_editor_auy::error::{ProblemDetails, suggestion_for_token};

use crate::problem_map::build;

/// Reports a token that the full schema cannot represent.
pub(crate) fn token_problem(token: &str, instance: &str) -> ProblemDetails {
    build(
        "unknown-token",
        "Unknown AUY token",
        422,
        "unknown_token",
        format!("The token '{token}' has no representation in the full schema."),
        Some(instance),
        Some(String::from(suggestion_for_token(token))),
    )
}

/// Reports a transaction that was already applied.
pub(crate) fn consumed_transaction() -> ProblemDetails {
    build(
        "consumed-transaction",
        "Consumed transaction",
        409,
        "consumed_transaction",
        String::from("The transaction was already applied; build a new one."),
        Some("transaction"),
        Some(String::from(
            "Start a new transaction against the current version.",
        )),
    )
}

/// Reports an unknown history origin name.
pub(crate) fn unknown_origin(origin: &str) -> ProblemDetails {
    build(
        "unknown-origin",
        "Unknown history origin",
        422,
        "unknown_origin",
        format!("The origin '{origin}' is not a history origin."),
        Some("origin"),
        Some(String::from("Use user, paste, remote, api, or import.")),
    )
}

/// Reports a command that needs a selection but has none.
pub(crate) fn missing_selection() -> ProblemDetails {
    build(
        "missing-selection",
        "Missing selection",
        409,
        "missing_selection",
        String::from("The command needs a caret or a selected range."),
        Some("selection"),
        Some(String::from("Set a selection before running the command.")),
    )
}

/// Reports an unrepresentable token at a structural path.
pub(crate) fn unknown_token_problem(instance: &str) -> ProblemDetails {
    build(
        "unknown-token",
        "Unknown AUY token",
        422,
        "unknown_token",
        format!(
            "The input at {instance} contains a token with no representation in the full schema."
        ),
        Some(instance),
        Some(String::from(suggestion_for_token(""))),
    )
}

/// Reports a step that references an undeclared declaration.
pub(crate) fn unknown_declaration_problem(path: &str) -> ProblemDetails {
    build(
        "unknown-token",
        "Unknown AUY declaration",
        422,
        "unknown_token",
        format!("The step references an undeclared node, mark, or attribute at {path}."),
        Some(path),
        Some(String::from(suggestion_for_token(""))),
    )
}

/// Reports an attribute that the owning declaration does not define.
pub(crate) fn attr_problem(name: &str, instance: &str) -> ProblemDetails {
    build(
        "unknown-attribute",
        "Unknown AUY attribute",
        422,
        "unknown_attr",
        format!("The attribute '{name}' is not declared for this token."),
        Some(instance),
        Some(String::from(
            "Remove the attribute or map it to a declared attribute before ingestion.",
        )),
    )
}

/// Reports a URL scheme forbidden by the ingestion contract.
pub(crate) fn unsafe_url_problem(instance: &str) -> ProblemDetails {
    build(
        "unsafe-url",
        "Unsafe URL",
        422,
        "unsafe_url",
        format!("The URL at {instance} uses a forbidden scheme."),
        Some(instance),
        Some(String::from(
            "Use an http(s), mailto, tel, or relative URL; data: and javascript: are rejected.",
        )),
    )
}

/// Reports an attribute value that does not match its declared type.
pub(crate) fn invalid_attr_problem(name: &str, instance: &str) -> ProblemDetails {
    build(
        "invalid-attribute",
        "Invalid AUY attribute value",
        422,
        "invalid_attr",
        format!("The value of '{name}' does not match its declared attribute type."),
        Some(instance),
        Some(String::from(
            "Send the value in the type declared by the full schema.",
        )),
    )
}

/// Reports a limit that the payload exceeded.
pub(crate) fn limit_problem(limit: &str, instance: &str) -> ProblemDetails {
    build(
        "limit-exceeded",
        "Limit exceeded",
        413,
        "limit_exceeded",
        format!("The input exceeds the configured {limit} limit."),
        Some(instance),
        Some(String::from("Reduce the payload before ingestion.")),
    )
}

/// Reports an internal projection failure.
pub(crate) fn view_problem() -> ProblemDetails {
    build(
        "view-encoding",
        "View encoding failed",
        500,
        "view_encoding",
        String::from("The document could not be encoded as a render view."),
        Some("doc"),
        Some(String::from(
            "Validate the document against the full schema and retry.",
        )),
    )
}

/// Reports a batch computed against an outdated document version.
pub(crate) fn stale_problem(expected: u32, actual: u32) -> ProblemDetails {
    build(
        "stale-batch",
        "Stale batch",
        409,
        "stale_batch",
        format!("The batch targets version {expected} but the document is at version {actual}."),
        Some("base_version"),
        Some(String::from(
            "Reload the editor state and rebuild the batch.",
        )),
    )
}

/// Reports a batch encoded against a different schema fingerprint.
pub(crate) fn schema_mismatch_problem(expected: &str, actual: &str) -> ProblemDetails {
    build(
        "schema-mismatch",
        "Schema mismatch",
        409,
        "schema_mismatch",
        format!("The batch targets schema '{actual}' but this editor uses '{expected}'."),
        Some("schema"),
        Some(String::from(
            "Rebuild the payload against the active schema.",
        )),
    )
}

/// Reports an anchor whose import policy rejected it.
pub(crate) fn id_rejected_problem(instance: &str) -> ProblemDetails {
    build(
        "id-rejected",
        "Anchor rejected",
        422,
        "id_rejected",
        format!("The payload carries an anchor at {instance} and the id policy rejects anchors."),
        Some(instance),
        Some(String::from(
            "Strip anchors, use idPolicy \"regenerate\", or accept anchors with \"preserve\".",
        )),
    )
}

/// Reports an unusable anchor policy.
pub(crate) fn id_policy_problem(detail: &str) -> ProblemDetails {
    build(
        "invalid-id-policy",
        "Invalid id policy",
        422,
        "invalid_id_policy",
        String::from(detail),
        Some("options.idPolicy"),
        Some(String::from(
            "Use idPolicy preserve, regenerate, or reject; regenerate needs idSeed.",
        )),
    )
}

/// Reports a malformed batch envelope.
pub(crate) fn malformed_problem(detail: String, instance: &str) -> ProblemDetails {
    build(
        "malformed-batch",
        "Malformed batch",
        400,
        "malformed_batch",
        detail,
        Some(instance),
        Some(String::from(
            "Send JSON matching the documented batch shape.",
        )),
    )
}

/// Reports an unsupported import mode.
pub(crate) fn mode_problem(mode: &str) -> ProblemDetails {
    build(
        "invalid-mode",
        "Invalid import mode",
        400,
        "invalid_mode",
        format!("The import mode '{mode}' is not supported."),
        Some("mode"),
        Some(String::from(
            "Use 'document', 'fragment', 'replace', 'insert', or 'ops'.",
        )),
    )
}

pub(crate) use crate::problem_map::{
    from_edit_error, from_limit_error, from_parse_error, from_schema_error, from_selection_error,
    from_step_error, problem_json, problem_value,
};
