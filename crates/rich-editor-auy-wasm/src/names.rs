//! Name-based attribute decoding for the JavaScript surface.

use rich_editor_auy::model::{Attr, Attrs, Mark, NodeKind};
use rich_editor_auy::schema::{AttrSpec, Schema};
use serde_json::Value;
use wasm_bindgen::JsValue;

use crate::attr;
use crate::problem;

/// Decodes a JSON object of names into node attributes.
pub(crate) fn node_attrs(
    schema: &Schema,
    kind: NodeKind,
    attrs: Option<String>,
) -> Result<Attrs, JsValue> {
    let mut decoded = Attrs::empty();
    let Some(raw) = attrs else {
        return Ok(decoded);
    };
    let value = json_value(&raw)?;
    let Value::Object(map) = value else {
        return Err(malformed("attrs must be an object", "attrs"));
    };
    for (name, item) in &map {
        let spec = schema
            .node_attr(kind, name)
            .ok_or_else(|| crate::problem_js(&problem::attr_problem(name, "node")))?;
        decoded = with_attr(&decoded, spec, item, name)?;
    }
    Ok(decoded)
}

/// Builds a mark from a name and optional JSON attributes.
pub(crate) fn mark_from_json(
    schema: &Schema,
    name: &str,
    attrs: Option<String>,
) -> Result<Mark, JsValue> {
    let id = schema
        .mark_id(name)
        .ok_or_else(|| crate::problem_js(&problem::token_problem(name, "mark")))?;
    let mut decoded = Attrs::empty();
    if let Some(raw) = attrs {
        let value = json_value(&raw)?;
        let Value::Object(map) = value else {
            return Err(malformed("attrs must be an object", "attrs"));
        };
        let spec = schema
            .mark(id)
            .ok_or_else(|| crate::problem_js(&problem::token_problem(name, "mark")))?;
        for (attr_name, item) in &map {
            let attr_spec = spec
                .attrs
                .iter()
                .find(|candidate| &*candidate.name == attr_name)
                .ok_or_else(|| crate::problem_js(&problem::attr_problem(attr_name, "mark")))?;
            decoded = with_attr(&decoded, attr_spec, item, attr_name)?;
        }
    }
    Ok(Mark { id, attrs: decoded })
}

/// Decodes one JSON value.
pub(crate) fn json_value(value: &str) -> Result<Value, JsValue> {
    serde_json::from_str(value).map_err(|error| {
        crate::problem_js(&problem::malformed_problem(format!("{error}"), "value"))
    })
}

/// Builds a malformed-payload problem.
fn malformed(detail: &str, instance: &str) -> JsValue {
    crate::problem_js(&problem::malformed_problem(String::from(detail), instance))
}

/// Decodes one attribute value and appends it to a set.
fn with_attr(attrs: &Attrs, spec: &AttrSpec, value: &Value, name: &str) -> Result<Attrs, JsValue> {
    let decoded = attr::value_from_json(&spec.kind, value, name)
        .map_err(|problem| crate::problem_js(&problem))?;
    attrs
        .with(Attr {
            id: spec.id,
            value: decoded,
        })
        .ok_or_else(|| crate::problem_js(&problem::invalid_attr_problem(name, "node")))
}
