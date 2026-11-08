//! Mark-derived accessibility roles and attributes.

use alloc::boxed::Box;

use crate::model::MarkSet;
use crate::schema::Schema;

use super::build::attr_string;
use super::role::A11yRole;

pub(super) fn mark_attr(
    schema: &Schema,
    marks: &MarkSet,
    mark: &str,
    attr: &str,
) -> Option<Box<str>> {
    let id = schema.mark_id(mark)?;
    let spec = schema.mark(id)?;
    let attr_spec = spec.attrs.iter().find(|spec| &*spec.name == attr)?;
    let attrs = marks.attrs_for(id)?;
    attr_string(attrs.get(attr_spec.id))
}

pub(super) fn mark_role(schema: &Schema, marks: &MarkSet) -> A11yRole {
    for spec in schema.marks() {
        if marks.contains(spec.id)
            && let Some(a11y) = spec.a11y
            && a11y.role != A11yRole::Text
        {
            return a11y.role;
        }
    }
    A11yRole::Text
}

pub(super) fn mark_href(schema: &Schema, marks: &MarkSet) -> Option<Box<str>> {
    for spec in schema.marks() {
        if marks.contains(spec.id)
            && schema
                .mark(spec.id)
                .and_then(|mark| mark.attrs.iter().find(|attr| &*attr.name == "href"))
                .is_some()
        {
            return mark_attr(schema, marks, &spec.name, "href");
        }
    }
    None
}
