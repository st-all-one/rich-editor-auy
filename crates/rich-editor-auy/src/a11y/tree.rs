//! Deterministic projection from a document and schema into a semantic tree.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::model::{Doc, MarkSet, NodeId};
use crate::pos::{DocPath, Pos, Range};
use crate::schema::Schema;

use super::role::A11yRole;

/// Cell scope in a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellScope {
    /// Column.
    Col,
    /// Row.
    Row,
    /// Column group.
    ColGroup,
    /// Row group.
    RowGroup,
}

/// One semantic node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A11yNode {
    /// Stable document path.
    pub path: DocPath,
    /// Stable node anchor.
    pub id: Option<NodeId>,
    /// Semantic role.
    pub role: A11yRole,
    /// Accessible name.
    pub name: Option<Box<str>>,
    /// Accessible description.
    pub description: Option<Box<str>>,
    /// Textual alternative.
    pub alt: Option<Box<str>>,
    /// Inherited language.
    pub lang: Option<Box<str>>,
    /// Inherited direction.
    pub dir: Option<Box<str>>,
    /// Heading level.
    pub level: Option<u8>,
    /// Leaf text.
    pub text: Option<Box<str>>,
    /// Marks on a text leaf.
    pub marks: MarkSet,
    /// Link target.
    pub href: Option<Box<str>>,
    /// Media or attachment source.
    pub src: Option<Box<str>>,
    /// Caption.
    pub caption: Option<Box<str>>,
    /// Media transcript.
    pub transcript: Option<Box<str>>,
    /// Declared code language for code and raw-code blocks.
    pub code_language: Option<Box<str>>,
    /// Whether a list is ordered.
    pub ordered: Option<bool>,
    /// Table cell scope.
    pub scope: Option<CellScope>,
    /// Table cell column span.
    pub colspan: Option<u16>,
    /// Table cell row span.
    pub rowspan: Option<u16>,
    /// Logical range.
    pub range: Range,
    /// Child nodes in reading order.
    pub children: Box<[Self]>,
}

impl A11yNode {
    /// Returns whether the node has no children.
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// Semantic projection of a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A11yTree {
    /// Document root.
    pub root: A11yNode,
    /// Document language.
    pub lang: Option<Box<str>>,
    /// Document direction.
    pub dir: Option<Box<str>>,
}

impl Doc {
    /// Projects the document into a semantic accessibility tree.
    #[must_use]
    pub fn a11y_tree(&self, schema: &Schema) -> A11yTree {
        let lang = super::build::doc_attr(schema, self.attrs(), "lang");
        let dir = super::build::doc_attr(schema, self.attrs(), "dir");
        let root_path = DocPath::root();
        let projection = super::build::Projection::new(schema, lang.clone(), dir.clone());
        let mut children = Vec::with_capacity(self.blocks().len());
        let mut offset = 0_u32;
        for (index, block) in self.blocks().iter().enumerate() {
            projection.block_into(
                block,
                Pos(offset),
                root_path.child(u32::try_from(index).unwrap_or(u32::MAX)),
                &mut children,
            );
            offset = offset.saturating_add(block.size().0);
        }
        let root = A11yNode {
            path: root_path,
            id: None,
            role: A11yRole::Document,
            name: None,
            description: None,
            alt: None,
            lang: lang.clone(),
            dir: dir.clone(),
            level: None,
            text: None,
            marks: MarkSet::empty(),
            href: None,
            src: None,
            caption: None,
            transcript: None,
            code_language: None,
            ordered: None,
            scope: None,
            colspan: None,
            rowspan: None,
            range: Range::new(Pos(0), Pos(offset)),
            children: children.into_boxed_slice(),
        };
        A11yTree { root, lang, dir }
    }
}
