// TRECHO de crates/html-auy-macros-core/src/validate.rs (linhas 1-200 e 430-560 do original).
// O arquivo completo tem 1041 linhas; aqui ficam o despacho da validação, o fechamento de
// escopo (ids/referências/nome acessível) e as checagens de atributo.

//! Validação do bloco contra a tabela do spec.
//!
//! A maior parte das regras é **data-driven**: allowlist, obrigatórios, conteúdo-modelo, enums e
//! booleanos vêm de `spec/elements.toml`. O que exige olhar o bloco inteiro (símbolos de `id`,
//! referências acessíveis, estrutura de tabela e mídia) mora em [`crate::rules`].

use crate::ast::{Attr, AttrValue, Block, Element, IfNode, Item, Otherwise};
use crate::attrs::{is_boolean, is_typed};
use crate::diagnostic::Diag;
use crate::rules;
use crate::table::{self, Only, ValueKind};
use core::mem;
use proc_macro2::Span;

/// Atributos que nenhum elemento aceita (docs/03 §5, docs/08 §2).
const FORBIDDEN_ATTRS: [&str; 12] = [
    "srcdoc",
    "formaction",
    "ping",
    "slot",
    "part",
    "exportparts",
    "shadowrootmode",
    "is",
    "xlink:href",
    "accesskey",
    "draggable",
    "novalidate",
];

/// Categorias que são conteúdo de fluxo: um pai com `children = ["flow"]` as aceita todas
/// (os elementos declaram só a categoria mais específica; `phrasing` e `media` também são fluxo).
const FLOW_ABSORBS: [&str; 5] = ["sectioning", "heading", "form", "phrasing", "interactive"];

/// Atributos que todo controle de formulário aceita além dos do próprio `type` (E21).
const INPUT_BASE_ATTRS: [&str; 7] = [
    "autocomplete",
    "disabled",
    "name",
    "readonly",
    "required",
    "type",
    "value",
];

/// Controle de formulário que precisa de nome acessível resolvido no bloco (E20).
pub(crate) struct Control {
    /// Nome do elemento.
    pub(crate) name: String,
    /// Onde apontar.
    pub(crate) span: Span,
    /// `type` de isenção já reconhecida.
    pub(crate) exempt: bool,
    /// Nome já garantido (label envolvendo ou `aria-labelledby`).
    pub(crate) named: bool,
    /// `id` literal, quando houver.
    pub(crate) id: Option<String>,
}

/// Estado da validação de um bloco.
pub(crate) struct Ctx {
    /// Erros encontrados, na ordem do fonte.
    pub(crate) diags: Vec<Diag>,
    /// `id` literais do bloco (valor, span).
    pub(crate) ids: Vec<(String, Span)>,
    /// Referências literais a id (alvo, span, atributo).
    pub(crate) refs: Vec<(String, Span, String, String)>,
    /// Controles cujo nome acessível depende de outro elemento do bloco.
    pub(crate) controls: Vec<Control>,
}

impl Ctx {
    const fn new() -> Self {
        Self {
            diags: Vec::new(),
            ids: Vec::new(),
            refs: Vec::new(),
            controls: Vec::new(),
        }
    }

    /// Registra um erro.
    pub(crate) fn error(&mut self, diag: Diag) {
        self.diags.push(diag);
    }
}

/// Valida o bloco inteiro. A ordem é a de pré-ordem do fonte; os diagnósticos de escopo de bloco
/// (ids duplicados, referências órfãs, controles sem nome) fecham a lista.
pub(crate) fn check(block: &Block) -> Vec<Diag> {
    let mut ctx = Ctx::new();
    let mut ancestors: Vec<&'static table::Element> = Vec::new();
    check_block(block, &mut ancestors, &mut ctx);
    finish(ctx)
}

fn check_block(block: &Block, ancestors: &mut Vec<&'static table::Element>, ctx: &mut Ctx) {
    for item in &block.items {
        match item {
            Item::Element(element) => check_element(element, ancestors, ctx),
            Item::If(node) => check_if(node, ancestors, ctx),
            Item::For(node) => check_block(&node.body, ancestors, ctx),
            // Os filhos de um primitivo (`Form::open(…) { … }`) viram markup do bloco: passam
            // pela mesma validação, senão `Comp() { script { … } }` fura o vocabulário fechado.
            Item::Primitive(primitive) => check_block(&primitive.children, ancestors, ctx),
            Item::Message(_) | Item::Child(_) => {}
        }
    }
}

