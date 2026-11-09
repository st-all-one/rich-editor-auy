//! Attribute declaration grammar.
//!
//! ```text
//! attrs: { name: str, flag: bool, level: int(1..=6) = 1, role: enum("a","b"),
//!          tags: list(str), meta: map(str), link: str required }
//! ```

#![allow(
    clippy::redundant_pub_crate,
    reason = "the module is private; its items are crate-visible on purpose"
)]
use std::iter::Peekable;

use proc_macro2::token_stream::IntoIter as TokenIter;
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::parse::{Entry, entries, int_literal, split_top_level, string_literal};

/// One parsed attribute declaration; the vector index is the declaration index.
pub(crate) struct AttrDecl {
    /// Attribute name.
    pub(crate) name: String,
    /// Builder expression producing the `AttrSpec`.
    pub(crate) tokens: TokenStream2,
}

/// Parses the `attrs: { ... }` body of one declaration.
pub(crate) fn attr_decls(group: Group) -> Result<Vec<AttrDecl>, String> {
    let mut parsed: Vec<(String, TokenStream2)> = Vec::new();
    for entry in entries(group)? {
        if entry.name == "id" {
            return Err("`id` is reserved".into());
        }
        if parsed.iter().any(|(name, _)| *name == entry.name) {
            return Err(format!("duplicate attribute `{}`", entry.name));
        }
        let tokens = attr_tokens(&entry)?;
        parsed.push((entry.name, tokens));
    }
    Ok(parsed
        .into_iter()
        .map(|(name, tokens)| AttrDecl { name, tokens })
        .collect())
}

/// One attribute type after parsing.
struct Typed {
    tokens: TokenStream2,
    defaultable: bool,
}

fn attr_tokens(entry: &Entry) -> Result<TokenStream2, String> {
    let mut tokens: Peekable<TokenIter> = entry.value.clone().into_iter().peekable();
    let kind = parse_type(&mut tokens)?;
    let required = tokens
        .peek()
        .is_some_and(|token| matches!(token, TokenTree::Ident(ident) if ident == "required"));
    if required {
        tokens.next();
    }
    let default = parse_default_of(&mut tokens, &kind, &entry.name)?;
    let name = proc_macro2::Literal::string(&entry.name);
    let kind = kind.tokens;
    let mut spec = quote! { ::rich_editor_auy::schema::AttrSpec::new(#name, #kind) };
    if required {
        spec = quote! { #spec.required() };
    }
    if let Some(default) = default {
        spec = quote! { #spec.with_default(#default) };
    }
    Ok(spec)
}

/// Parses the optional `= literal` suffix of one attribute declaration.
fn parse_default_of(
    tokens: &mut Peekable<TokenIter>,
    kind: &Typed,
    name: &str,
) -> Result<Option<TokenStream2>, String> {
    let is_default = tokens
        .peek()
        .is_some_and(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == '='));
    if !is_default {
        return Ok(None);
    }
    tokens.next();
    if !kind.defaultable {
        return Err(format!("unsupported attribute default for `{name}`"));
    }
    Ok(Some(parse_default(tokens.by_ref().collect())?))
}

#[allow(clippy::too_many_lines, reason = "one grammar arm per attribute type")]
fn parse_type(tokens: &mut Peekable<TokenIter>) -> Result<Typed, String> {
    let Some(TokenTree::Ident(kind)) = tokens.next() else {
        return Err("expected attribute type".into());
    };
    match kind.to_string().as_str() {
        "str" => Ok(Typed {
            tokens: quote! { ::rich_editor_auy::schema::AttrType::Str },
            defaultable: true,
        }),
        "bool" => Ok(Typed {
            tokens: quote! { ::rich_editor_auy::schema::AttrType::Bool },
            defaultable: true,
        }),
        "ref" => Ok(Typed {
            tokens: quote! { ::rich_editor_auy::schema::AttrType::Ref },
            defaultable: false,
        }),
        "any" => Ok(Typed {
            tokens: quote! { ::rich_editor_auy::schema::AttrType::Any },
            defaultable: false,
        }),
        "int" => {
            let group = next_group(tokens, "int(min..=max)")?;
            let parts = split_top_level(group.stream(), "..=")?;
            if parts.len() != 2 {
                return Err("int requires `(min..=max)`".into());
            }
            let min = int_literal(parts[0].clone())?;
            let max = int_literal(parts[1].clone())?;
            Ok(Typed {
                tokens: quote! {
                    ::rich_editor_auy::schema::AttrType::Int { min: #min, max: #max }
                },
                defaultable: true,
            })
        }
        "enum" => {
            let group = next_group(tokens, "enum(\"a\", \"b\")")?;
            let mut values = Vec::new();
            for part in split_top_level(group.stream(), ",")? {
                values.push(string_literal(&part)?);
            }
            if values.is_empty() {
                return Err("enum requires at least one value".into());
            }
            Ok(Typed {
                tokens: quote! {
                    ::rich_editor_auy::schema::AttrType::enumerated(&[#(#values),*])
                },
                defaultable: true,
            })
        }
        "list" | "map" => {
            let nested = kind.to_string();
            let group = next_group(tokens, "list(type)")?;
            let mut inner = group.stream().into_iter().peekable();
            let parsed = parse_type(&mut inner)?;
            if inner.peek().is_some() {
                return Err(format!("unexpected tokens in `{nested}` type"));
            }
            let inner = parsed.tokens;
            let tokens = if nested == "list" {
                quote! { ::rich_editor_auy::schema::AttrType::list(#inner) }
            } else {
                quote! { ::rich_editor_auy::schema::AttrType::map(#inner) }
            };
            Ok(Typed {
                tokens,
                defaultable: false,
            })
        }
        other => Err(format!("unsupported attribute type `{other}`")),
    }
}

fn next_group(tokens: &mut Peekable<TokenIter>, what: &str) -> Result<Group, String> {
    let Some(TokenTree::Group(group)) = tokens.next() else {
        return Err(format!("expected `{what}`"));
    };
    Ok(group)
}

fn parse_default(tokens: TokenStream2) -> Result<TokenStream2, String> {
    let mut iter = tokens.into_iter();
    let Some(token) = iter.next() else {
        return Err("empty attribute default".into());
    };
    if iter.next().is_some() {
        return Err("attribute default must be a literal".into());
    }
    match token {
        TokenTree::Literal(literal) => {
            let text = literal.to_string();
            if text.starts_with('"') {
                Ok(quote! { ::rich_editor_auy::model::AttrValue::Str(#literal.into()) })
            } else if text == "true" || text == "false" {
                let value: bool = text == "true";
                Ok(quote! { ::rich_editor_auy::model::AttrValue::Bool(#value) })
            } else if let Ok(value) = text.parse::<i64>() {
                Ok(quote! { ::rich_editor_auy::model::AttrValue::Int(#value) })
            } else {
                Err("unsupported attribute default".into())
            }
        }
        _ => Err("attribute default must be a literal".into()),
    }
}
