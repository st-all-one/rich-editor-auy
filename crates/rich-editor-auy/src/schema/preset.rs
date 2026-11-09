//! Canonical schema presets used by tests and examples.

#[cfg(feature = "migration")]
use crate::a11y::{A11yRole, A11ySpec, AltPolicy, NamePolicy};
#[cfg(feature = "migration")]
use crate::error::SchemaError;
#[cfg(feature = "migration")]
use crate::model::{AttrId, AttrValue};

use super::Schema;
#[cfg(feature = "migration")]
use super::{AttrSpec, AttrType, DocSpec, MarkSpec, NodeSpec};

impl Schema {
    /// Builds the basic document schema.
    ///
    /// The preset contains document, paragraph, heading, list, image, and the common text marks.
    ///
    /// # Errors
    ///
    /// Returns a schema error if the preset itself is inconsistent.
    #[allow(clippy::too_many_lines, reason = "one readable canonical preset")]
    #[cfg(feature = "migration")]
    pub fn basic() -> Result<Self, SchemaError> {
        let heading_level = AttrSpec::new("level", AttrType::Int { min: 1, max: 6 })
            .required()
            .with_default(AttrValue::Int(1));
        let image_alt = AttrSpec::new("alt", AttrType::Str).required();
        let image_name = NamePolicy::Attr(AttrId(1));
        let doc = DocSpec::new()
            .content("block+")
            .attr(AttrSpec::new("lang", AttrType::Str))
            .attr(AttrSpec::new("dir", AttrType::Str));
        Self::builder()
            .doc(doc)
            .node(
                NodeSpec::new("paragraph")
                    .group("block")
                    .content("text*")
                    .marks_all()
                    .a11y(A11ySpec::new(A11yRole::Paragraph).with_name(NamePolicy::TextContent)),
            )
            .node(
                NodeSpec::new("heading")
                    .group("block")
                    .content("text*")
                    .marks_all()
                    .attr(heading_level)
                    .a11y(
                        A11ySpec::new(A11yRole::Heading { level: 1 })
                            .with_name(NamePolicy::TextContent)
                            .with_level(AttrId(1)),
                    ),
            )
            .node(
                NodeSpec::new("bullet_list")
                    .group("block")
                    .content("list_item+")
                    .a11y(A11ySpec::new(A11yRole::List)),
            )
            .node(
                NodeSpec::new("list_item")
                    .group("block")
                    .content("block+")
                    .a11y(A11ySpec::new(A11yRole::ListItem)),
            )
            .node(
                NodeSpec::new("image").inline().atom().attr(image_alt).a11y(
                    A11ySpec::new(A11yRole::Image)
                        .with_name(image_name)
                        .with_alt(AltPolicy::Required),
                ),
            )
            .node(
                NodeSpec::new("text")
                    .inline()
                    .a11y(A11ySpec::new(A11yRole::Text).with_name(NamePolicy::TextContent)),
            )
            .mark(MarkSpec::new("strong"))
            .mark(MarkSpec::new("em"))
            .mark(MarkSpec::new("code"))
            .mark(
                MarkSpec::new("link")
                    .attr(AttrSpec::new("href", AttrType::Str).required())
                    .a11y(A11ySpec::new(A11yRole::Link).with_name(NamePolicy::TextContent)),
            )
            .mark(MarkSpec::new("ref").attr(AttrSpec::new("target", AttrType::Ref).required()))
            .build()
    }
}