fn check_if(node: &IfNode, ancestors: &mut Vec<&'static table::Element>, ctx: &mut Ctx) {
    check_block(&node.then, ancestors, ctx);
    match &node.otherwise {
        None => {}
        Some(Otherwise::Block(block)) => check_block(block, ancestors, ctx),
        Some(Otherwise::If(inner)) => check_if(inner, ancestors, ctx),
    }
}

fn check_element(element: &Element, ancestors: &mut Vec<&'static table::Element>, ctx: &mut Ctx) {
    let Some(spec) = table::element(&element.name) else {
        ctx.error(unknown_element(element));
        return;
    };

    if let Some(only) = spec.only {
        ctx.error(document_element(element, spec, only));
        return;
    }

    check_parent(element, spec, ancestors, ctx);
    check_required(element, spec, ctx);
    check_allowed(element, spec, ctx);
    check_duplicates(element, ctx);
    for attr in &element.attrs {
        check_attr_value(element, spec, attr, ctx);
    }
    check_children(element, spec, ctx);
    check_forbidden_descendants(element, spec, ctx);

    ancestors.push(spec);
    if let Some(body) = &element.body {
        check_block(body, ancestors, ctx);
    }
    ancestors.pop();

    rules::check(element, ancestors.as_slice(), ctx);
}

/// Fecha a validação do bloco: ids repetidos e referências órfãs (E14/E15/E16).
fn finish(ctx: Ctx) -> Vec<Diag> {
    let mut diags = ctx.diags;

    for (index, (id, span)) in ctx.ids.iter().enumerate() {
        let repeated = ctx.ids.iter().take(index).any(|(other, _)| other == id);
        if repeated {
            diags.push(
                Diag::new(
                    *span,
                    format!("`id=\"{id}\"` aparece mais de uma vez no bloco"),
                )
                .rule("escopo local (IDs únicos)")
                .hint("use ids distintos; o `audit` cobra o documento inteiro"),
            );
        }
    }

    for (target, span, attr, source) in &ctx.refs {
        if ctx.ids.iter().any(|(id, _)| id == target) {
            continue;
        }
        diags.push(orphan(*span, attr, target, source));
    }

    for control in &ctx.controls {
        if control.exempt || control.named {
            continue;
        }
        let labelled = control.id.as_ref().is_some_and(|id| {
            ctx.refs
                .iter()
                .any(|(target, _, attr, source)| attr == "for" && source == "label" && target == id)
        });
        if labelled {
            continue;
        }
        diags.push(
            Diag::new(
                control.span,
                format!("`{}` não tem nome acessível", control.name),
            )
            .rule("WCAG 4.1.2 / spec:control_has_accessible_name")
            .hint(control_hint(&control.name)),
        );
    }

    diags
}

fn orphan(span: Span, attr: &str, target: &str, source: &str) -> Diag {

// ───────────── … (linhas 201-429 omitidas: mensagens de erro e conteúdo-modelo) ─────────────

    names.join(", ")
}

fn check_required(element: &Element, spec: &'static table::Element, ctx: &mut Ctx) {
    for required in spec.required {
        if element.attr(required).is_some() {
            continue;
        }
        let (rule, hint) = requirement_hint(spec.name, required);
        ctx.error(
            Diag::new(element.span, format!("`{}` exige `{required}`", spec.name))
                .rule(rule)
                .hint(hint),
        );
    }
}

