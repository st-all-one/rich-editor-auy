//! Parser do `html!`: tokens → [`Block`]. Cada erro tem span, código e mensagem de contrato.

use crate::ast::{
    Attr, AttrValue, Block, Child, Element, ForNode, IfNode, Item, Message, Otherwise, Primitive,
};
use crate::diagnostic::Diag;
use crate::expr;
use crate::table;
use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenStream, TokenTree};

/// Aninhamento máximo de blocos: entrada patológica vira erro, não estouro de pilha.
const MAX_DEPTH: usize = 64;

/// Cursor sobre a lista de tokens de um bloco.
#[derive(Debug, Clone)]
pub(crate) struct Cursor {
    tokens: Vec<TokenTree>,
    index: usize,
    /// Nível do bloco na árvore (`0` no `html!` raiz).
    depth: usize,
}

impl Cursor {
    pub(crate) const fn new(tokens: Vec<TokenTree>) -> Self {
        Self {
            tokens,
            index: 0,
            depth: 0,
        }
    }

    pub(crate) fn peek(&self) -> Option<&TokenTree> {
        self.tokens.get(self.index)
    }

    pub(crate) fn peek_at(&self, offset: usize) -> Option<&TokenTree> {
        self.tokens.get(self.index.saturating_add(offset))
    }

    pub(crate) fn bump(&mut self) -> Option<TokenTree> {
        let token = self.tokens.get(self.index).cloned()?;
        self.index = self.index.saturating_add(1);
        Some(token)
    }

    pub(crate) fn check_punct(&self, want: char) -> bool {
        matches!(self.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == want)
    }

    pub(crate) fn check_punct_pair(&self, first: char, second: char) -> bool {
        self.check_punct(first)
            && matches!(self.peek_at(1), Some(TokenTree::Punct(punct)) if punct.as_char() == second)
    }

    pub(crate) fn check_ident(&self, want: &str) -> bool {
        matches!(self.peek(), Some(TokenTree::Ident(ident)) if ident == want)
    }

    fn check_brace(&self) -> bool {
        matches!(self.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
    }

    fn take_brace(&mut self) -> Option<Group> {
        match self.peek() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let group = group.clone();
                self.bump()?;
                Some(group)
            }
            _ => None,
        }
    }

    /// Descarta um grupo `{ … }` que sobrou de uma forma inválida (evita erro em cascata).
    fn skip_brace(&mut self) {
        if self.check_brace() {
            self.bump();
        }
    }

    fn span(&self) -> Span {
        self.peek().map_or_else(Span::call_site, TokenTree::span)
    }
}

/// Converte o fluxo de tokens em AST. Qualquer erro aborta o codegen (não há expansão parcial).
pub(crate) fn parse(input: TokenStream) -> Result<Block, Vec<Diag>> {
    let mut cursor = Cursor::new(input.into_iter().collect());
    let mut diags = Vec::new();
    let block = parse_block(&mut cursor, &mut diags);
    if diags.is_empty() {
        Ok(block)
    } else {
        Err(diags)
    }
}

fn parse_block(cursor: &mut Cursor, diags: &mut Vec<Diag>) -> Block {
    if cursor.depth > MAX_DEPTH {
        if cursor.depth == MAX_DEPTH.saturating_add(1) {
            diags.push(
                Diag::new(
                    cursor.span(),
                    format!("aninhamento de `html!` excede {MAX_DEPTH} níveis"),
                )
                .hint("divida o bloco em componentes"),
            );
        }
        return Block { items: Vec::new() };
    }
    let mut items = Vec::new();
    while let Some(token) = cursor.peek().cloned() {
        let before = cursor.index;
        match token {
            TokenTree::Punct(punct) if punct.as_char() == '@' => {
                cursor.bump();
                if let Some(item) = parse_directive(cursor, diags, punct.span()) {
                    items.push(item);
                }
            }
            TokenTree::Ident(ident) if ident == "for" => {
                cursor.bump();
                if let Some(item) = parse_for(cursor, diags, ident.span()) {
                    items.push(Item::For(item));
                }
            }
            TokenTree::Ident(_) if looks_like_element(cursor) => {
                if let Some(item) = parse_element(cursor, diags) {
                    items.push(item);
                }
            }
            _ => {
                if let Some(item) = parse_expression(cursor, diags) {
                    items.push(item);
                } else {
                    cursor.bump();
                }
            }
        }
        if cursor.index == before && cursor.peek().is_some() {
            // Garantia de progresso: nenhum caminho pode devolver o mesmo token.
            cursor.bump();
        }
    }
    Block { items }
}

/// Um elemento começa com `Ident` do vocabulário, ou com `Ident` seguido de outro `Ident`
/// (atributo), de um grupo `{}` ou do `;` do `void`. Qualquer outra coisa (`::`, `.`) é expressão.
fn looks_like_element(cursor: &Cursor) -> bool {
    if let Some(TokenTree::Ident(ident)) = cursor.peek()
        && table::element(&ident.to_string()).is_some()
    {
        return true;
    }
    match cursor.peek_at(1) {
        Some(TokenTree::Ident(_)) => true,
        Some(TokenTree::Group(group)) => group.delimiter() == Delimiter::Brace,
        Some(TokenTree::Punct(punct)) => punct.as_char() == ';',
        _ => false,
    }
}

