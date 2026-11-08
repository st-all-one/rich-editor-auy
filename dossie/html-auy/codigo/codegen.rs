//! Codegen: AST → escrita de bytes.
//!
//! Cada item vira uma chamada ao `__private::Writer`, e o escritor já sabe a forma canônica:
//! ordem de atributo, presença de booleano e escape. O macro não indenta nada — a marcação do
//! fonte é irrelevante para os bytes.

use crate::ast::{
    Attr, AttrValue, Block, Child, Element, ForNode, IfNode, Item, Message, Otherwise, Primitive,
};
use crate::attrs::is_boolean;
use crate::table;
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};

/// Bytes estimados por filho interpolado (mensagem/composição).
const INTERPOLATION_BYTES: usize = 24;

/// Expande o bloco num `::html_auy::Markup`.
pub(crate) fn expand(block: &Block) -> TokenStream {
    let writer = format_ident!("__html_w0", span = Span::mixed_site());
    let capacity = estimate(block);
    let body = emit_block(block, &writer, 0);
    quote! {
        {
            let mut #writer = ::html_auy::__private::Writer::with_capacity(#capacity);
            #body
            #writer.finish()
        }
    }
}

fn emit_block(block: &Block, writer: &Ident, depth: usize) -> TokenStream {
    let mut out = TokenStream::new();
    for item in &block.items {
        out.extend(emit_item(item, writer, depth));
    }
    out
}

