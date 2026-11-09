//! Declarative schema values.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::a11y::A11ySpec;
use crate::model::{AttrId, AttrValue, NodeKind};

/// Attribute value domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttrType {
    /// UTF-8 string.
    Str,
    /// Signed integer in an inclusive range.
    Int {
        /// Inclusive minimum.
        min: i64,
        /// Inclusive maximum.
        max: i64,
    },
    /// Boolean.
    Bool,
    /// Reference target.
    Ref,
    /// One of a closed set of strings.
    Enum(Box<[Box<str>]>),
    /// Canonical array whose items share one type.
    List(Box<Self>),
    /// Canonical object whose values share one type.
    Map(Box<Self>),
    /// Any canonical value.
    Any,
}

/// One declared attribute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttrSpec {
    /// Attribute index in its owning declaration.
    pub id: AttrId,
    /// Stable name.
    pub name: Box<str>,
    /// Value domain.
    pub kind: AttrType,
    /// Whether a value is mandatory.
    pub required: bool,
    /// Default value.
    pub default: Option<AttrValue>,
}

impl AttrSpec {
    /// Creates an optional attribute with default index zero.
    #[must_use]
    pub fn new(name: &str, kind: AttrType) -> Self {
        Self {
            id: AttrId(0),
            name: Box::from(name),
            kind,
            required: false,
            default: None,
        }
    }

    /// Marks the attribute as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Sets a default value.
    #[must_use]
    pub fn with_default(mut self, value: AttrValue) -> Self {
        self.default = Some(value);
        self
    }
}

/// Mark permission for a node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkMode {
    /// No marks are allowed.
    #[default]
    None,
    /// Every mark is allowed.
    All,
}

/// Declaration of one node kind.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "flags are independent schema semantics"
)]
pub struct NodeSpec {
    /// Stable name.
    pub name: Box<str>,
    /// Content expression, or `None` for a leaf.
    pub content: Option<Box<str>>,
    /// Groups this node belongs to.
    pub groups: Vec<Box<str>>,
    /// Whether the node is inline.
    pub inline: bool,
    /// Whether the node is atomic.
    pub atom: bool,
    /// Whether the node is a defining boundary.
    pub defining: bool,
    /// Whether the node isolates edits.
    pub isolating: bool,
    /// Mark permission.
    pub marks: MarkMode,
    /// Declared attributes.
    pub attrs: Vec<AttrSpec>,
    /// Accessibility declaration.
    pub a11y: Option<A11ySpec>,
    /// Assigned node kind.
    pub kind: NodeKind,
}

impl NodeSpec {
    /// Creates a leaf node with the reserved `id` attribute.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: Box::from(name),
            content: None,
            groups: Vec::new(),
            inline: false,
            atom: false,
            defining: false,
            isolating: false,
            marks: MarkMode::None,
            attrs: Vec::from([AttrSpec {
                id: AttrId::NODE_ID,
                name: Box::from("id"),
                kind: AttrType::Str,
                required: false,
                default: None,
            }]),
            a11y: None,
            kind: NodeKind(0),
        }
    }

    /// Sets the content expression.
    #[must_use]
    pub fn content(mut self, expression: &str) -> Self {
        self.content = Some(Box::from(expression));
        self
    }

    /// Adds a group membership.
    #[must_use]
    pub fn group(mut self, group: &str) -> Self {
        self.groups.push(Box::from(group));
        self
    }

    /// Marks the node as inline.
    #[must_use]
    pub const fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// Marks the node as atomic.
    #[must_use]
    pub const fn atom(mut self) -> Self {
        self.atom = true;
        self
    }

    /// Marks the node as defining.
    #[must_use]
    pub const fn defining(mut self) -> Self {
        self.defining = true;
        self
    }

    /// Marks the node as isolating.
    #[must_use]
    pub const fn isolating(mut self) -> Self {
        self.isolating = true;
        self
    }

    /// Allows all marks.
    #[must_use]
    pub const fn marks_all(mut self) -> Self {
        self.marks = MarkMode::All;
        self
    }

    /// Adds an attribute after the reserved `id` slot.
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

/// Declaration of the document root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocSpec {
    /// Root content expression.
    pub content: Box<str>,
    /// Root attributes.
    pub attrs: Vec<AttrSpec>,
    /// Accessibility declaration.
    pub a11y: Option<A11ySpec>,
}

impl Default for DocSpec {
    fn default() -> Self {
        Self {
            content: Box::from("block+"),
            attrs: Vec::new(),
            a11y: Some(A11ySpec::new(crate::a11y::A11yRole::Document)),
        }
    }
}

impl DocSpec {
    /// Creates a document declaration with `block+` content.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the root content expression.
    #[must_use]
    pub fn content(mut self, expression: &str) -> Self {
        self.content = Box::from(expression);
        self
    }

    /// Adds a root attribute.
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