fn requirement_hint(element: &str, attr: &str) -> (String, String) {
    let hint = match (element, attr) {
        ("img", "alt") => String::from("`Alt::new(\"Descrição objetiva\")` ou `Alt::decorative()`"),
        ("img" | "source", "width" | "height") => {
            String::from("`WidthPx::new(720)` / `HeightPx::new(480)` (CLS 0)")
        }
        ("source", "sizes") => String::from("`Sizes::new(\"(min-width: 48em) 50vw, 100vw\")`"),
        ("a", "href") => String::from("`Href::internal(LUrl::…)` ou `Href::external(…)`"),
        ("iframe", "title") => String::from("nome acessível do frame (WCAG 4.1.2)"),
        ("input", "type" | "name") => {
            String::from("use `Field::text(FieldName::new(\"…\"), Msg::…, &state)`")
        }
        ("button", "type") => String::from("use `type=\"button\"` (nunca o submit implícito)"),
        ("label", "for") => {
            String::from("use `for=\"id\"` ou envolva o controle; melhor: `Field::text(…)`")
        }
        ("th", "scope") => String::from("`scope=\"col\"` em `thead`; `scope=\"row\"` no corpo"),
        ("time", "datetime") => String::from("`datetime=(Timestamp::new(…))` (ISO 8601)"),
        _ => format!("adicione `{attr}=(…)`"),
    };
    let rule = if element == "img" && attr == "alt" {
        String::from("WCAG 1.1.1 / spec:element.img")
    } else {
        format!("spec:element.{element}")
    };
    (rule, hint)
}

// ─────────────────────────────── atributos ───────────────────────────────

fn check_allowed(element: &Element, spec: &'static table::Element, ctx: &mut Ctx) {
    for attr in &element.attrs {
        let key = attr.key();
        if key == "role" {
            check_role(spec, attr, ctx);
        } else if is_forbidden(&key) {
            let hint = if key == "accesskey" {
                "atalhos de teclado não são portáveis entre navegador/leitor de tela; use o skip link e `Tab`"
            } else {
                "estilo é token e comportamento é asset declarado"
            };
            ctx.error(
                Diag::new(
                    attr.span,
                    format!("atributo proibido `{key}` em `{}`", spec.name),
                )
                .rule(format!("spec:element.{}", spec.name))
                .hint(hint),
            );
        } else if key.starts_with("data-") {
            if !valid_data_name(&key) {
                ctx.error(
                    Diag::new(attr.span, format!("`data` inválido: `{key}`"))
                        .rule("spec:data")
                        .hint("use `data-*` minúsculo com segmentos `[a-z0-9]`"),
                );
            }
        } else if !allowed_key(spec, &key) {
            let names = allowed_names(spec);
            let mut message = format!("atributo desconhecido `{key}` em `{}`", spec.name);
            if let Some(candidate) = nearest(&key, names.iter().copied()) {
                message.push_str("; você quis dizer `");
                message.push_str(candidate);
                message.push_str("`?");
            }
            ctx.error(Diag::new(attr.span, message).hint(format!(
                "atributos válidos em `{}`: {}",
                spec.name,
                names.join(", ")
            )));
        }
    }
}

fn check_role(spec: &'static table::Element, attr: &Attr, ctx: &mut Ctx) {
    if spec.roles.is_empty() {
        ctx.error(
            Diag::new(attr.span, format!("`role` não existe em `{}`", spec.name))
                .rule("D4")
                .hint("use o papel nativo do elemento"),
        );
        return;
    }
    match &attr.value {
        AttrValue::Literal { text, span } if spec.roles.contains(&text.as_str()) => {}
        AttrValue::Literal { text, span } => ctx.error(
            Diag::new(
                *span,
                format!("`role=\"{text}\"` não é válido em `{}`", spec.name),
            )
            .rule(format!("spec:element.{}", spec.name))
            .hint(format!("papéis aceitos: {}", spec.roles.join(", "))),
        ),
        AttrValue::Computed { span, .. } => ctx.error(
            Diag::new(*span, "`role` não aceita expressão; use um literal")
                .rule("D4")
                .hint("o papel é uma decisão de projeto, não um dado"),
        ),
    }
}

fn is_forbidden(key: &str) -> bool {
    FORBIDDEN_ATTRS.contains(&key) || key.starts_with("on")
}

fn allowed_key(spec: &'static table::Element, key: &str) -> bool {
    if spec.required.contains(&key) || spec.allowed.contains(&key) {
        return true;
    }
    if key.starts_with("data-") {
        return spec.allowed.contains(&"data-*");
    }
    // `aria-*` é allowlist POR ELEMENTO: os nomes aparecem em `GLOBALS` (para o tipo do valor),
    // mas o elemento só os aceita se declarar `aria-*` em `allowed`.