fn emit_item(item: &Item, writer: &Ident, depth: usize) -> TokenStream {
    match item {
        Item::Element(element) => emit_element(element, writer, depth),
        Item::Message(Message { path, span }) => {
            quote_spanned! {*span=> #writer.msg(#path); }
        }
        Item::Child(Child { expr, span }) => {
            quote_spanned! {*span=> #writer.child(&(#expr)); }
        }
        Item::Primitive(primitive) => emit_primitive(primitive, writer, depth),
        Item::If(node) => emit_if(node, writer, depth),
        Item::For(ForNode {
            pattern,
            iterable,
            span,
            body,
        }) => {
            let children = emit_block(body, writer, depth);
            quote_spanned! {*span=> for #pattern in #iterable { #children } }
        }
    }
}

fn emit_if(node: &IfNode, writer: &Ident, depth: usize) -> TokenStream {
    let condition = &node.condition;
    let then = emit_block(&node.then, writer, depth);
    let span = node.span;
    match &node.otherwise {
        None => quote_spanned! {span=> if #condition { #then } },
        Some(Otherwise::Block(block)) => {
            let otherwise = emit_block(block, writer, depth);
            quote_spanned! {span=> if #condition { #then } else { #otherwise } }
        }
        Some(Otherwise::If(inner)) => {
            let nested = emit_if(inner, writer, depth);
            quote_spanned! {span=> if #condition { #then } else #nested }
        }
    }
}

fn emit_primitive(primitive: &Primitive, writer: &Ident, depth: usize) -> TokenStream {
    let inner = depth.saturating_add(1);
    let child_writer = format_ident!("__html_w{}", inner, span = Span::mixed_site());
    let capacity = estimate(&primitive.children);
    let children = emit_block(&primitive.children, &child_writer, inner);
    let argument = quote! {
        {
            let mut #child_writer = ::html_auy::__private::Writer::with_capacity(#capacity);
            #children
            #child_writer.finish()
        }
    };
    let callee = append_argument(&primitive.callee, argument);
    let span = primitive.span;
    quote_spanned! {span=> #writer.child(&(#callee)); }
}

/// Insere o `Markup` dos filhos como último argumento da chamada (`Form::open(a, b, c, filhos)`).
fn append_argument(callee: &TokenStream, argument: TokenStream) -> TokenStream {
    let mut tokens: Vec<TokenTree> = callee.clone().into_iter().collect();
    let Some(TokenTree::Group(group)) = tokens.pop() else {
        return callee.clone();
    };
    if group.delimiter() != Delimiter::Parenthesis {
        tokens.push(TokenTree::Group(group));
        return tokens.into_iter().collect();
    }
    let mut inner: Vec<TokenTree> = group.stream().into_iter().collect();
    if !inner.is_empty() {
        inner.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
    }
    inner.extend(argument);
    tokens.push(TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        inner.into_iter().collect(),
    )));
    tokens.into_iter().collect()
}

/// Um atributo planejado para emissão: do fonte ou sintetizado pelo gerador.
struct Planned {
    name: String,
    parsed: Option<usize>,
    synthetic: Option<&'static str>,
    span: Span,
}

fn emit_element(element: &Element, writer: &Ident, depth: usize) -> TokenStream {
    let spec = table::element(&element.name);
    let tag = spec.map_or(element.name.as_str(), |spec| spec.tag);
    let tag_literal = Literal::string(tag);
    let span = element.span;
    let open = quote_spanned! {span=> #writer.open(#tag_literal); };

    let planned = plan_attrs(element, spec);
    let mut attrs = TokenStream::new();
    for item in &planned {
        if let Some(index) = item.parsed
            && let Some(attr) = element.attrs.get(index)
        {
            attrs.extend(emit_attr(element, attr, &item.name, writer));
        } else if let Some(value) = item.synthetic {
            let name = Literal::string(&item.name);
            let value = Literal::string(value);
            let at = item.span;
            attrs.extend(quote_spanned! {at=> #writer.attr_static(#name, #value); });
        }
    }

    // `svg_shape` é void no spec, mas em SVG o `/>` é o que fecha o elemento (sem ele, um `circle`
    // seguinte vira filho do anterior).
    let self_closing = spec.is_some_and(|spec| spec.categories.contains(&"svg_shape"));
    let close_open = if self_closing {
        quote_spanned! {span=> #writer.close_open_self(); }
    } else {
        quote_spanned! {span=> #writer.close_open(); }
    };
    let children = element
        .body
        .as_ref()
        .map_or_else(TokenStream::new, |body| emit_block(body, writer, depth));
    let close = if spec.is_some_and(|spec| spec.void) {
        TokenStream::new()
    } else {
        quote_spanned! {span=> #writer.close(#tag_literal); }
    };
    quote! { #open #attrs #close_open #children #close }
}

fn canonical_class(value: &str) -> String {
    let mut tokens: Vec<&str> = value.split_whitespace().collect();
    tokens.sort_unstable();
    tokens.dedup();
    tokens.join(" ")
}

fn emit_attr(element: &Element, attr: &Attr, name: &str, writer: &Ident) -> TokenStream {
    let name_literal = Literal::string(name);
    match &attr.value {
        AttrValue::Literal { text, span } => {
            if is_boolean(name) {
                // O validador recusa; sem codegen para não emitir `disabled="false"`.
                return TokenStream::new();
            }
            let at = *span;
            if name == "class" {
                let normalized = canonical_class(text);
                if normalized.is_empty() {
                    return TokenStream::new();
                }
                let value = Literal::string(&normalized);
                return quote_spanned! {at=> #writer.attr_static(#name_literal, #value); };
            }
            let value = Literal::string(text);
            quote_spanned! {at=> #writer.attr_static(#name_literal, #value); }
        }
        AttrValue::Computed { expr, span } => {
            let at = *span;
            if is_boolean(name) {
                let flag = quote_spanned! {at=> #writer.attr_flag(#name_literal, #expr); };
                return if attr.optional {
                    quote_spanned! {at=> if let Some(__v) = &(#expr) { #writer.attr_flag(#name_literal, *__v); } }
                } else {
                    flag
                };
            }
            let call = |value: &TokenStream| {
                attr_call(&element.name, name, writer, &name_literal, value, at)
            };
            if attr.optional {
                let call = call(&quote! { __v });
                return quote_spanned! {at=> if let Some(__v) = &(#expr) { #call; } };
            }
            let call = call(&quote! { &(#expr) });
            quote_spanned! {at=> #call; }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "espelha a assinatura do método `Writer` gerado; agrupar em struct não ajuda o leitor"
)]
fn attr_call(
    element: &str,
    name: &str,
    writer: &Ident,
    name_literal: &Literal,
    value: &TokenStream,
    span: Span,
) -> TokenStream {
    let method = match name {
        "class" => "attr_class",
        "sizes" => "attr_sizes",
        "srcset" => "attr_srcset",
        "href" => "attr_href",
        "src" | "poster" => "attr_src",
        "alt" => "attr_alt",
        "aria-label" => "attr_label",
        "id" => "attr_id",
        "name" => "attr_field_name",
        "datetime" => "attr_timestamp",
        // Dimensão tipada só onde o plano tipa (`img`/`video`/`iframe`); em SVG é genérica.
        "width" if matches!(element, "img" | "video" | "iframe") => "attr_width",
        "height" if matches!(element, "img" | "video" | "iframe") => "attr_height",
        _ => {
            let method = format_ident!("attr", span = Span::mixed_site());
            return quote_spanned! {span=> #writer.#method(#name_literal, #value) };
        }
    };
    let method = format_ident!("{}", method, span = Span::mixed_site());
    quote_spanned! {span=> #writer.#method(#value) }
}

/// Ordena e completa os atributos do elemento (inclusive os que o gerador sintetiza).
fn plan_attrs(element: &Element, spec: Option<&'static table::Element>) -> Vec<Planned> {
    let mut planned: Vec<Planned> = element
        .attrs
        .iter()
        .enumerate()
        .map(|(index, attr)| Planned {
            name: attr.key(),
            parsed: Some(index),
            synthetic: None,
            span: attr.span,
        })
        .collect();

    if let Some(value) = synthesized_rel(element) {
        planned.push(Planned {
            name: String::from("rel"),
            parsed: None,
            synthetic: Some(value),
            span: element.span,
        });
    }
    if is_decorative_svg(element) {
        // Atributo explícito vence: não sintetizar o que já existe (senão o primeiro valor ganha).
        for (name, value) in [("aria-hidden", "true"), ("focusable", "false")]
            .into_iter()
            .filter(|(name, _)| !element.attrs.iter().any(|attr| attr.key() == *name))
        {
            planned.push(Planned {
                name: String::from(name),
                parsed: None,
                synthetic: Some(value),
                span: element.span,
            });
        }
    }

    let order = spec.map_or_else(Vec::new, effective_order);
    planned.sort_by_key(|item| sort_key(&order, &item.name));
    planned
}

/// `target="_blank"` sem `rel` ganha `rel="noopener noreferrer"`.
fn synthesized_rel(element: &Element) -> Option<&'static str> {
    if element.name != "a" {
        return None;
    }
    let target_blank = element.attrs.iter().any(|attr| {
        attr.key() == "target"
            && matches!(&attr.value, AttrValue::Literal { text, .. } if text == "_blank")
    });
    let has_rel = element.attrs.iter().any(|attr| attr.key() == "rel");
    if target_blank && !has_rel {
        Some("noopener noreferrer")
    } else {
        None
    }
}

/// SVG sem nome acessível é decorativo: `aria-hidden="true" focusable="false"`.
fn is_decorative_svg(element: &Element) -> bool {
    if element.name != "svg" {
        return false;
    }
    let has_name = element
        .attrs
        .iter()
        .any(|attr| matches!(attr.key().as_str(), "aria-label" | "aria-labelledby"));
    let has_title = element.body.as_ref().is_some_and(|body| {
        body.items
            .iter()
            .any(|item| matches!(item, Item::Element(inner) if inner.name == "svg_title"))
    });
    !has_name && !has_title
}

/// Ordem canônica: `order` (quando declarada) e depois `required ++ allowed ++ globals ++ aria-*`.
fn effective_order(spec: &'static table::Element) -> Vec<&'static str> {
    let mut order: Vec<&'static str> = Vec::new();
    let mut push = |name: &'static str| {
        if !order.contains(&name) {
            order.push(name);
        }
    };
    for name in spec.order {
        push(name);
    }
    for name in spec.required {
        push(name);
    }
    for name in spec.allowed {
        push(name);
    }
    for global in table::GLOBALS {
        push(global.name);
    }
    for aria in table::ARIA_VALUES {
        push(aria.name);
    }
    order
}

/// Posição do atributo na ordem canônica; curingas (`data-*`, `aria-*`) ordenam por nome no lugar
/// do curinga; nome desconhecido vai para o fim, ordenado.
fn sort_key(order: &[&'static str], name: &str) -> (usize, u8, String) {
    for (index, entry) in order.iter().enumerate() {
        if *entry == name {
            return (index, 0, name.to_owned());
        }
        if let Some(prefix) = entry.strip_suffix('*')
            && name.starts_with(prefix)
        {
            return (index, 1, name.to_owned());
        }
    }
    (usize::MAX, 2, name.to_owned())
}

fn estimate(block: &Block) -> usize {
    let mut total = 0usize;
    for item in &block.items {
        total = total.saturating_add(estimate_item(item));
    }
    total
}

fn estimate_item(item: &Item) -> usize {
    match item {
        Item::Element(element) => {
            let tag =
                table::element(&element.name).map_or(element.name.len(), |spec| spec.tag.len());
            let mut size = tag.saturating_mul(2).saturating_add(5);
            for attr in &element.attrs {
                let value = match &attr.value {
                    // Pior caso de escape em atributo: cada byte vira `&#xNN;` (6 bytes).
                    AttrValue::Literal { text, .. } => text.len().saturating_mul(6),
                    AttrValue::Computed { .. } => INTERPOLATION_BYTES,
                };
                size = size
                    .saturating_add(attr.name.len())
                    .saturating_add(5)
                    .saturating_add(value);
            }
            size.saturating_add(element.body.as_ref().map_or(0, estimate))
        }
        Item::Primitive(primitive) => {
            INTERPOLATION_BYTES.saturating_add(estimate(&primitive.children))
        }
        Item::Message(_) | Item::Child(_) => INTERPOLATION_BYTES,
        Item::If(node) => estimate_if(node),
        Item::For(node) => estimate(&node.body),
    }
}

fn estimate_if(node: &IfNode) -> usize {
    let mut size = estimate(&node.then);
    match &node.otherwise {
        None => {}
        Some(Otherwise::Block(block)) => size = size.saturating_add(estimate(block)),
        Some(Otherwise::If(inner)) => size = size.saturating_add(estimate_if(inner)),
    }
    size
}
