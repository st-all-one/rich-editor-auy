//! Compile-time front-end for `rich-editor-auy` schemas.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::drain_collect,
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::iter_with_drain,
    clippy::needless_pass_by_value,
    clippy::unnecessary_map_or,
    clippy::unnecessary_wraps,
    reason = "token parsing is a bounded build-time front-end that reports errors as compile_error"
)]
//!
//! The runtime builder remains the source of truth: this crate translates the declarative
//! block into builder calls, so both paths compile through the same validation.
//!
//! # Expression mode
//!
//! ```text
//! let schema = schema! {
//!     doc { content: "block+", attrs: { lang: str } }
//!     paragraph { group: "block", content: "text*", marks: "_",
//!                 a11y: { role: paragraph, name: text } }
//!     heading { group: "block", content: "text*", marks: "_",
//!               attrs: { level: int(1..=6) = 1 },
//!               a11y: { role: heading(2), name: text, level: "level" } }
//!     text { inline: true, a11y: { role: text, name: text } }
//!     strong { mark }
//!     link { mark, attrs: { href: str required },
//!            a11y: { role: link, name: text } }
//! };
//! ```
//!
//! # Module mode
//!
//! A leading `pub mod NAME;` (or `mod NAME;`) turns the macro into an item that declares typed
//! accessors plus `build()`:
//!
//! ```text
//! schema! {
//!     pub mod kinds;
//!     doc { content: "block+" }
//!     paragraph { group: "block", content: "text*", marks: "_" }
//! }
//!
//! kinds::NODE_PARAGRAPH;  // NodeKind(0)
//! kinds::build();         // Result<Schema, SchemaError>
//! ```
//!
//! Accessor names are `NODE_<NODE>`, `NODE_<NODE>_<ATTR>`, `DOC_<ATTR>`, `MARK_<MARK>` and
//! `MARK_<MARK>_<ATTR>`, uppercased with underscores between words.
//!
//! # Attribute grammar
//!
//! ```text
//! attrs: { name: str, flag: bool, target: ref, payload: any,
//!          level: int(1..=6) = 1, role: enum("a", "b"),
//!          tags: list(str), meta: map(str), href: str required }
//! ```
//!
//! Defaults are accepted for `str`, `bool`, `int`, and `enum` literals only. `id` is reserved
//! for the block anchor and cannot be declared.
//!
//! # Accessibility grammar
//!
//! ```text
//! a11y: {
//!     role: document | text | paragraph | list | list_item | quote | code | code_block |
//!           table | table_caption | table_row | table_cell | table_header | link | image |
//!           figure | caption | video | audio | attachment | separator | heading(1..=6),
//!     name: none | text | required | attr("<attribute>"),
//!     alt: required | optional | not_applicable,
//!     level: "<attribute>"
//! }
//! ```
//!
//! A heading requires `level`. Roles and policies are validated by the builder through
//! `A11ySpec::error_code`, so the macro never duplicates that check.

mod a11y;
mod accessors;
mod attrs;
mod expand;
mod parse;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Literal, TokenStream as TokenStream2};
use quote::quote;

use crate::accessors::expand_module;
use crate::expand::{DocDecl, MarkDecl, NodeDecl, expand_doc, expand_mark, expand_node};
use crate::parse::{Cursor, entries};

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
    let module = cursor.module();
    let mut doc = None;
    let mut nodes = Vec::new();
    let mut marks = Vec::new();
    while !cursor.is_empty() {
        let name = cursor.expect_ident("declaration")?;
        let body = cursor.expect_group(Delimiter::Brace, "declaration body")?;
        if name == "doc" {
            if doc.is_some() {
                return Err("duplicate doc declaration".into());
            }
            doc = Some(expand_doc(body)?);
            continue;
        }
        let entries = entries(body)?;
        if entries.iter().any(|entry| entry.name == "mark") {
            marks.push(expand_mark(name, entries)?);
        } else {
            nodes.push(expand_node(name, entries)?);
        }
    }
    let doc = doc.unwrap_or(DocDecl {
        tokens: quote! { ::rich_editor_auy::schema::DocSpec::new() },
        attrs: Vec::new(),
    });
    let builder = builder_expr(&doc, &nodes, &marks);
    Ok(match module {
        Some(module) => expand_module(&module, &doc, &nodes, &marks, &builder),
        None => builder,
    })
}

/// Returns the builder expression shared by both modes.
fn builder_expr(doc: &DocDecl, nodes: &[NodeDecl], marks: &[MarkDecl]) -> TokenStream2 {
    let doc = &doc.tokens;
    let mut calls = quote! { ::rich_editor_auy::schema::Schema::builder().doc(#doc) };
    for node in nodes {
        let tokens = &node.tokens;
        calls = quote! { #calls.node(#tokens) };
    }
    for mark in marks {
        let tokens = &mark.tokens;
        calls = quote! { #calls.mark(#tokens) };
    }
    quote! { #calls.build() }
}
