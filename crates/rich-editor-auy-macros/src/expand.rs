//! Declaration expansion into canonical builder calls.

#![allow(
    clippy::redundant_pub_crate,
    reason = "the module is private; its items are crate-visible on purpose"
)]
use proc_macro2::{Delimiter, Literal, TokenStream as TokenStream2};
use quote::quote;

use crate::a11y::{AttrScope, a11y_tokens};
use crate::attrs::{AttrDecl, attr_decls};
use crate::parse::{Entry, bool_literal, entries as entries_of, string_literal};

/// One expanded node declaration.
pub(crate) struct NodeDecl {
    /// Declared name.
    pub(crate) name: String,
    /// Builder expression.
    pub(crate) tokens: TokenStream2,
    /// Declared attributes, in declaration order.
    pub(crate) attrs: Vec<AttrDecl>,
}

/// One expanded mark declaration.
pub(crate) struct MarkDecl {
    /// Declared name.
    pub(crate) name: String,
    /// Builder expression.
    pub(crate) tokens: TokenStream2,
    /// Declared attributes, in declaration order.
    pub(crate) attrs: Vec<AttrDecl>,
}

/// One expanded document declaration.
pub(crate) struct DocDecl {
    /// Builder expression.
    pub(crate) tokens: TokenStream2,
    /// Declared attributes, in declaration order.
    pub(crate) attrs: Vec<AttrDecl>,
}

/// Expands the `doc { ... }` declaration.
pub(crate) fn expand_doc(body: proc_macro2::Group) -> Result<DocDecl, String> {
    let entries = entries_of(body)?;
    let attrs = declared_attrs(&entries)?;
    let scope = AttrScope::new(&attrs, 0);
    let mut spec = quote! { ::rich_editor_auy::schema::DocSpec::new() };
    for entry in &entries {
        match entry.name.as_str() {
            "content" => {
                let value = string_literal(&entry.value)?;
                spec = quote! { #spec.content(#value) };
            }
            "attrs" => {
                let group = entry.group(Delimiter::Brace, "attrs")?;
                if group.stream().is_empty() {
                    continue;
                }
                for attr in attr_decls(group)? {
                    let tokens = attr.tokens;
                    spec = quote! { #spec.attr(#tokens) };
                }
            }
            "a11y" => {
                let group = entry.group(Delimiter::Brace, "a11y")?;
                let a11y = a11y_tokens(group, &scope)?;
                spec = quote! { #spec.a11y(#a11y) };
            }
            other => return Err(format!("unsupported doc field `{other}`")),
        }
    }
    Ok(DocDecl {
        tokens: spec,
        attrs,
    })
}

/// Expands one node declaration.
#[allow(
    clippy::too_many_lines,
    reason = "one field arm per node declaration field"
)]
pub(crate) fn expand_node(name: String, entries: Vec<Entry>) -> Result<NodeDecl, String> {
    let attrs = declared_attrs(&entries)?;
    // Nodes reserve attribute identifier zero for the block anchor.
    let scope = AttrScope::new(&attrs, 1);
    let literal = Literal::string(&name);
    let mut spec = quote! { ::rich_editor_auy::schema::NodeSpec::new(#literal) };
    for entry in &entries {
        match entry.name.as_str() {
            "content" => {
                let value = string_literal(&entry.value)?;
                spec = quote! { #spec.content(#value) };
            }
            "group" => {
                let value = string_literal(&entry.value)?;
                spec = quote! { #spec.group(#value) };
            }
            "marks" => {
                let value = entry.value.to_string();
                if value == "\"_\""
                    || value == "\"all\""
                    || value == "\"*\""
                    || value == "_"
                    || value == "all"
                    || value == "*"
                {
                    spec = quote! { #spec.marks_all() };
                } else {
                    return Err("marks supports only `_`, `all` or `*`".into());
                }
            }
            "inline" => {
                if bool_literal(&entry.value)? {
                    spec = quote! { #spec.inline() };
                }
            }
            "atom" => {
                if bool_literal(&entry.value)? {
                    spec = quote! { #spec.atom() };
                }
            }
            "defining" => {
                if bool_literal(&entry.value)? {
                    spec = quote! { #spec.defining() };
                }
            }
            "isolating" => {
                if bool_literal(&entry.value)? {
                    spec = quote! { #spec.isolating() };
                }
            }
            "attrs" => {
                let group = entry.group(Delimiter::Brace, "attrs")?;
                if group.stream().is_empty() {
                    continue;
                }
                for attr in attr_decls(group)? {
                    let tokens = attr.tokens;
                    spec = quote! { #spec.attr(#tokens) };
                }
            }
            "a11y" => {
                let group = entry.group(Delimiter::Brace, "a11y")?;
                let a11y = a11y_tokens(group, &scope)?;
                spec = quote! { #spec.a11y(#a11y) };
            }
            other => return Err(format!("unsupported node field `{other}`")),
        }
    }
    Ok(NodeDecl {
        name,
        tokens: spec,
        attrs,
    })
}

/// Expands one mark declaration.
pub(crate) fn expand_mark(name: String, entries: Vec<Entry>) -> Result<MarkDecl, String> {
    let attrs = declared_attrs(&entries)?;
    let scope = AttrScope::new(&attrs, 0);
    let literal = Literal::string(&name);
    let mut spec = quote! { ::rich_editor_auy::schema::MarkSpec::new(#literal) };
    for entry in &entries {
        match entry.name.as_str() {
            "mark" => {}
            "inclusive" => {
                let value = bool_literal(&entry.value)?;
                spec = quote! { #spec.inclusive(#value) };
            }
            "excludes" => {
                let value = string_literal(&entry.value)?;
                spec = quote! { #spec.excludes(#value) };
            }
            "attrs" => {
                let group = entry.group(Delimiter::Brace, "attrs")?;
                if group.stream().is_empty() {
                    continue;
                }
                for attr in attr_decls(group)? {
                    let tokens = attr.tokens;
                    spec = quote! { #spec.attr(#tokens) };
                }
            }
            "a11y" => {
                let group = entry.group(Delimiter::Brace, "a11y")?;
                let a11y = a11y_tokens(group, &scope)?;
                spec = quote! { #spec.a11y(#a11y) };
            }
            other => return Err(format!("unsupported mark field `{other}`")),
        }
    }
    Ok(MarkDecl {
        name,
        tokens: spec,
        attrs,
    })
}

/// Collects the declared attribute names in declaration order.
fn declared_attrs(entries: &[Entry]) -> Result<Vec<AttrDecl>, String> {
    for entry in entries {
        if entry.name == "attrs" {
            let group = entry.group(Delimiter::Brace, "attrs")?;
            if group.stream().is_empty() {
                return Ok(Vec::new());
            }
            return attr_decls(group);
        }
    }
    Ok(Vec::new())
}