fn parse_element(cursor: &mut Cursor, diags: &mut Vec<Diag>) -> Option<Item> {
    let Some(TokenTree::Ident(ident)) = cursor.bump() else {
        return None;
    };
    let name = ident.to_string();
    let span = ident.span();
    if !name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        diags.push(Diag::new(span, "nome de elemento inválido"));
        return None;
    }

    let mut attrs = Vec::new();
    loop {
        match cursor.peek().cloned() {
            Some(TokenTree::Ident(attr)) => {
                if let Some(parsed) = parse_attr(cursor, diags, &attr) {
                    attrs.push(parsed);
                }
            }
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let group = cursor.take_brace()?;
                let body = parse_group(&group, diags, cursor.depth.saturating_add(1));
                return Some(Item::Element(Element {
                    name,
                    span,
                    attrs,
                    body: Some(body),
                }));
            }
            Some(TokenTree::Punct(punct)) if punct.as_char() == ';' => {
                cursor.bump();
                return Some(Item::Element(Element {
                    name,
                    span,
                    attrs,
                    body: None,
                }));
            }
            Some(other) => {
                diags.push(Diag::new(other.span(), "token inesperado em `html!`").hint(
                    "atributos vêm antes do corpo; o corpo é `{ … }` e o elemento vazio termina em `;`",
                ));
                return None;
            }
            None => {
                let void = table::element(&name).is_some_and(|spec| spec.void);
                let message = if void {
                    format!("`{name}` é void: termine com `;`")
                } else {
                    format!("`{name}` exige corpo: `{name} {{ … }}`")
                };
                diags.push(Diag::new(span, message));
                return Some(Item::Element(Element {
                    name,
                    span,
                    attrs,
                    body: None,
                }));
            }
        }
    }
}

fn parse_attr(cursor: &mut Cursor, diags: &mut Vec<Diag>, ident: &Ident) -> Option<Attr> {
    let name = ident.to_string();
    let span = ident.span();
    cursor.bump();
    let optional = cursor.check_punct('?');
    if optional {
        cursor.bump();
    }
    if !cursor.check_punct('=') {
        diags.push(Diag::new(
            span,
            format!("atributo `{name}` sem valor; use `{name}=(…)`"),
        ));
        return None;
    }
    cursor.bump();
    match cursor.bump() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            let expr = group.stream();
            let expr_span = group.span();
            Some(Attr {
                name,
                span,
                value: AttrValue::Computed {
                    expr,
                    span: expr_span,
                },
                optional,
            })
        }
        Some(TokenTree::Literal(literal)) => {
            let Some(text) = literal_text(&literal) else {
                diags.push(Diag::new(literal.span(), "literal de texto inválido"));
                return None;
            };
            Some(Attr {
                name,
                span,
                value: AttrValue::Literal {
                    text,
                    span: literal.span(),
                },
                optional,
            })
        }
        other => {
            let at = other.map_or(span, |token| token.span());
            diags.push(Diag::new(
                at,
                format!("`{name}` espera `(expr)` ou `\"literal\"`"),
            ));
            None
        }
    }
}

/// Desescapa um literal de string comum (`"…"`), sem sufixo e sem raw string.
fn literal_text(literal: &Literal) -> Option<String> {
    let raw = literal.to_string();
    if let Some(content) = raw_text(&raw) {
        return Some(String::from(content));
    }
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    unescape(inner)
}

/// Conteúdo de um literal `r"…"`/`r#"…"#` (a parte entre as aspas).
fn raw_text(raw: &str) -> Option<&str> {
    let rest = raw.strip_prefix('r')?;
    let hashes = rest.chars().take_while(|ch| *ch == '#').count();
    let body = rest[hashes..].strip_prefix('"')?;
    let close = hashes.checked_add(1)?;
    if body.len() < close {
        return None;
    }
    let (content, tail) = body.split_at(body.len().checked_sub(close)?);
    if tail.starts_with('"') && tail[1..].chars().all(|ch| ch == '#') {
        Some(content)
    } else {
        None
    }
}

/// Desescapa uma string normal do Rust (o tokenizador já garantiu a forma).
fn unescape(inner: &str) -> Option<String> {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let escaped = chars.next()?;
        match escaped {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            '0' => out.push('\0'),
            '\'' => out.push('\''),
            'x' => {
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                out.push(char::from_u32(hi.saturating_mul(16).saturating_add(lo))?);
            }
            'u' => {
                if chars.next()? != '{' {
                    return None;
                }
                let mut hex = String::new();
                loop {
                    match chars.next()? {
                        '}' => break,
                        '_' => {}
                        digit => hex.push(digit),
                    }
                }
                out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
            }
            '\n' => {
                // Continuação de linha: o Rust remove o `\` + nova linha e o espaço seguinte.
                for next in chars.by_ref() {
                    if !next.is_whitespace() {
                        out.push(next);
                        break;
                    }
                }
            }
            _ => return None,
        }
    }
    Some(out)
}

