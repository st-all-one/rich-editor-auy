#![allow(
    clippy::redundant_pub_crate,
    reason = "the module is private; its items are shared within the proc-macro crate"
)]

//! Declarative token parser for `schema!`.

use proc_macro2::{Delimiter, Group, Literal, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, quote};

pub(crate) fn parse_attr_type(
    tokens: &mut impl Iterator<Item = TokenTree>,
) -> Result<TokenStream2, String> {
    let Some(TokenTree::Ident(kind)) = tokens.next() else {
        return Err("expected attribute type".into());
    };
    match kind.to_string().as_str() {
        "str" => Ok(quote! { ::rich_editor_auy::schema::AttrType::Str }),
        "bool" => Ok(quote! { ::rich_editor_auy::schema::AttrType::Bool }),
        "ref" => Ok(quote! { ::rich_editor_auy::schema::AttrType::Ref }),
        "any" => Ok(quote! { ::rich_editor_auy::schema::AttrType::Any }),
        "int" => {
            let Some(TokenTree::Group(range)) = tokens.next() else {
                return Err("int requires `(min..=max)`".into());
            };
            let parts = split_top_level(range.stream(), "..=")?;
            if parts.len() != 2 {
                return Err("int requires `(min..=max)`".into());
            }
            let min = int_literal(parts[0].clone())?;
            let max = int_literal(parts[1].clone())?;
            Ok(quote! {
                ::rich_editor_auy::schema::AttrType::Int { min: #min, max: #max }
            })
        }
        other => Err(format!("unsupported attribute type `{other}`")),
    }
}

pub(crate) fn parse_default(tokens: TokenStream2) -> Result<TokenStream2, String> {
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

pub(crate) fn int_literal(stream: TokenStream2) -> Result<Literal, String> {
    let mut iter = stream.into_iter();
    let Some(TokenTree::Literal(literal)) = iter.next() else {
        return Err("expected integer literal".into());
    };
    if iter.next().is_some() {
        return Err("expected integer literal".into());
    }
    Ok(literal)
}

pub(crate) fn string_literal(stream: &TokenStream2) -> Result<Literal, String> {
    let mut iter = stream.clone().into_iter();
    let Some(TokenTree::Literal(literal)) = iter.next() else {
        return Err("expected string literal".into());
    };
    if iter.next().is_some() || !literal.to_string().starts_with('"') {
        return Err("expected string literal".into());
    }
    Ok(literal)
}

pub(crate) fn bool_literal(stream: &TokenStream2) -> Result<bool, String> {
    let text = stream.to_string();
    match text.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("expected boolean literal".into()),
    }
}

pub(crate) fn single_group(
    stream: TokenStream2,
    delimiter: Delimiter,
    name: &str,
) -> Result<Group, String> {
    let mut iter = stream.into_iter();
    let Some(TokenTree::Group(group)) = iter.next() else {
        return Err(format!("expected `{name}` group"));
    };
    if group.delimiter() != delimiter || iter.next().is_some() {
        return Err(format!("expected single `{name}` group"));
    }
    Ok(group)
}

pub(crate) fn split_top_level(
    stream: TokenStream2,
    separator: &str,
) -> Result<Vec<TokenStream2>, String> {
    let separator: Vec<char> = separator.chars().collect();
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    let mut index = 0;
    while index < tokens.len() {
        if separator.len() == 1
            && let TokenTree::Punct(punct) = &tokens[index]
            && punct.as_char() == separator[0]
        {
            parts.push(current.drain(..).collect());
            index += 1;
            continue;
        }
        if separator.len() == 3
            && let [
                TokenTree::Punct(a),
                TokenTree::Punct(b),
                TokenTree::Punct(c),
            ] = &tokens[index..index + 3.min(tokens.len() - index)]
            && a.as_char() == '.'
            && b.as_char() == '.'
            && c.as_char() == '='
        {
            parts.push(current.drain(..).collect());
            index += 3;
            continue;
        }
        current.push(tokens[index].clone());
        index += 1;
    }
    if !current.is_empty() {
        parts.push(current.into_iter().collect());
    }
    Ok(parts)
}

#[derive(Debug)]
pub(crate) struct Entry {
    pub name: String,
    pub value: TokenStream2,
}

pub(crate) fn entries(group: Group) -> Result<Vec<Entry>, String> {
    let mut result = Vec::new();
    for part in split_top_level(group.stream(), ",")? {
        let mut iter = part.into_iter();
        let Some(TokenTree::Ident(name)) = iter.next() else {
            return Err("expected field name".into());
        };
        let name = name.to_string();
        let rest: Vec<TokenTree> = iter.collect();
        let value = if rest.is_empty() {
            TokenStream2::new()
        } else {
            let Some(TokenTree::Punct(colon)) = rest.first() else {
                return Err(format!("expected `:` after `{name}`"));
            };
            if colon.as_char() != ':' {
                return Err(format!("expected `:` after `{name}`"));
            }
            rest.into_iter().skip(1).collect()
        };
        result.push(Entry { name, value });
    }
    Ok(result)
}

pub(crate) struct Cursor {
    tokens: Vec<TokenTree>,
    index: usize,
}

impl Cursor {
    pub(crate) fn new(stream: TokenStream2) -> Self {
        Self {
            tokens: stream.into_iter().collect(),
            index: 0,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub(crate) fn expect_ident(&mut self, what: &str) -> Result<String, String> {
        let Some(TokenTree::Ident(ident)) = self.tokens.get(self.index) else {
            return Err(format!("expected {what}"));
        };
        let name = ident.to_string();
        self.index += 1;
        Ok(name)
    }

    pub(crate) fn expect_group(
        &mut self,
        delimiter: Delimiter,
        what: &str,
    ) -> Result<Group, String> {
        let Some(TokenTree::Group(group)) = self.tokens.get(self.index) else {
            return Err(format!("expected {what}"));
        };
        if group.delimiter() != delimiter {
            return Err(format!("expected {what}"));
        }
        let group = group.clone();
        self.index += 1;
        Ok(group)
    }
}

impl ToTokens for Entry {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.value.to_tokens(tokens);
    }
}
