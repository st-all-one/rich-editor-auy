//! Tree construction helpers.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::model::{Attrs, Block, BlockContent, InlineItem, MarkSet};
use crate::pos::{DocPath, Pos, Range};
use crate::schema::{NodeSpec, Schema};

use super::attrs::{AttrCache, integer, text};
use super::marks::marks_role_and_href;
use super::role::{A11yRole, AltPolicy, NamePolicy};
use super::text::descendant_text;
use super::tree::{A11yNode, CellScope};

/// Per-tree projection state: schema, cached attribute ids, and inherited language context.
pub(super) struct Projection<'a> {
    schema: &'a Schema,
    attrs: AttrCache,
    lang: Option<Box<str>>,
    dir: Option<Box<str>>,
}

impl<'a> Projection<'a> {
    /// Creates a projection for one document.
    pub(super) fn new(schema: &'a Schema, lang: Option<Box<str>>, dir: Option<Box<str>>) -> Self {
        Self {
            schema,
            attrs: AttrCache::new(schema),
            lang,
            dir,
        }
    }

    /// Appends the nodes projected from one block, in reading order.
    #[allow(
        clippy::too_many_lines,
        reason = "one node projection with every inherited field"
    )]
    pub(super) fn block_into(
        &self,
        block: &Block,
        start: Pos,
        path: DocPath,
        out: &mut Vec<A11yNode>,
    ) {
        let Some(spec) = self.schema.node(block.kind()) else {
            return;
        };
        let Some(a11y) = spec.a11y else {
            self.child_into(block, start, &path, out);
            return;
        };
        let cached = self.attrs.get(block.kind());
        let mut children = Vec::new();
        self.child_into(block, start, &path, &mut children);
        let attrs = block.attrs();
        let mut text_buffer = String::new();
        if let Some(inner) = block.as_textblock() {
            inner.text_into(&mut text_buffer);
        }
        let text_field = if text_buffer.is_empty() {
            None
        } else {
            Some(text_buffer.into_boxed_str())
        };
        let name_text = match a11y.name {
            NamePolicy::TextContent if text_field.is_none() => {
                let mut collected = String::new();
                descendant_text(block, &mut collected);
                Cow::Owned(collected)
            }
            _ => Cow::Borrowed(text_field.as_deref().unwrap_or("")),
        };
        let name = name_for(spec, a11y.name, attrs, &name_text);
        let level = a11y.level.and_then(|id| match attrs.get(id) {
            Some(crate::model::AttrValue::Int(value)) => u8::try_from(*value).ok(),
            _ => match a11y.role {
                A11yRole::Heading { level } => Some(level),
                _ => None,
            },
        });
        let alt = alt_for(a11y.alt, attrs, cached.alt, name.as_deref());
        out.push(A11yNode {
            path,
            id: block.id(),
            role: a11y.role,
            name,
            description: text(attrs, cached.description),
            alt,
            lang: self.lang.clone(),
            dir: self.dir.clone(),
            level,
            text: text_field,
            marks: MarkSet::empty(),
            href: text(attrs, cached.href),
            src: text(attrs, cached.src),
            caption: text(attrs, cached.caption),
            transcript: text(attrs, cached.transcript),
            code_language: text(attrs, cached.language),
            ordered: ordered_of(&spec.name),
            scope: text(attrs, cached.scope).as_deref().and_then(parse_scope),
            colspan: integer(attrs, cached.colspan),
            rowspan: integer(attrs, cached.rowspan),
            range: Range::new(start, Pos(start.0.saturating_add(block.size().0))),
            children: children.into_boxed_slice(),
        });
    }

    /// Appends the children of one block.
    fn child_into(&self, block: &Block, start: Pos, path: &DocPath, out: &mut Vec<A11yNode>) {
        match block.content() {
            BlockContent::Blocks(children) => {
                out.reserve(children.len());
                let mut offset = start.0.saturating_add(1);
                for (index, child) in children.iter().enumerate() {
                    self.block_into(
                        child,
                        Pos(offset),
                        path.child(u32::try_from(index).unwrap_or(u32::MAX)),
                        out,
                    );
                    offset = offset.saturating_add(child.size().0);
                }
            }
            BlockContent::Text(inline) => {
                out.reserve(inline.items().len());
                let mut offset = start.0.saturating_add(1);
                for (index, item) in inline.items().iter().enumerate() {
                    let item_path = path.child(u32::try_from(index).unwrap_or(u32::MAX));
                    let size = match item {
                        InlineItem::Text(run) => u32::try_from(run.text.len()).unwrap_or(u32::MAX),
                        InlineItem::Atom(_) => 1,
                    };
                    let range = Range::new(Pos(offset), Pos(offset.saturating_add(size)));
                    self.push_inline(item, item_path, range, out);
                    offset = offset.saturating_add(size);
                }
            }
            BlockContent::Atom(_) => {}
        }
    }

    /// Appends one inline leaf.
    #[allow(
        clippy::too_many_lines,
        reason = "one leaf projection with every field"
    )]
    fn push_inline(&self, item: &InlineItem, path: DocPath, range: Range, out: &mut Vec<A11yNode>) {
        match item {
            InlineItem::Text(run) => {
                let (role, href) = marks_role_and_href(&run.marks, self.schema);
                let text: Box<str> = Box::from(run.text.as_str());
                out.push(A11yNode {
                    path,
                    id: None,
                    role,
                    name: Some(text.clone()),
                    description: None,
                    alt: None,
                    lang: self.lang.clone(),
                    dir: self.dir.clone(),
                    level: None,
                    text: Some(text),
                    marks: run.marks.clone(),
                    href,
                    src: None,
                    caption: None,
                    transcript: None,
                    code_language: None,
                    ordered: None,
                    scope: None,
                    colspan: None,
                    rowspan: None,
                    range,
                    children: Box::new([]),
                });
            }
            InlineItem::Atom(atom) => {
                let Some(atom_spec) = self.schema.node(atom.kind) else {
                    return;
                };
                let Some(a11y) = atom_spec.a11y else {
                    return;
                };
                let cached = self.attrs.get(atom.kind);
                let name = name_for(atom_spec, a11y.name, &atom.attrs, "");
                out.push(A11yNode {
                    path,
                    id: None,
                    role: a11y.role,
                    name: name.clone(),
                    description: text(&atom.attrs, cached.description),
                    alt: alt_for(a11y.alt, &atom.attrs, cached.alt, name.as_deref()),
                    lang: self.lang.clone(),
                    dir: self.dir.clone(),
                    level: None,
                    text: None,
                    marks: MarkSet::empty(),
                    href: text(&atom.attrs, cached.href),
                    src: text(&atom.attrs, cached.src),
                    caption: None,
                    transcript: None,
                    code_language: None,
                    ordered: None,
                    scope: None,
                    colspan: None,
                    rowspan: None,
                    range,
                    children: Box::new([]),
                });
            }
        }
    }
}

