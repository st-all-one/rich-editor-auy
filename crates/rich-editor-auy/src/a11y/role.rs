//! Semantic roles and schema-level accessibility policies.

use alloc::boxed::Box;

use crate::model::AttrId;

/// Opaque custom role index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleId(pub u16);

/// Semantic role, independent of HTML or ARIA spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum A11yRole {
    /// Document root.
    Document,
    /// Plain text leaf.
    Text,
    /// Paragraph.
    Paragraph,
    /// Heading with a level from 1 to 6.
    Heading {
        /// Heading level.
        level: u8,
    },
    /// List container.
    List,
    /// List item.
    ListItem,
    /// Block quote.
    Quote,
    /// Inline code.
    Code,
    /// Code block.
    CodeBlock,
    /// Table.
    Table,
    /// Table caption.
    TableCaption,
    /// Table row.
    TableRow,
    /// Table cell.
    TableCell {
        /// Whether the cell is a header.
        header: bool,
    },
    /// Link.
    Link,
    /// Image.
    Image,
    /// Figure.
    Figure,
    /// Caption.
    Caption,
    /// Video.
    Video,
    /// Audio.
    Audio,
    /// Attachment.
    Attachment,
    /// Separator.
    Separator,
    /// Role declared by the schema.
    Custom(RoleId),
}

impl A11yRole {
    /// Returns the stable role name used in diagnostics and fingerprints.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Text => "text",
            Self::Paragraph => "paragraph",
            Self::Heading { .. } => "heading",
            Self::List => "list",
            Self::ListItem => "list_item",
            Self::Quote => "quote",
            Self::Code => "code",
            Self::CodeBlock => "code_block",
            Self::Table => "table",
            Self::TableCaption => "table_caption",
            Self::TableRow => "table_row",
            Self::TableCell { header: true } => "table_header",
            Self::TableCell { header: false } => "table_cell",
            Self::Link => "link",
            Self::Image => "image",
            Self::Figure => "figure",
            Self::Caption => "caption",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Attachment => "attachment",
            Self::Separator => "separator",
            Self::Custom(_) => "custom",
        }
    }

    /// Returns whether this role needs a textual alternative.
    #[must_use]
    pub const fn requires_alt(self) -> bool {
        matches!(
            self,
            Self::Image | Self::Video | Self::Audio | Self::Attachment
        )
    }

    /// Returns whether this role needs a non-empty accessible name.
    #[must_use]
    pub const fn requires_name(self) -> bool {
        matches!(self, Self::Link)
    }
}

/// How an accessible name is produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamePolicy {
    /// No name is exposed.
    None,
    /// Derive from textual content.
    TextContent,
    /// Read a declared attribute.
    Attr(AttrId),
    /// The schema requires the host to provide a name.
    Required,
}

/// How a textual alternative is handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AltPolicy {
    /// The role has no alternative.
    NotApplicable,
    /// The host may provide an alternative.
    Optional,
    /// The host must provide an alternative.
    Required,
}

/// Accessibility declaration attached to a node or mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct A11ySpec {
    /// Semantic role.
    pub role: A11yRole,
    /// Accessible-name policy.
    pub name: NamePolicy,
    /// Alternative-text policy.
    pub alt: AltPolicy,
    /// Attribute holding a heading level.
    pub level: Option<AttrId>,
}

impl A11ySpec {
    /// Creates a role declaration with default policies.
    #[must_use]
    pub const fn new(role: A11yRole) -> Self {
        Self {
            role,
            name: NamePolicy::None,
            alt: if role.requires_alt() {
                AltPolicy::Required
            } else {
                AltPolicy::NotApplicable
            },
            level: None,
        }
    }

    /// Sets the name policy.
    #[must_use]
    pub const fn with_name(mut self, name: NamePolicy) -> Self {
        self.name = name;
        self
    }

    /// Sets the alternative policy.
    #[must_use]
    pub const fn with_alt(mut self, alt: AltPolicy) -> Self {
        self.alt = alt;
        self
    }

    /// Sets the heading level attribute.
    #[must_use]
    pub const fn with_level(mut self, attr: AttrId) -> Self {
        self.level = Some(attr);
        self
    }

    /// Validates internal consistency.
    #[must_use]
    pub fn error_code(self) -> Option<Box<str>> {
        match self.role {
            A11yRole::Heading { level } if !(1..=6).contains(&level) => {
                Some(Box::from("heading_level"))
            }
            A11yRole::Heading { .. } if self.level.is_none() => Some(Box::from("heading_attr")),
            role if role.requires_alt() && self.alt != AltPolicy::Required => {
                Some(Box::from("alt_required"))
            }
            role if role.requires_name() && self.name == NamePolicy::None => {
                Some(Box::from("name_required"))
            }
            _ => None,
        }
    }
}
