//! Unit tests for the JSON DTOs, problem mapping, and attribute decoding.

use rich_editor_auy::model::AttrValue;
use rich_editor_auy::schema::AttrType;
use serde_json::json;

use crate::attr;
use crate::dto::{BatchPayload, ImportPayload, Op};
use crate::ids::IdPolicy;
use crate::problem;

#[test]
fn parses_snake_and_camel_operation_names() {
    let value = json!({"ops": [
        {"op": "insertText", "at": 1, "text": "x"},
        {"op": "set_block_attr", "at": 0, "attr": "level", "value": 2},
    ]});
    let parsed = serde_json::from_value::<BatchPayload>(value);
    assert!(parsed.is_ok());
    let Ok(batch) = parsed else {
        return;
    };
    assert_eq!(batch.ops.len(), 2);
    assert!(matches!(
        batch.ops.first(),
        Some(Op::InsertText { at: 1, .. })
    ));
    assert!(matches!(
        batch.ops.get(1),
        Some(Op::SetBlockAttr { at: 0, name, .. }) if name == "level"
    ));
}

#[test]
fn rejects_unknown_operation_names() {
    let value = json!({"ops": [{"op": "explode", "at": 0}]});
    assert!(serde_json::from_value::<BatchPayload>(value).is_err());
}

#[test]
fn parses_import_base_version_alias() {
    let value = json!({"mode": "ops", "baseVersion": 7, "ops": []});
    let parsed = serde_json::from_value::<ImportPayload>(value);
    assert!(parsed.is_ok());
    let Ok(payload) = parsed else {
        return;
    };
    assert_eq!(payload.base_version, Some(7));
}

#[test]
fn problem_json_carries_code_and_suggestion() {
    let value = problem::problem_value(&problem::unknown_token_problem("blocks[3].kind"));
    assert_eq!(
        value.get("code").and_then(|code| code.as_str()),
        Some("unknown_token")
    );
    assert_eq!(
        value.get("instance").and_then(|instance| instance.as_str()),
        Some("blocks[3].kind")
    );
    assert!(value.get("suggestion").is_some());
    assert_eq!(
        value.get("status").and_then(serde_json::Value::as_u64),
        Some(422)
    );
}

#[test]
fn decodes_declared_attribute_values() {
    let string = attr::value_from_json(&AttrType::Str, &json!("text"), "language");
    assert!(matches!(string, Ok(AttrValue::Str(_))));

    let integer = attr::value_from_json(&AttrType::Int { min: 1, max: 6 }, &json!(3), "level");
    assert!(matches!(integer, Ok(AttrValue::Int(3))));

    assert!(
        attr::value_from_json(&AttrType::Int { min: 1, max: 6 }, &json!("3"), "level").is_err()
    );
    assert!(attr::value_from_json(&AttrType::Bool, &json!(true), "tight").is_ok());

    let list = attr::value_from_json(
        &AttrType::List(Box::new(AttrType::Str)),
        &json!(["a", "b"]),
        "tags",
    );
    assert!(matches!(list, Ok(AttrValue::Array(items)) if items.len() == 2));

    let map = attr::value_from_json(
        &AttrType::Map(Box::new(AttrType::Str)),
        &json!({"b": "2", "a": "1"}),
        "metadata",
    );
    assert!(
        matches!(map, Ok(AttrValue::Map(entries)) if entries.first().is_some_and(|entry| &*entry.0 == "a"))
    );

    let reference = attr::value_from_json(
        &AttrType::Ref,
        &json!({"doc": "doc-1", "node": "n1"}),
        "target",
    );
    assert!(matches!(reference, Ok(AttrValue::Ref(_))));
    assert!(attr::value_from_json(&AttrType::Ref, &json!({"node": "n1"}), "target").is_err());
}

#[test]
fn id_policy_defaults_to_preserve_and_requires_a_seed() {
    assert!(matches!(
        IdPolicy::parse(None, None),
        Ok(IdPolicy::Preserve)
    ));
    assert!(matches!(
        IdPolicy::parse(Some("reject"), None),
        Ok(IdPolicy::Reject)
    ));
    assert!(IdPolicy::parse(Some("regenerate"), None).is_err());
    assert!(IdPolicy::parse(Some("regenerate"), Some("")).is_err());
    assert!(matches!(
        IdPolicy::parse(Some("regenerate"), Some("seed")),
        Ok(IdPolicy::Regenerate(seed)) if seed == "seed"
    ));
    assert!(IdPolicy::parse(Some("shuffle"), None).is_err());
}

#[test]
fn parses_import_options_and_request_id() {
    let value = json!({
        "mode": "document",
        "doc": {"type": "doc", "content": []},
        "requestId": "req-1",
        "source": "panel-a",
        "destination": "note-9",
        "options": {
            "idPolicy": "regenerate",
            "idSeed": "s",
            "limits": {"maxBlocks": 4, "maxDepth": 8}
        }
    });
    let parsed = serde_json::from_value::<ImportPayload>(value);
    assert!(parsed.is_ok());
    let Ok(payload) = parsed else {
        return;
    };
    assert_eq!(payload.request_id.as_deref(), Some("req-1"));
    assert_eq!(payload.source.as_deref(), Some("panel-a"));
    assert_eq!(payload.destination.as_deref(), Some("note-9"));
    assert_eq!(payload.options.id_policy.as_deref(), Some("regenerate"));
    assert_eq!(payload.options.id_seed.as_deref(), Some("s"));
    assert_eq!(
        payload
            .options
            .limits
            .as_ref()
            .and_then(|limits| limits.max_blocks),
        Some(4)
    );
    assert_eq!(
        payload
            .options
            .limits
            .as_ref()
            .and_then(|limits| limits.max_depth),
        Some(8)
    );
}
