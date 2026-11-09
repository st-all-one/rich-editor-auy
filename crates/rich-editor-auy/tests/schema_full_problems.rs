//! Full-schema ingestion problem-detail tests.

use rich_editor_auy::error::{ProblemDetails, suggestion_for_token};

#[test]
fn problem_details_describe_unknown_tokens_and_suggest_conversions() {
    let problem = ProblemDetails::unknown_token("chart", "blocks[3].kind");
    assert_eq!(
        &*problem.type_uri,
        "https://rich-editor-auy.dev/problems/unknown-token"
    );
    assert_eq!(&*problem.code, "unknown_token");
    assert_eq!(problem.status, 422);
    assert!(problem.detail.contains("chart"));
    assert_eq!(problem.instance.as_deref(), Some("blocks[3].kind"));
    let suggestion = problem.suggestion.as_deref().unwrap_or("");
    assert!(suggestion.contains("image"));

    let attr = ProblemDetails::unknown_attr("srcset", "attrs.srcset");
    assert_eq!(&*attr.code, "unknown_attr");
    assert!(attr.suggestion.is_some());
    let mark = ProblemDetails::unknown_mark("sparkle", "marks[0].type");
    assert_eq!(&*mark.code, "unknown_mark");
    assert!(mark.suggestion.is_some());
    assert!(suggestion_for_token("xml").contains("raw_code"));
}
