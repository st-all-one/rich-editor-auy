//! RFC 9457 problem details for rejected ingestion.

use alloc::boxed::Box;
use core::fmt;

/// Base URI shared by every canonical problem type.
pub const PROBLEM_BASE: &str = "https://rich-editor-auy.dev/problems/";

/// Structured failure returned when tokens cannot be represented by the schema.
///
/// The field names mirror RFC 9457. The wire spelling of [`Self::type_uri`] is `type`,
/// which is a reserved word in Rust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProblemDetails {
    /// RFC 9457 `type` URI.
    pub type_uri: Box<str>,
    /// Short human-readable summary.
    pub title: Box<str>,
    /// HTTP status code carried by the problem.
    pub status: u16,
    /// Human-readable explanation.
    pub detail: Box<str>,
    /// Pointer to the offending value, for example `blocks[3].kind`.
    pub instance: Option<Box<str>>,
    /// Stable machine-readable code.
    pub code: Box<str>,
    /// Actionable correction advice.
    pub suggestion: Option<Box<str>>,
}

impl ProblemDetails {
    /// Reports a token that the full schema cannot represent.
    #[must_use]
    pub fn unknown_token(token: &str, instance: &str) -> Self {
        Self::new(
            "unknown-token",
            "Unknown AUY token",
            422,
            "unknown_token",
            alloc::format!("The token '{token}' has no representation in the full schema."),
            instance,
            suggestion_for_token(token),
        )
    }

    /// Reports an attribute that the full schema does not declare.
    #[must_use]
    pub fn unknown_attr(name: &str, instance: &str) -> Self {
        Self::new(
            "unknown-attribute",
            "Unknown AUY attribute",
            422,
            "unknown_attr",
            alloc::format!("The attribute '{name}' is not declared for this token."),
            instance,
            Box::from("Remove the attribute or map it to a declared attribute before ingestion."),
        )
    }

    /// Reports a mark that the full schema does not declare.
    #[must_use]
    pub fn unknown_mark(name: &str, instance: &str) -> Self {
        Self::new(
            "unknown-mark",
            "Unknown AUY mark",
            422,
            "unknown_mark",
            alloc::format!("The mark '{name}' has no representation in the full schema."),
            instance,
            Box::from("Remove the mark or map it to a declared mark before ingestion."),
        )
    }

    /// Reports a feature with no semantic representation, such as a page break.
    #[must_use]
    pub fn unrepresentable_feature(feature: &str, instance: &str) -> Self {
        Self::new(
            "unrepresentable-feature",
            "Unrepresentable feature",
            422,
            "unrepresentable_feature",
            alloc::format!("The feature '{feature}' cannot be represented by the full schema."),
            instance,
            suggestion_for_token(feature),
        )
    }

    /// Reports an exceeded structural or size limit.
    #[must_use]
    pub fn limit_exceeded(limit: &str, instance: &str) -> Self {
        Self::new(
            "limit-exceeded",
            "Limit exceeded",
            413,
            "limit_exceeded",
            alloc::format!("The input exceeds the configured {limit} limit."),
            instance,
            Box::from("Reduce the payload before ingestion."),
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one constructor mirrors the RFC 9457 fields"
    )]
    fn new(
        problem: &str,
        title: &str,
        status: u16,
        code: &str,
        detail: alloc::string::String,
        instance: &str,
        suggestion: Box<str>,
    ) -> Self {
        let type_uri = alloc::format!("{PROBLEM_BASE}{problem}");
        Self {
            type_uri: type_uri.into_boxed_str(),
            title: Box::from(title),
            status,
            detail: detail.into_boxed_str(),
            instance: Some(Box::from(instance)),
            code: Box::from(code),
            suggestion: Some(suggestion),
        }
    }
}

/// Returns the documented conversion advice for an unknown token.
#[must_use]
pub fn suggestion_for_token(token: &str) -> Box<str> {
    match token {
        "chart" | "graph" => Box::from("Render the chart as an image with alt text and a caption."),
        "formula" | "math" => {
            Box::from("Render the formula as an image and use the formula text as alt.")
        }
        "page_break" | "page-break" => Box::from(
            "Page breaks are not representable; remove them or use a separator if the \
             product accepts different semantics.",
        ),
        "xml" | "html" | "raw" | "code" => {
            Box::from("Import the payload as raw_code with language and format, never as markup.")
        }
        "text_box" | "textbox" => {
            Box::from("Flatten the text box into a paragraph or blockquote in reading order.")
        }
        "comment" => Box::from("Remove the comment or export it separately."),
        "tracked_change" => Box::from("Accept or reject tracked changes before ingestion."),
        "footnote" => {
            Box::from("Footnotes have no schema node; move the text into the body or raw_code.")
        }
        _ => Box::from(
            "Map the token to a declared node, mark, atom, or attribute before ingestion, \
             or omit it.",
        ),
    }
}

impl fmt::Display for ProblemDetails {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({}): {}", self.title, self.code, self.detail)?;
        if let Some(instance) = &self.instance {
            write!(formatter, " at {instance}")?;
        }
        if let Some(suggestion) = &self.suggestion {
            write!(formatter, "; {suggestion}")?;
        }
        Ok(())
    }
}

impl core::error::Error for ProblemDetails {}
