//! Tree construction helpers.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::model::{AttrValue, Attrs, Block, BlockContent, InlineItem, MarkSet, NodeKind};
use crate::pos::{DocPath, Pos, Range};
use crate::schema::{NodeSpec, Schema};

use super::role::{A11yRole, AltPolicy, NamePolicy};
use super::tree::{A11yNode, CellScope};

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "tree projection carries inherited context"
)]
pub(super) fn block_nodes(
    schema: &Schema,
    block: &Block,
    start: Pos,
    path: &DocPath,
    lang: Option<Box<str>>,
    dir: Option<Box<str>>,
) -> Vec<A11yNode> {
    let range = Range::new(start, Pos(start.0.saturating_add(block.size().0)));
    let Some(spec) = schema.node(block.kind()) else {
        return Vec::new();
    };
    let children = child_nodes(schema, block, start, path, lang.clone(), dir.clone());
    let Some(a11y) = spec.a11y else {
        return children;
    };
    let text = block_text(block);
    let text_field = if text.is_empty() {
        None
    } else {
        Some(Box::from(text.as_str()))
    };
    let name = name_for(spec, a11y.name, block.attrs(), &text);
    let alt = alt_for(spec, a11y.alt, block.attrs(), name.as_deref());
    let href = attr_str(schema, block.kind(), block.attrs(), "href");
    let src = attr_str(schema, block.kind(), block.attrs(), "src");
    let caption = attr_str(schema, block.kind(), block.attrs(), "caption");
    let transcript = attr_str(schema, block.kind(), block.attrs(), "transcript");
    let ordered = attr_bool(schema, block.kind(), block.attrs(), "ordered");
    let level = a11y.level.and_then(|id| match block.attrs().get(id) {
        Some(AttrValue::Int(value)) => u8::try_from(*value).ok(),
        _ => match a11y.role {
            A11yRole::Heading { level } => Some(level),
            _ => None,
        },
    });
    let scope = attr_str(schema, block.kind(), block.attrs(), "scope")
        .as_deref()
        .and_then(parse_scope);
    let colspan = attr_u16(schema, block.kind(), block.attrs(), "colspan");
    let rowspan = attr_u16(schema, block.kind(), block.attrs(), "rowspan");
    Vec::from([A11yNode {
        id: block.id(),
        role: a11y.role,
        name,
        description: attr_str(schema, block.kind(), block.attrs(), "description"),
        alt,
        lang,
        dir,
        level,
        text: text_field,
        marks: MarkSet::empty(),
        href,
        src,
        caption,
        transcript,
        ordered,
        scope,
        colspan,
        rowspan,
        range,
        children: children.into_boxed_slice(),
    }])
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "tree projection carries inherited context"
)]
fn child_nodes(
    schema: &Schema,
    block: &Block,
    start: Pos,
    path: &DocPath,
    lang: Option<Box<str>>,
    dir: Option<Box<str>>,
) -> Vec<A11yNode> {
    match block.content() {
        BlockContent::Blocks(children) => {
            let mut nodes = Vec::new();
            let mut offset = start.0.saturating_add(1);
            for (index, child) in children.iter().enumerate() {
                nodes.extend(block_nodes(
                    schema,
                    child,
                    Pos(offset),
                    &path.child(u32::try_from(index).unwrap_or(u32::MAX)),
                    lang.clone(),
                    dir.clone(),
                ));
                offset = offset.saturating_add(child.size().0);
            }
            nodes
        }
        BlockContent::Text(text) => {
            let mut nodes = Vec::new();
            let mut offset = start.0.saturating_add(1);
            for item in text.items() {
                let size = match item {
                    InlineItem::Text(run) => u32::try_from(run.text.len()).unwrap_or(u32::MAX),
                    InlineItem::Atom(_) => 1,
                };
                let range = Range::new(Pos(offset), Pos(offset.saturating_add(size)));
                match item {
                    InlineItem::Text(run) => nodes.push(A11yNode {
                        id: None,
                        role: super::marks::mark_role(schema, &run.marks),
                        name: Some(Box::from(run.text.as_str())),
                        description: None,
                        alt: None,
                        lang: lang.clone(),
                        dir: dir.clone(),
                        level: None,
                        text: Some(Box::from(run.text.as_str())),
                        marks: run.marks.clone(),
                        href: super::marks::mark_href(schema, &run.marks),
                        src: None,
                        caption: None,
                        transcript: None,
                        ordered: None,
                        scope: None,
                        colspan: None,
                        rowspan: None,
                        range,
                        children: Box::new([]),
                    }),
                    InlineItem::Atom(atom) => {
                        if let Some(atom_spec) = schema.node(atom.kind)
                            && let Some(a11y) = atom_spec.a11y
                        {
                            let name = name_for(atom_spec, a11y.name, &atom.attrs, "");
                            nodes.push(A11yNode {
                                id: None,
                                role: a11y.role,
                                name: name.clone(),
                                description: attr_str(
                                    schema,
                                    atom.kind,
                                    &atom.attrs,
                                    "description",
                                ),
                                alt: alt_for(atom_spec, a11y.alt, &atom.attrs, name.as_deref()),
                                lang: lang.clone(),
                                dir: dir.clone(),
                                level: None,
                                text: None,
                                marks: MarkSet::empty(),
                                href: attr_str(schema, atom.kind, &atom.attrs, "href"),
                                src: attr_str(schema, atom.kind, &atom.attrs, "src"),
                                caption: None,
                                transcript: None,
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
                offset = offset.saturating_add(size);
            }
            nodes
        }
        BlockContent::Atom(_) => Vec::new(),
    }
}

fn block_text(block: &Block) -> String {
    let mut text = String::new();
    if let Some(inner) = block.as_textblock() {
        inner.text_into(&mut text);
    }
    text
}

fn name_for(spec: &NodeSpec, policy: NamePolicy, attrs: &Attrs, text: &str) -> Option<Box<str>> {
    match policy {
        NamePolicy::None | NamePolicy::Required => None,
        NamePolicy::TextContent => {
            if text.is_empty() {
                None
            } else {
                Some(Box::from(text))
            }
        }
        NamePolicy::Attr(id) => match spec.attrs.iter().find(|attr| attr.id == id) {
            Some(attr) => attr_string(attrs.get(attr.id)),
            None => None,
        },
    }
}

fn alt_for(
    spec: &NodeSpec,
    policy: AltPolicy,
    attrs: &Attrs,
    name: Option<&str>,
) -> Option<Box<str>> {
    if policy == AltPolicy::NotApplicable {
        return None;
    }
    if let Some(attr) = spec.attrs.iter().find(|attr| &*attr.name == "alt")
        && let Some(value) = attr_string(attrs.get(attr.id))
    {
        return Some(value);
    }
    name.map(Box::from)
}

pub(super) fn attr_string(value: Option<&AttrValue>) -> Option<Box<str>> {
    match value {
        Some(AttrValue::Str(value)) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}

fn attr_str(schema: &Schema, kind: NodeKind, attrs: &Attrs, name: &str) -> Option<Box<str>> {
    let spec = schema.node(kind)?;
    let attr = spec.attrs.iter().find(|attr| &*attr.name == name)?;
    attr_string(attrs.get(attr.id))
}

fn attr_bool(schema: &Schema, kind: NodeKind, attrs: &Attrs, name: &str) -> Option<bool> {
    let spec = schema.node(kind)?;
    let attr = spec.attrs.iter().find(|attr| &*attr.name == name)?;
    match attrs.get(attr.id) {
        Some(AttrValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn attr_u16(schema: &Schema, kind: NodeKind, attrs: &Attrs, name: &str) -> Option<u16> {
    let spec = schema.node(kind)?;
    let attr = spec.attrs.iter().find(|attr| &*attr.name == name)?;
    match attrs.get(attr.id) {
        Some(AttrValue::Int(value)) => u16::try_from(*value).ok(),
        _ => None,
    }
}

pub(super) fn doc_attr(schema: &Schema, attrs: &Attrs, name: &str) -> Option<Box<str>> {
    let attr = schema.doc().attrs.iter().find(|attr| &*attr.name == name)?;
    attr_string(attrs.get(attr.id))
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
