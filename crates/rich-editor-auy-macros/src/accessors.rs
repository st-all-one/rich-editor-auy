//! Typed accessors for module mode.

#![allow(
    clippy::redundant_pub_crate,
    reason = "the module is private; its items are crate-visible on purpose"
)]
use proc_macro2::{Ident, Literal, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

use crate::attrs::AttrDecl;
use crate::expand::{DocDecl, MarkDecl, NodeDecl};
use crate::parse::{ModuleDecl, Visibility};

/// Expands `pub(crate) mod name;` plus the declarations into a module with constants and `build()`.
pub(crate) fn expand_module(
    module: &ModuleDecl,
    doc: &DocDecl,
    nodes: &[NodeDecl],
    marks: &[MarkDecl],
    builder: &TokenStream2,
) -> TokenStream2 {
    let name = ident_tokens(&module.name);
    let visibility = match module.vis {
        Visibility::Public => quote! { pub(crate) },
        Visibility::Private => TokenStream2::new(),
    };
    let mut constants = TokenStream2::new();
    for (index, node) in nodes.iter().enumerate() {
        constants.extend(node_constants(&node.name, index, &node.attrs));
    }
    for (index, attr) in doc.attrs.iter().enumerate() {
        let name = ident_tokens(&format!("DOC_{}", screaming(&attr.name)));
        let label = format!("Attribute identifier for `doc.{}`.", attr.name);
        constants.extend(attr_constant(&name, index, &label));
    }
    for (index, mark) in marks.iter().enumerate() {
        let name = ident_tokens(&format!("MARK_{}", screaming(&mark.name)));
        let label = format!("Mark identifier for `{}`.", mark.name);
        constants.extend(mark_constant(&name, index, &label));
        for (attr_index, attr) in mark.attrs.iter().enumerate() {
            let name = ident_tokens(&format!(
                "MARK_{}_{}",
                screaming(&mark.name),
                screaming(&attr.name)
            ));
            let label = format!("Attribute identifier for `{}.{}`.", mark.name, attr.name);
            constants.extend(attr_constant(&name, attr_index, &label));
        }
    }
    let doc_label = Literal::string("Declared schema with typed accessors.");
    quote! {
        #[doc = #doc_label]
        #visibility mod #name {
            #constants

            /// Compiles the declared schema.
            ///
            /// # Errors
            ///
            /// Returns the same structured error as the runtime builder.
            pub(crate) fn build() -> ::core::result::Result<
                ::rich_editor_auy::schema::Schema,
                ::rich_editor_auy::error::SchemaError,
            > {
                #builder
            }
        }
    }
}

fn node_constants(name: &str, index: usize, attrs: &[AttrDecl]) -> TokenStream2 {
    let node = ident_tokens(&format!("NODE_{}", screaming(name)));
    let label = format!("Node kind for `{name}`.");
    let mut tokens = node_constant(&node, index, &label);
    for (attr_index, attr) in attrs.iter().enumerate() {
        let name = ident_tokens(&format!(
            "{}_{}",
            screaming_node(name),
            screaming(&attr.name)
        ));
        let label = format!("Attribute identifier for `{name}.{}`.", attr.name);
        // Nodes reserve identifier zero for the block anchor.
        tokens.extend(attr_constant(&name, attr_index.saturating_add(1), &label));
    }
    tokens
}

fn screaming_node(name: &str) -> String {
    format!("NODE_{}", screaming(name))
}

fn node_constant(name: &TokenStream2, index: usize, label: &str) -> TokenStream2 {
    let value = u16::try_from(index).unwrap_or(u16::MAX);
    let doc = Literal::string(label);
    quote! {
        #[doc = #doc]
        #[allow(dead_code, reason = "generated accessors may be unused")]
        pub(crate) const #name: ::rich_editor_auy::model::NodeKind =
            ::rich_editor_auy::model::NodeKind(#value);
    }
}

fn mark_constant(name: &TokenStream2, index: usize, label: &str) -> TokenStream2 {
    let value = u16::try_from(index).unwrap_or(u16::MAX);
    let doc = Literal::string(label);
    quote! {
        #[doc = #doc]
        #[allow(dead_code, reason = "generated accessors may be unused")]
        pub(crate) const #name: ::rich_editor_auy::model::MarkId =
            ::rich_editor_auy::model::MarkId(#value);
    }
}

fn attr_constant(name: &TokenStream2, index: usize, label: &str) -> TokenStream2 {
    let value = u16::try_from(index).unwrap_or(u16::MAX);
    let doc = Literal::string(label);
    quote! {
        #[doc = #doc]
        #[allow(dead_code, reason = "generated accessors may be unused")]
        pub(crate) const #name: ::rich_editor_auy::model::AttrId =
            ::rich_editor_auy::model::AttrId(#value);
    }
}

fn ident_tokens(name: &str) -> TokenStream2 {
    Ident::new(name, proc_macro2::Span::call_site()).into_token_stream()
}

/// Converts a declaration name to `SCREAMING_SNAKE_CASE`.
fn screaming(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut previous_lower = false;
    for character in name.chars() {
        if character == '-' || character == ' ' {
            out.push('_');
            previous_lower = false;
            continue;
        }
        if character.is_ascii_uppercase() && previous_lower {
            out.push('_');
        }
        out.extend(character.to_uppercase());
        previous_lower = character.is_ascii_lowercase() || character.is_ascii_digit();
    }
    out
}
