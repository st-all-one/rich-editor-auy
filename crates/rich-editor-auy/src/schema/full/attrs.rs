//! Attribute declarations for the complete schema.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::schema::{AttrSpec, AttrType, DocSpec};

/// Document root attributes: presentation, publication metadata, and editorial metadata.
pub(super) fn doc() -> DocSpec {
    DocSpec::new()
        .content("block+")
        .attr(str_attr("lang"))
        .attr(enum_attr("dir", &["ltr", "rtl", "auto"]))
        .attr(str_attr("title"))
        .attr(str_attr("description"))
        .attr(str_attr("canonical_url"))
        .attr(list_attr("authors", AttrType::Str))
        .attr(list_attr("tags", AttrType::Str))
        .attr(map_attr("metadata", AttrType::Str))
}

pub(super) fn str_attr(name: &str) -> AttrSpec {
    AttrSpec::new(name, AttrType::Str)
}

pub(super) fn int_attr(name: &str, min: i64, max: i64) -> AttrSpec {
    AttrSpec::new(name, AttrType::Int { min, max })
}

pub(super) fn bool_attr(name: &str) -> AttrSpec {
    AttrSpec::new(name, AttrType::Bool)
}

pub(super) fn enum_attr(name: &str, values: &[&str]) -> AttrSpec {
    let values = values
        .iter()
        .map(|value| Box::<str>::from(*value))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    AttrSpec::new(name, AttrType::Enum(values))
}

pub(super) fn list_attr(name: &str, inner: AttrType) -> AttrSpec {
    AttrSpec::new(name, AttrType::List(Box::new(inner)))
}

pub(super) fn map_attr(name: &str, inner: AttrType) -> AttrSpec {
    AttrSpec::new(name, AttrType::Map(Box::new(inner)))
}
