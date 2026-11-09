//! Mark declarations for the complete schema.

use crate::a11y::{A11yRole, A11ySpec, NamePolicy};
use crate::schema::{AttrSpec, AttrType, MarkSpec};

use super::attrs::{enum_attr, str_attr};

pub(super) fn strong() -> MarkSpec {
    MarkSpec::new("strong")
}

pub(super) fn em() -> MarkSpec {
    MarkSpec::new("em")
}

pub(super) fn code() -> MarkSpec {
    MarkSpec::new("code").a11y(A11ySpec::new(A11yRole::Code).with_name(NamePolicy::TextContent))
}

pub(super) fn underline() -> MarkSpec {
    MarkSpec::new("underline")
}

pub(super) fn strike() -> MarkSpec {
    MarkSpec::new("strike")
}

pub(super) fn highlight() -> MarkSpec {
    MarkSpec::new("highlight")
}

pub(super) fn sup() -> MarkSpec {
    MarkSpec::new("sup")
}

pub(super) fn sub() -> MarkSpec {
    MarkSpec::new("sub")
}

pub(super) fn link() -> MarkSpec {
    MarkSpec::new("link")
        .attr(str_attr("href").required())
        .attr(str_attr("title"))
        .attr(str_attr("rel"))
        .attr(enum_attr("target", &["_self", "_blank", "_parent", "_top"]))
        .a11y(A11ySpec::new(A11yRole::Link).with_name(NamePolicy::TextContent))
}

pub(super) fn reference() -> MarkSpec {
    MarkSpec::new("ref")
        .attr(AttrSpec::new("target", AttrType::Ref).required())
        .a11y(A11ySpec::new(A11yRole::Link).with_name(NamePolicy::TextContent))
}