/// Resolves the accessible name for one node.
fn name_for(
    spec: &NodeSpec,
    policy: NamePolicy,
    attrs: &Attrs,
    text_value: &str,
) -> Option<Box<str>> {
    match policy {
        NamePolicy::None | NamePolicy::Required => None,
        NamePolicy::TextContent => {
            if text_value.is_empty() {
                None
            } else {
                Some(Box::from(text_value))
            }
        }
        NamePolicy::Attr(id) => match spec.attrs.iter().find(|attr| attr.id == id) {
            Some(attr) => text(attrs, Some(attr.id)),
            None => None,
        },
    }
}

/// Resolves the textual alternative for one node.
fn alt_for(
    policy: AltPolicy,
    attrs: &Attrs,
    alt_id: Option<crate::model::AttrId>,
    name: Option<&str>,
) -> Option<Box<str>> {
    if policy == AltPolicy::NotApplicable {
        return None;
    }
    if let Some(value) = text(attrs, alt_id) {
        return Some(value);
    }
    name.map(Box::from)
}

/// Returns the list ordering flag for a node name.
fn ordered_of(name: &str) -> Option<bool> {
    match name {
        "ordered_list" => Some(true),
        "bullet_list" => Some(false),
        _ => None,
    }
}

/// Returns a document attribute value by name.
pub(super) fn doc_attr(schema: &Schema, attrs: &Attrs, name: &str) -> Option<Box<str>> {
    let attr = schema.doc().attrs.iter().find(|attr| &*attr.name == name)?;
    text(attrs, Some(attr.id))
}

fn parse_scope(value: &str) -> Option<CellScope> {
    match value {
        "col" => Some(CellScope::Col),
        "row" => Some(CellScope::Row),
        "colgroup" => Some(CellScope::ColGroup),
        "rowgroup" => Some(CellScope::RowGroup),
        _ => None,
    }
}
