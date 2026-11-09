//! Accessibility declaration grammar.
//!
//! ```text
//! a11y: { role: paragraph, name: text }
//! a11y: { role: heading(2), name: text, level: "level" }
//! a11y: { role: image, name: attr("alt"), alt: required }
//! ```

#![allow(
    clippy::redundant_pub_crate,
    reason = "the module is private; its items are crate-visible on purpose"
)]
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::attrs::AttrDecl;
use crate::parse::{Entry, entries, int_literal, string_value};

/// Attribute identifiers visible to one declaration.
pub(crate) struct AttrScope<'a> {
    attrs: &'a [AttrDecl],
    /// Offset added to the declaration index, because nodes reserve identifier zero for `id`.
    offset: u16,
}

impl<'a> AttrScope<'a> {
    /// Creates a scope for a node, mark, or document declaration.
    pub(crate) const fn new(attrs: &'a [AttrDecl], offset: u16) -> Self {
        Self { attrs, offset }
    }

    fn resolve(&self, name: &str) -> Result<u16, String> {
        for (index, attr) in self.attrs.iter().enumerate() {
            if attr.name == name {
                let index = u16::try_from(index).map_err(|_| "too many attributes")?;
                return Ok(index.saturating_add(self.offset));
            }
        }
        Err(format!("unknown attribute `{name}`"))
    }
}

/// Parses an `a11y: { ... }` body into an `A11ySpec` builder expression.
pub(crate) fn a11y_tokens(group: Group, scope: &AttrScope<'_>) -> Result<TokenStream2, String> {
    let entries = entries(group)?;
    let mut role = None;
    let mut name = None;
    let mut alt = None;
    let mut level = None;
    for entry in entries {
        match entry.name.as_str() {
            "role" => role = Some(role_tokens(&entry)?),
            "name" => name = Some(name_tokens(&entry, scope)?),
            "alt" => alt = Some(alt_tokens(&entry)?),
            "level" => level = Some(level_tokens(&entry, scope)?),
            other => return Err(format!("unsupported a11y field `{other}`")),
        }
    }
    let Some((role, is_heading)) = role else {
        return Err("a11y requires `role`".into());
    };
    if is_heading && level.is_none() {
        return Err("heading requires level".into());
    }
    let mut spec = quote! { ::rich_editor_auy::a11y::A11ySpec::new(#role) };
    if let Some(name) = name {
        spec = quote! { #spec.with_name(#name) };
    }
    if let Some(alt) = alt {
        spec = quote! { #spec.with_alt(#alt) };
    }
    if let Some(level) = level {
        spec = quote! { #spec.with_level(#level) };
    }
    Ok(spec)
}

/// Parses the role field, returning the role expression and whether it is a heading.
fn role_tokens(entry: &Entry) -> Result<(TokenStream2, bool), String> {
    let mut tokens = entry.value.clone().into_iter();
    let Some(TokenTree::Ident(role)) = tokens.next() else {
        return Err("expected accessibility role".into());
    };
    let role = role.to_string();
    let argument = match tokens.next() {
        None => None,
        Some(TokenTree::Group(group)) => Some(group),
        Some(_) => return Err("expected accessibility role".into()),
    };
    if tokens.next().is_some() {
        return Err("unexpected tokens after accessibility role".into());
    }
    let plain = |tokens: TokenStream2| Ok((tokens, false));
    match (role.as_str(), argument) {
        ("document", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Document }),
        ("text", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Text }),
        ("paragraph", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Paragraph }),
        ("list", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::List }),
        ("list_item", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::ListItem }),
        ("quote", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Quote }),
        ("code", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Code }),
        ("code_block", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::CodeBlock }),
        ("table", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Table }),
        ("table_caption", None) => {
            plain(quote! { ::rich_editor_auy::a11y::A11yRole::TableCaption })
        }
        ("table_row", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::TableRow }),
        ("table_cell", None) => plain(quote! {
            ::rich_editor_auy::a11y::A11yRole::TableCell { header: false }
        }),
        ("table_header", None) => plain(quote! {
            ::rich_editor_auy::a11y::A11yRole::TableCell { header: true }
        }),
        ("link", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Link }),
        ("image", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Image }),
        ("figure", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Figure }),
        ("caption", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Caption }),
        ("video", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Video }),
        ("audio", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Audio }),
        ("attachment", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Attachment }),
        ("separator", None) => plain(quote! { ::rich_editor_auy::a11y::A11yRole::Separator }),
        ("heading", Some(group)) => {
            let level = int_literal(group.stream())?;
            Ok((
                quote! { ::rich_editor_auy::a11y::A11yRole::Heading { level: #level } },
                true,
            ))
        }
        _ => Err(format!("unsupported accessibility role `{role}`")),
    }
}

/// Parses the name policy field.
fn name_tokens(entry: &Entry, scope: &AttrScope<'_>) -> Result<TokenStream2, String> {
    let mut tokens = entry.value.clone().into_iter();
    let Some(TokenTree::Ident(policy)) = tokens.next() else {
        return Err("expected name policy".into());
    };
    match policy.to_string().as_str() {
        "none" => Ok(quote! { ::rich_editor_auy::a11y::NamePolicy::None }),
        "text" => Ok(quote! { ::rich_editor_auy::a11y::NamePolicy::TextContent }),
        "required" => Ok(quote! { ::rich_editor_auy::a11y::NamePolicy::Required }),
        "attr" => {
            let Some(TokenTree::Group(group)) = tokens.next() else {
                return Err("`attr` requires a name".into());
            };
            let id = scope.resolve(&string_value(&group.stream())?)?;
            Ok(
                quote! { ::rich_editor_auy::a11y::NamePolicy::Attr(::rich_editor_auy::model::AttrId(#id)) },
            )
        }
        other => Err(format!("unsupported name policy `{other}`")),
    }
}

/// Parses the alternative-text policy field.
fn alt_tokens(entry: &Entry) -> Result<TokenStream2, String> {
    match entry.value.to_string().as_str() {
        "required" => Ok(quote! { ::rich_editor_auy::a11y::AltPolicy::Required }),
        "optional" => Ok(quote! { ::rich_editor_auy::a11y::AltPolicy::Optional }),
        "not_applicable" => Ok(quote! { ::rich_editor_auy::a11y::AltPolicy::NotApplicable }),
        other => Err(format!("unsupported alt policy `{other}`")),
    }
}

/// Parses the heading-level attribute field.
fn level_tokens(entry: &Entry, scope: &AttrScope<'_>) -> Result<TokenStream2, String> {
    let id = scope.resolve(&string_value(&entry.value)?)?;
    Ok(quote! { ::rich_editor_auy::model::AttrId(#id) })
}
