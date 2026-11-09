#![allow(
    clippy::redundant_pub_crate,
    reason = "the module is private; its items are crate-visible on purpose"
)]
//! Token helpers shared by the schema front-end.
#![allow(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "token parsing over a bounded, validated stream"
)]

use proc_macro2::{Delimiter, Group, Literal, TokenStream as TokenStream2, TokenTree};
use quote::ToTokens;

/// One `name: value` pair inside a declaration body.
#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) value: TokenStream2,
}

impl Entry {
    /// Returns the entry when it is the same kind of declaration body.
    pub(crate) fn group(&self, delimiter: Delimiter, what: &str) -> Result<Group, String> {
        single_group(self.value.clone(), delimiter, what)
    }
}

/// Returns an integer literal.
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

/// Returns a string literal.
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

/// Returns the value of a string literal.
pub(crate) fn string_value(stream: &TokenStream2) -> Result<String, String> {
    let literal = string_literal(stream)?;
    let text = literal.to_string();
    Ok(text.trim_matches('"').to_owned())
}

/// Returns a boolean literal.
pub(crate) fn bool_literal(stream: &TokenStream2) -> Result<bool, String> {
    match stream.to_string().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("expected boolean literal".into()),
    }
}

/// Returns the single group of the expected delimiter.
pub(crate) fn single_group(
    stream: TokenStream2,
    delimiter: Delimiter,
    what: &str,
) -> Result<Group, String> {
    let mut iter = stream.into_iter();
    let Some(TokenTree::Group(group)) = iter.next() else {
        return Err(format!("expected `{what}` group"));
    };
    if group.delimiter() != delimiter || iter.next().is_some() {
        return Err(format!("expected single `{what}` group"));
    }
    Ok(group)
}

/// Splits a token stream at top-level separators.
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

/// Parses a declaration body into `name: value` entries.
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

/// Visibility of the generated module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visibility {
    /// `pub(crate) mod`.
    Public,
    /// Private `mod`.
    Private,
}

/// Module mode declaration.
#[derive(Debug)]
pub(crate) struct ModuleDecl {
    pub(crate) vis: Visibility,
    pub(crate) name: String,
}

/// Token cursor over the macro input.
pub(crate) struct Cursor {
    tokens: Vec<TokenTree>,
    index: usize,
}

impl Cursor {
    /// Creates a cursor.
    pub(crate) fn new(stream: TokenStream2) -> Self {
        Self {
            tokens: stream.into_iter().collect(),
            index: 0,
        }
    }

    /// Returns whether the input is exhausted.
    pub(crate) fn is_empty(&self) -> bool {
        self.index >= self.tokens.len()
    }

    /// Consumes one token.
    pub(crate) fn bump(&mut self) {
        self.index = self.index.saturating_add(1);
    }

    /// Returns the identifier `offset` tokens ahead without consuming it.
    fn ident_at(&self, offset: usize) -> Option<String> {
        match self.tokens.get(self.index.saturating_add(offset)) {
            Some(TokenTree::Ident(ident)) => Some(ident.to_string()),
            _ => None,
        }
    }

    /// Parses an optional `pub(crate) mod NAME;` or `mod NAME;` prefix.
    pub(crate) fn module(&mut self) -> Option<ModuleDecl> {
        let first = self.ident_at(0)?;
        let (vis, keyword) = match first.as_str() {
            "pub" => (Visibility::Public, 1),
            "mod" => (Visibility::Private, 0),
            _ => return None,
        };
        if self.ident_at(keyword).as_deref() != Some("mod") {
            return None;
        }
        let name = self.ident_at(keyword.saturating_add(1))?;
        let semicolon = self
            .tokens
            .get(self.index.saturating_add(keyword).saturating_add(2))?;
        match semicolon {
            TokenTree::Punct(punct) if punct.as_char() == ';' => {}
            _ => return None,
        }
        self.index = self.index.saturating_add(keyword).saturating_add(3);
        Some(ModuleDecl { vis, name })
    }

    /// Consumes an identifier.
    pub(crate) fn expect_ident(&mut self, what: &str) -> Result<String, String> {
        let Some(TokenTree::Ident(ident)) = self.tokens.get(self.index) else {
            return Err(format!("expected {what}"));
        };
        let name = ident.to_string();
        self.bump();
        Ok(name)
    }

    /// Consumes a braced group.
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
        self.bump();
        Ok(group)
    }
}

impl ToTokens for Entry {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.value.to_tokens(tokens);
    }
}
