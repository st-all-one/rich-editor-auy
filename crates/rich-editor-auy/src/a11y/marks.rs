//! Mark-derived accessibility roles and attributes.

use alloc::boxed::Box;

use crate::model::MarkSet;
use crate::schema::Schema;

use super::attrs::text;
use super::role::A11yRole;

/// Resolves the semantic role and link target of a run in one schema pass.
///
/// The role is the first mark with a non-text role in declaration order; the link is the value
/// of the first mark that declares an `href` attribute, even when that value is absent.
pub(super) fn marks_role_and_href(
    marks: &MarkSet,
    schema: &Schema,
) -> (A11yRole, Option<Box<str>>) {
    if marks.is_empty() {
        return (A11yRole::Text, None);
    }
    let mut role = None;
    let mut href = None;
    for spec in schema.marks() {
        if !marks.contains(spec.id) {
            continue;
        }
        if role.is_none()
            && let Some(a11y) = spec.a11y
            && a11y.role != A11yRole::Text
        {
            role = Some(a11y.role);
        }
        if href.is_none()
            && let Some(attr) = spec.attrs.iter().find(|attr| &*attr.name == "href")
        {
            href = Some(match marks.attrs_for(spec.id) {
                Some(attrs) => text(attrs, Some(attr.id)),
                None => None,
            });
        }
        if role.is_some() && href.is_some() {
            break;
        }
    }
    (role.unwrap_or(A11yRole::Text), href.flatten())
}
