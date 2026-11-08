//! Compile-time front-end for `rich-editor-auy` schemas.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::drain_collect,
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::iter_with_drain,
    clippy::needless_pass_by_value,
    clippy::unnecessary_map_or,
    clippy::unnecessary_option_map_or_else,
    clippy::unnecessary_wraps,
    reason = "token parsing is a bounded build-time front-end that reports errors as compile_error"
)]
//!
//! The runtime builder remains the source of truth. This crate translates the declarative block
//! into builder calls, so both paths compile through the same validation.
//!
//! Supported form:
//!
//! ```text
//! schema! {
//!     doc { content: "block+" }
//!     paragraph { group: "block", content: "text*", marks: "_" }
//!     heading { group: "block", content: "text*", attrs: { level: int(1..=6) = 1 } }
//!     text { inline: true }
//!     strong { mark }
//!     link { mark, attrs: { href: str } }
//! }
//! ```

mod parse;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Literal, TokenStream as TokenStream2, TokenTree};

use crate::parse::{
    Cursor, Entry, bool_literal, entries, parse_attr_type, parse_default, single_group,
    string_literal,
};
use quote::quote;

/// Declares a schema using the canonical builder.
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    match expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(message) => {
            let message = Literal::string(&message);
            quote! { ::core::compile_error!(#message) }.into()
        }
    }
}

fn expand(input: TokenStream2) -> Result<TokenStream2, String> {
    let mut cursor = Cursor::new(input);
    let mut doc = None;
    let mut nodes = Vec::new();
    let mut marks = Vec::new();
    while !cursor.is_empty() {
        let name = cursor.expect_ident("declaration")?;
        let body = cursor.expect_group(Delimiter::Brace, "declaration body")?;
        if name == "doc" {
            doc = Some(expand_doc(body)?);
        } else {
            let entries = entries(body)?;
            if entries.iter().any(|entry| entry.name == "mark") {
                marks.push(expand_mark(name, entries)?);
            } else {
                nodes.push(expand_node(name, entries)?);
            }
        }
    }
    let doc = doc.map_or_else(
        || quote! { ::rich_editor_auy::schema::DocSpec::new() },
        |tokens| tokens,
    );
    let mut calls = quote! { ::rich_editor_auy::schema::Schema::builder().doc(#doc) };
    for node in nodes {
        calls = quote! { #calls.node(#node) };
    }
    for mark in marks {
        calls = quote! { #calls.mark(#mark) };
    }
    Ok(quote! { #calls.build() })
}

fn expand_doc(body: Group) -> Result<TokenStream2, String> {
    let mut spec = quote! { ::rich_editor_auy::schema::DocSpec::new() };
    for entry in entries(body)? {
        match entry.name.as_str() {
            "content" => {
                let value = string_literal(&entry.value)?;
                spec = quote! { #spec.content(#value) };
            }
            "attrs" => {
                for attr in attr_entries(entry.value)? {
                    spec = quote! { #spec.attr(#attr) };
                }
            }
            other => return Err(format!("unsupported doc field `{other}`")),
        }
    }
    Ok(spec)
}

fn expand_node(name: String, entries: Vec<Entry>) -> Result<TokenStream2, String> {
    let name = Literal::string(&name);
    let mut spec = quote! { ::rich_editor_auy::schema::NodeSpec::new(#name) };
    for entry in entries {
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
                let value = string_literal(&entry.value)?;
                if value.to_string() == "\"_\""
                    || value.to_string() == "\"all\""
                    || value.to_string() == "\"*\""
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
                for attr in attr_entries(entry.value)? {
                    spec = quote! { #spec.attr(#attr) };
                }
            }
            other => return Err(format!("unsupported node field `{other}`")),
        }
    }
    Ok(spec)
}

fn expand_mark(name: String, entries: Vec<Entry>) -> Result<TokenStream2, String> {
    let name = Literal::string(&name);
    let mut spec = quote! { ::rich_editor_auy::schema::MarkSpec::new(#name) };
    for entry in entries {
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
                for attr in attr_entries(entry.value)? {
                    spec = quote! { #spec.attr(#attr) };
                }
            }
            other => return Err(format!("unsupported mark field `{other}`")),
        }
    }
    Ok(spec)
}

fn attr_entries(value: TokenStream2) -> Result<Vec<TokenStream2>, String> {
    let group = single_group(value, Delimiter::Brace, "attrs")?;
    let mut attrs = Vec::new();
    for entry in entries(group)? {
        let mut tokens = entry.value.into_iter().peekable();
        let kind = parse_attr_type(&mut tokens)?;
        let default = if tokens.peek().is_some() {
            let Some(TokenTree::Punct(punct)) = tokens.next() else {
                return Err("expected `=` before attribute default".into());
            };
            if punct.as_char() != '=' {
                return Err("expected `=` before attribute default".into());
            }
            Some(parse_default(tokens.collect())?)
        } else {
            None
        };
        let name = Literal::string(&entry.name);
        let mut attr = quote! { ::rich_editor_auy::schema::AttrSpec::new(#name, #kind) };
        if let Some(default) = default {
            attr = quote! { #attr.with_default(#default) };
        }
        attrs.push(attr);
    }
    Ok(attrs)
}