fn parse_directive(cursor: &mut Cursor, diags: &mut Vec<Diag>, span: Span) -> Option<Item> {
    if cursor.check_ident("if") {
        cursor.bump();
        return parse_if(cursor, diags, span).map(Item::If);
    }
    if cursor.check_ident("else") {
        let else_span = cursor.span();
        diags.push(Diag::new(else_span, "`@else` sem `@if`"));
        return None;
    }
    let name = match cursor.peek() {
        Some(TokenTree::Ident(ident)) => ident.to_string(),
        _ => String::from("?"),
    };
    diags.push(
        Diag::new(span, format!("diretiva desconhecida `@{name}`"))
            .hint("as diretivas são `@if`, `@else if` e `@else`"),
    );
    None
}

fn parse_if(cursor: &mut Cursor, diags: &mut Vec<Diag>, span: Span) -> Option<IfNode> {
    let mut condition = Vec::new();
    while let Some(token) = cursor.peek() {
        if matches!(token, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace) {
            break;
        }
        condition.push(cursor.bump()?);
    }
    let Some(group) = cursor.take_brace() else {
        diags.push(Diag::new(span, "`@if` exige um corpo `{ … }`"));
        return None;
    };
    let then = parse_group(&group, diags, cursor.depth.saturating_add(1));

    let mut otherwise = None;
    if cursor.check_punct('@')
        && matches!(cursor.peek_at(1), Some(TokenTree::Ident(ident)) if ident == "else")
    {
        cursor.bump();
        cursor.bump();
        if cursor.check_ident("if") {
            let if_span = cursor.span();
            cursor.bump();
            otherwise = parse_if(cursor, diags, if_span)
                .map(Box::new)
                .map(Otherwise::If);
        } else {
            let Some(group) = cursor.take_brace() else {
                diags.push(Diag::new(span, "`@else` exige um corpo `{ … }`"));
                return None;
            };
            otherwise = Some(Otherwise::Block(parse_group(
                &group,
                diags,
                cursor.depth.saturating_add(1),
            )));
        }
    }

    Some(IfNode {
        condition: condition.into_iter().collect(),
        span,
        then,
        otherwise,
    })
}

fn parse_for(cursor: &mut Cursor, diags: &mut Vec<Diag>, span: Span) -> Option<ForNode> {
    let mut pattern = Vec::new();
    let mut iterable = Vec::new();
    let mut in_pattern = true;
    while let Some(token) = cursor.peek() {
        if matches!(token, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace) {
            break;
        }
        if in_pattern && matches!(token, TokenTree::Ident(ident) if ident == "in") {
            cursor.bump();
            in_pattern = false;
            continue;
        }
        if in_pattern {
            pattern.push(cursor.bump()?);
        } else {
            iterable.push(cursor.bump()?);
        }
    }
    if in_pattern || iterable.is_empty() {
        diags.push(Diag::new(span, "`for` exige `in` e um corpo `{ … }`"));
        cursor.skip_brace();
        return None;
    }
    let Some(group) = cursor.take_brace() else {
        diags.push(Diag::new(span, "`for` exige um corpo `{ … }`"));
        return None;
    };
    Some(ForNode {
        pattern: pattern.into_iter().collect(),
        iterable: iterable.into_iter().collect(),
        span,
        body: parse_group(&group, diags, cursor.depth.saturating_add(1)),
    })
}

fn parse_expression(cursor: &mut Cursor, diags: &mut Vec<Diag>) -> Option<Item> {
    if let Some(TokenTree::Literal(literal)) = cursor.peek() {
        diags.push(
            Diag::new(
                literal.span(),
                "texto literal não é permitido; use uma variante de `Msg`",
            )
            .hint(
                "mensagem estática: `Msg::Texto`; dado dinâmico: `Text::new(&valor)`; composição: `(minha_markup)`",
            ),
        );
        cursor.bump();
        return None;
    }
    if cursor.check_brace() {
        diags.push(Diag::new(
            cursor.span(),
            "bloco inesperado em `html!`; escreva a expressão entre parênteses",
        ));
        cursor.bump();
        return None;
    }
    let Some(scanned) = expr::scan(cursor) else {
        diags.push(Diag::new(cursor.span(), "token inesperado em `html!`"));
        cursor.bump();
        return None;
    };
    if let Some(group) = cursor.take_brace() {
        let children = parse_group(&group, diags, cursor.depth.saturating_add(1));
        return Some(Item::Primitive(Primitive {
            callee: scanned.tokens,
            children,
            span: scanned.span,
        }));
    }
    if scanned.pure_path {
        Some(Item::Message(Message {
            path: scanned.tokens,
            span: scanned.span,
        }))
    } else {
        Some(Item::Child(Child {
            expr: scanned.tokens,
            span: scanned.span,
        }))
    }
}

fn parse_group(group: &Group, diags: &mut Vec<Diag>, depth: usize) -> Block {
    let mut cursor = Cursor::new(group.stream().into_iter().collect());
    cursor.depth = depth;
    parse_block(&mut cursor, diags)
}
