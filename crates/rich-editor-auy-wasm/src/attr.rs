//! Attribute value decoding and schema attribute resolution.

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::model::{
    AttrValue, Doc, DocId, InlineItem, MarkId, NodeId, NodeKind, RefTarget, VersionTag,
};
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::AttrType;
use serde_json::Value;

use crate::problem::invalid_attr_problem;

/// Decodes a JSON value into the canonical attribute value for a declared type.
pub(crate) fn value_from_json(
    kind: &AttrType,
    value: &Value,
    name: &str,
) -> Result<AttrValue, ProblemDetails> {
    match kind {
        AttrType::Str | AttrType::Enum(_) => value
            .as_str()
            .map(|text| AttrValue::Str(Box::from(text)))
            .ok_or_else(|| invalid_attr_problem(name, "value")),
        AttrType::Int { .. } => value
            .as_i64()
            .map(AttrValue::Int)
            .ok_or_else(|| invalid_attr_problem(name, "value")),
        AttrType::Bool => value
            .as_bool()
            .map(AttrValue::Bool)
            .ok_or_else(|| invalid_attr_problem(name, "value")),
        AttrType::Ref => ref_from_json(value, name).map(AttrValue::Ref),
        AttrType::List(inner) => {
            let items = value
                .as_array()
                .ok_or_else(|| invalid_attr_problem(name, "value"))?;
            let mut decoded = Vec::with_capacity(items.len());
            for item in items {
                decoded.push(value_from_json(inner, item, name)?);
            }
            Ok(AttrValue::Array(decoded.into_boxed_slice()))
        }
        AttrType::Map(inner) => {
            let members = value
                .as_object()
                .ok_or_else(|| invalid_attr_problem(name, "value"))?;
            let mut entries = Vec::with_capacity(members.len());
            for (key, member) in members {
                entries.push((
                    Box::from(key.as_str()),
                    value_from_json(inner, member, name)?,
                ));
            }
            AttrValue::map(entries).ok_or_else(|| invalid_attr_problem(name, "value"))
        }
        AttrType::Any => any_from_json(value, name),
    }
}

/// Decodes a reference target.
fn ref_from_json(value: &Value, name: &str) -> Result<RefTarget, ProblemDetails> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid_attr_problem(name, "value"))?;
    let doc = object
        .get("doc")
        .and_then(Value::as_str)
        .and_then(DocId::new)
        .ok_or_else(|| invalid_attr_problem(name, "value"))?;
    let node = match object.get("node") {
        None => None,
        Some(raw) => Some(
            raw.as_str()
                .and_then(NodeId::new)
                .ok_or_else(|| invalid_attr_problem(name, "value"))?,
        ),
    };
    let version = match object.get("version") {
        None => None,
        Some(raw) => Some(
            raw.as_str()
                .and_then(VersionTag::new)
                .ok_or_else(|| invalid_attr_problem(name, "value"))?,
        ),
    };
    let fragment = match object.get("fragment") {
        None => None,
        Some(raw) => {
            let range = raw
                .as_object()
                .ok_or_else(|| invalid_attr_problem(name, "value"))?;
            let start = range
                .get("start")
                .and_then(Value::as_u64)
                .and_then(|number| u32::try_from(number).ok())
                .ok_or_else(|| invalid_attr_problem(name, "value"))?;
            let end = range
                .get("end")
                .and_then(Value::as_u64)
                .and_then(|number| u32::try_from(number).ok())
                .ok_or_else(|| invalid_attr_problem(name, "value"))?;
            if start > end {
                return Err(invalid_attr_problem(name, "value"));
            }
            Some(Range::new(Pos(start), Pos(end)))
        }
    };
    Ok(RefTarget {
        doc,
        node,
        version,
        fragment,
    })
}

/// Decodes an untyped canonical value.
fn any_from_json(value: &Value, name: &str) -> Result<AttrValue, ProblemDetails> {
    match value {
        Value::Null => Ok(AttrValue::Null),
        Value::Bool(flag) => Ok(AttrValue::Bool(*flag)),
        Value::Number(number) => number
            .as_i64()
            .map(AttrValue::Int)
            .ok_or_else(|| invalid_attr_problem(name, "value")),
        Value::String(text) => Ok(AttrValue::Str(Box::from(text.as_str()))),
        Value::Array(items) => {
            let mut decoded = Vec::with_capacity(items.len());
            for item in items {
                decoded.push(any_from_json(item, name)?);
            }
            Ok(AttrValue::Array(decoded.into_boxed_slice()))
        }
        Value::Object(members) => {
            if members.contains_key("doc") {
                return ref_from_json(value, name).map(AttrValue::Ref);
            }
            let mut entries = Vec::with_capacity(members.len());
            for (key, member) in members {
                entries.push((Box::from(key.as_str()), any_from_json(member, name)?));
            }
            AttrValue::map(entries).ok_or_else(|| invalid_attr_problem(name, "value"))
        }
    }
}

/// Returns the kind of the top-level block containing a position.
pub(crate) fn block_kind_at(doc: &Doc, pos: Pos) -> Option<NodeKind> {
    doc.block_at(pos).map(|(_, block)| block.kind())
}

/// Returns the kind of the inline atom covered by a one-byte range.
pub(crate) fn atom_kind_at(doc: &Doc, from: Pos, to: Pos) -> Option<NodeKind> {
    if from >= to || to.0 != from.0.checked_add(1)? {
        return None;
    }
    let (index, block) = doc.block_at(from)?;
    let range = doc.block_range(u32::try_from(index).ok()?)?;
    let local = from.0.checked_sub(range.start.0.checked_add(1)?)?;
    let text = block.as_textblock()?;
    let mut cursor = 0_u32;
    for item in text.items() {
        if cursor == local {
            return match item {
                InlineItem::Atom(atom) => Some(atom.kind),
                InlineItem::Text(_) => None,
            };
        }
        cursor = cursor.checked_add(item.size())?;
    }
    None
}

/// Returns whether every text run overlapping a range carries a mark.
pub(crate) fn range_has_mark(doc: &Doc, range: Range, mark: MarkId) -> bool {
    let mut touched = false;
    for (index, block) in doc.blocks().iter().enumerate() {
        let Ok(index) = u32::try_from(index) else {
            return false;
        };
        let Some(block_range) = doc.block_range(index) else {
            return false;
        };
        if block_range.end.0 <= range.start.0 || block_range.start.0 >= range.end.0 {
            continue;
        }
        let Some(text) = block.as_textblock() else {
            continue;
        };
        let content_start = block_range.start.0.saturating_add(1);
        let local_start = range.start.0.saturating_sub(content_start);
        let local_end = range.end.0.saturating_sub(content_start).min(text.len().0);
        let mut cursor = 0_u32;
        for item in text.items() {
            let size = match item {
                InlineItem::Text(run) => u32::try_from(run.text.len()).unwrap_or(u32::MAX),
                InlineItem::Atom(_) => 1,
            };
            let item_end = cursor.saturating_add(size);
            if item_end > local_start && cursor < local_end {
                touched = true;
                match item {
                    InlineItem::Text(run) if run.marks.contains(mark) => {}
                    _ => return false,
                }
            }
            cursor = item_end;
        }
    }
    touched
}
