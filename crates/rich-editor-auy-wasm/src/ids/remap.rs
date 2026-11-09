//! Reference remapping applied after anchors are regenerated.

use std::collections::BTreeMap;

use rich_editor_auy::error::ProblemDetails;
use rich_editor_auy::model::{
    Attr, AttrValue, Attrs, InlineAtom, InlineItem, Mark, MarkSet, NodeId, Run, TextBlock,
};
use rich_editor_auy::schema::Schema;

use crate::problem;

/// Old anchor to new anchor, used to rewrite references inside the payload.
pub(super) type Remap = BTreeMap<Box<str>, Box<str>>;

/// Rewrites every reference to a regenerated anchor in a text block.
pub(super) fn rewrite_text(
    text: &TextBlock,
    remap: &Remap,
    schema: &Schema,
) -> Result<TextBlock, ProblemDetails> {
    let mut changed = false;
    let mut items = Vec::with_capacity(text.items().len());
    for item in text.items() {
        match item {
            InlineItem::Text(run) => {
                if run.marks.is_empty() {
                    items.push(item.clone());
                    continue;
                }
                let mut marks = MarkSet::empty();
                for spec in schema.marks() {
                    if !run.marks.contains(spec.id) {
                        continue;
                    }
                    let attrs = run
                        .marks
                        .attrs_for(spec.id)
                        .cloned()
                        .unwrap_or_else(Attrs::empty);
                    let attrs = match remap_attrs(&attrs, remap)? {
                        Some(attrs) => attrs,
                        None => attrs,
                    };
                    marks = marks.with_mark(Mark { id: spec.id, attrs });
                }
                if marks == run.marks {
                    items.push(item.clone());
                } else {
                    changed = true;
                    items.push(InlineItem::Text(Run::new(run.text.clone(), marks)));
                }
            }
            InlineItem::Atom(atom) => match remap_attrs(&atom.attrs, remap)? {
                Some(attrs) => {
                    changed = true;
                    items.push(InlineItem::Atom(InlineAtom {
                        kind: atom.kind,
                        attrs,
                    }));
                }
                None => items.push(item.clone()),
            },
        }
    }
    if changed {
        Ok(TextBlock::new(items))
    } else {
        Ok(text.clone())
    }
}

/// Rewrites reference values in one attribute collection; `None` means nothing changed.
pub(super) fn remap_attrs(attrs: &Attrs, remap: &Remap) -> Result<Option<Attrs>, ProblemDetails> {
    let mut values = Vec::with_capacity(attrs.len());
    let mut changed = false;
    for attr in attrs {
        match remap_value(&attr.value, remap) {
            Some(value) => {
                changed = true;
                values.push(Attr { id: attr.id, value });
            }
            None => values.push(attr.clone()),
        }
    }
    if !changed {
        return Ok(None);
    }
    Attrs::new(values)
        .map(Some)
        .ok_or_else(|| problem::id_policy_problem("duplicate attribute while remapping references"))
}

/// Rewrites a reference target; `None` means nothing changed.
fn remap_value(value: &AttrValue, remap: &Remap) -> Option<AttrValue> {
    match value {
        AttrValue::Ref(target) => {
            let node = target.node.as_ref()?;
            let new = remap.get(node.as_str())?;
            let mut next = target.clone();
            next.node = Some(NodeId::new(new)?);
            Some(AttrValue::Ref(next))
        }
        AttrValue::Array(items) => {
            let mut changed = false;
            let mut next = Vec::with_capacity(items.len());
            for item in items {
                match remap_value(item, remap) {
                    Some(item) => {
                        changed = true;
                        next.push(item);
                    }
                    None => next.push(item.clone()),
                }
            }
            changed.then(|| AttrValue::Array(next.into_boxed_slice()))
        }
        AttrValue::Map(entries) => {
            let mut changed = false;
            let mut next = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                match remap_value(value, remap) {
                    Some(value) => {
                        changed = true;
                        next.push((key.clone(), value));
                    }
                    None => next.push((key.clone(), value.clone())),
                }
            }
            if changed { AttrValue::map(next) } else { None }
        }
        _ => None,
    }
}
