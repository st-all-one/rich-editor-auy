//! Mark declarations.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::a11y::A11ySpec;
use crate::model::{AttrId, MarkId};

use super::AttrSpec;

/// Declaration of one mark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkSpec {
    /// Stable name.
    pub name: Box<str>,
    /// Groups this mark belongs to.
    pub groups: Vec<Box<str>>,
    /// Whether the mark extends over inserted text at its boundary.
    pub inclusive: bool,
    /// Marks that cannot coexist with this one.
    pub excludes: Vec<Box<str>>,
    /// Declared attributes.
    pub attrs: Vec<AttrSpec>,
    /// Accessibility declaration.
    pub a11y: Option<A11ySpec>,
    /// Assigned mark id.
    pub id: MarkId,
}

impl MarkSpec {
    /// Creates an inclusive mark without attributes.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: Box::from(name),
            groups: Vec::new(),
            inclusive: true,
            excludes: Vec::new(),
            attrs: Vec::new(),
            a11y: None,
            id: MarkId(0),
        }
    }

    /// Sets inclusivity.
    #[must_use]
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "fluent setter mirrors the schema flag"
    )]
    pub const fn inclusive(mut self, inclusive: bool) -> Self {
        self.inclusive = inclusive;
        self
    }

    /// Adds an exclusion.
    #[must_use]
    pub fn excludes(mut self, name: &str) -> Self {
        self.excludes.push(Box::from(name));
        self
    }

    /// Adds an attribute.
    #[must_use]
    pub fn attr(mut self, mut attr: AttrSpec) -> Self {
        let next = u16::try_from(self.attrs.len()).unwrap_or(u16::MAX);
        attr.id = AttrId(next);
        self.attrs.push(attr);
        self
    }

    /// Sets the accessibility declaration.
    #[must_use]
    pub const fn a11y(mut self, spec: A11ySpec) -> Self {
        self.a11y = Some(spec);
        self
    }
}
