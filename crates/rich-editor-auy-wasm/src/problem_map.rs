//! Mapping from core failures to RFC 9457 problem details.

use rich_editor_auy::error::{
    EditError, LimitError, PROBLEM_BASE, ParseError, ProblemDetails, SchemaError, SelectionError,
    StepError,
};
use serde_json::{Map, Value};

use crate::problem::{
    attr_problem, invalid_attr_problem, limit_problem, unknown_declaration_problem,
    unknown_token_problem, unsafe_url_problem,
};

/// Builds a problem with explicit RFC 9457 fields.
#[allow(
    clippy::too_many_arguments,
    reason = "one constructor mirrors the RFC 9457 fields"
)]
pub(crate) fn build(
    kind: &str,
    title: &str,
    status: u16,
    code: &str,
    detail: String,
    instance: Option<&str>,
    suggestion: Option<String>,
) -> ProblemDetails {
    ProblemDetails {
        type_uri: format!("{PROBLEM_BASE}{kind}").into_boxed_str(),
        title: Box::from(title),
        status,
        detail: detail.into_boxed_str(),
        instance: instance.map(Box::from),
        code: Box::from(code),
        suggestion: suggestion.map(String::into_boxed_str),
    }
}

/// Maps a parse failure to a problem object.
pub(crate) fn from_parse_error(error: &ParseError) -> ProblemDetails {
    let instance = if error.path.is_empty() {
        String::from("input")
    } else {
        error.path.to_string()
    };
    match &*error.code {
        "unknown_node" | "unknown_inline" => unknown_token_problem(&instance),
        "unknown_attr" => attr_problem(&instance, &instance),
        "unknown_mark" => ProblemDetails::unknown_mark(&instance, &instance),
        "unsafe_url" => unsafe_url_problem(&instance),
        "limit" | "depth_limit" | "run_limit" => limit_problem("document size", &instance),
        "attr_type"
        | "expected_string"
        | "expected_int"
        | "expected_bool"
        | "expected_array"
        | "expected_map"
        | "expected_ref"
        | "invalid_ref_doc"
        | "invalid_ref_node"
        | "invalid_ref_version"
        | "invalid_ref_fragment"
        | "required_attr"
        | "duplicate_attr"
        | "duplicate_map_key" => invalid_attr_problem(&instance, &instance),
        code => build(
            "invalid-wire",
            "Invalid wire data",
            400,
            code,
            format!("{error} at byte offset {}.", error.offset),
            Some(&instance),
            None,
        ),
    }
}

/// Maps a step failure to a problem object.
pub(crate) fn from_step_error(error: &StepError) -> ProblemDetails {
    match error {
        StepError::OutOfBounds { position, size } => build(
            "out-of-bounds",
            "Position outside document",
            400,
            "out_of_bounds",
            format!("Position {position} is outside document size {size}."),
            None,
            Some(String::from("Clamp the position to the document size.")),
        ),
        StepError::Unknown { path } => unknown_declaration_problem(path),
        StepError::ContentMismatch { path } => build(
            "content-mismatch",
            "Content mismatch",
            422,
            "content_mismatch",
            format!("The edit violates the schema content expression at {path}."),
            Some(path),
            Some(String::from(
                "Move the node to a position accepted by the schema.",
            )),
        ),
        StepError::DuplicateId { id } => build(
            "duplicate-id",
            "Duplicate identifier",
            409,
            "duplicate_id",
            format!("The identifier '{id}' is already present in the document."),
            Some("id"),
            Some(String::from(
                "Use 'preserve' with unique ids, 'regenerate', or 'reject'.",
            )),
        ),
        StepError::Unsupported { operation } => build(
            "unsupported-step",
            "Unsupported step",
            422,
            "unsupported_step",
            format!("The operation '{operation}' is not representable by the step algebra."),
            Some(operation),
            Some(String::from(
                "Express the change with steps the editor supports.",
            )),
        ),
        StepError::Limit(error) => from_limit_error(error),
        StepError::Invariant(error) => build(
            "invariant",
            "Internal invariant violated",
            500,
            "invariant",
            format!("{error}"),
            None,
            None,
        ),
    }
}

/// Maps a configured limit failure to a problem object.
pub(crate) fn from_limit_error(error: &LimitError) -> ProblemDetails {
    limit_problem(&format!("{error}"), "limits")
}

/// Maps a selection failure to a problem object.
pub(crate) fn from_selection_error(error: &SelectionError) -> ProblemDetails {
    let code = if error.reason.is_empty() {
        "invalid_selection"
    } else {
        error.reason.as_ref()
    };
    build(
        "invalid-selection",
        "Invalid selection",
        422,
        code,
        format!("The selection is invalid at position {}.", error.position),
        Some("selection"),
        Some(String::from("Clamp the selection to the document size.")),
    )
}

/// Maps a schema failure to a problem object.
pub(crate) fn from_schema_error(error: &SchemaError) -> ProblemDetails {
    build(
        "invalid-schema",
        "Invalid schema",
        500,
        "schema_invalid",
        format!("{error}"),
        None,
        None,
    )
}

/// Maps a top-level editing failure to a problem object.
pub(crate) fn from_edit_error(error: &EditError) -> ProblemDetails {
    match error {
        EditError::Schema(error) => from_schema_error(error),
        EditError::Step(error) => from_step_error(error),
        EditError::Selection(error) => from_selection_error(error),
        EditError::Limit(error) => from_limit_error(error),
        EditError::Parse(error) => from_parse_error(error),
        EditError::Invariant(error) => build(
            "invariant",
            "Internal invariant violated",
            500,
            "invariant",
            format!("{error}"),
            None,
            None,
        ),
    }
}

/// Returns the RFC 9457 JSON value for a problem.
pub(crate) fn problem_value(problem: &ProblemDetails) -> Value {
    let mut map = Map::new();
    map.insert(
        String::from("type"),
        Value::String(problem.type_uri.to_string()),
    );
    map.insert(
        String::from("title"),
        Value::String(problem.title.to_string()),
    );
    map.insert(String::from("status"), Value::Number(problem.status.into()));
    map.insert(
        String::from("detail"),
        Value::String(problem.detail.to_string()),
    );
    if let Some(instance) = &problem.instance {
        map.insert(
            String::from("instance"),
            Value::String(instance.to_string()),
        );
    }
    map.insert(
        String::from("code"),
        Value::String(problem.code.to_string()),
    );
    if let Some(suggestion) = &problem.suggestion {
        map.insert(
            String::from("suggestion"),
            Value::String(suggestion.to_string()),
        );
    }
    Value::Object(map)
}

/// Serializes a problem as JSON text.
pub(crate) fn problem_json(problem: &ProblemDetails) -> String {
    problem_value(problem).to_string()
}
