//! Renderização HTML de LaTeX via `KaTeX`, com CSS embutido (auto-contido).
//!
//! Inspirado no processador original `_REF/processor/html/processor.html.ts`:
//! injeta o motor `KaTeX` e o CSS minificado (com fontes em *data URI*) no
//! fragmento, gerando uma saída auto-contida, acessível e sem rede.

use std::sync::OnceLock;

use katex::{KatexContext, Settings, render_to_string};

use crate::error::RenderError;
use crate::options::{CssMode, ErrorPolicy, MathStyle};

/// CSS minificado do `KaTeX`, embutido para saída auto-contida.
///
/// Contém as fontes matemáticas em *data URI*, de modo que o fragmento não
/// depende de rede nem de folhas de estilo externas.
pub const KATEX_CSS: &str = include_str!("../assets/katex.min.css");

/// Opções de renderização HTML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlOptions {
    /// Estilo tipográfico (bloco ou embutido).
    pub style: MathStyle,
    /// Política de erros de parsing.
    pub error_policy: ErrorPolicy,
    /// Classe CSS adicional aplicada ao container (apenas tokens válidos).
    pub container_class: Option<String>,
    /// Modo de injeção do CSS.
    pub css: CssMode,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            style: MathStyle::Display,
            error_policy: ErrorPolicy::Render,
            container_class: None,
            css: CssMode::Inline,
        }
    }
}

impl HtmlOptions {
    /// Cria opções padrão (bloco, erros renderizados, CSS embutido).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Define o estilo tipográfico.
    #[must_use]
    pub const fn with_style(mut self, style: MathStyle) -> Self {
        self.style = style;
        self
    }

    /// Define a política de erros.
    #[must_use]
    pub const fn with_error_policy(mut self, policy: ErrorPolicy) -> Self {
        self.error_policy = policy;
        self
    }

    /// Define a classe CSS adicional do container.
    #[must_use]
    pub fn with_container_class(mut self, class: impl Into<String>) -> Self {
        self.container_class = Some(class.into());
        self
    }

    /// Define o modo de injeção de CSS.
    #[must_use]
    pub const fn with_css(mut self, css: CssMode) -> Self {
        self.css = css;
        self
    }
}

/// Retorna o contexto global do `KaTeX`, inicializado uma única vez.
fn katex_context() -> &'static KatexContext {
    static CONTEXT: OnceLock<KatexContext> = OnceLock::new();
    CONTEXT.get_or_init(KatexContext::default)
}

/// Escapa um texto para uso seguro em um atributo HTML.
fn escape_attribute(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
    output
}

/// Verifica se um token é uma classe CSS segura (`[A-Za-z0-9_-]`).
fn is_valid_class_token(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

/// Sanitiza tokens de classe CSS, descartando qualquer token inválido.
fn sanitize_classes(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|token| is_valid_class_token(token))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Renderiza LaTeX como um fragmento HTML auto-contido via `KaTeX`.
///
/// O `aria_label` é a descrição verbal do cálculo (acessibilidade); deve ser
/// gerada pela camada de NLG (ver `calc-auy-output::to_verbal_a11y`).
///
/// # Errors
///
/// Retorna [`RenderError::Katex`] se o LaTeX for inválido e a política de erro
/// for [`ErrorPolicy::Throw`].
///
/// # Examples
///
/// ```
/// use calc_auy_render::{HtmlOptions, to_html};
///
/// let html = to_html(r"1 + 1", "um mais um", &HtmlOptions::default()).expect("render");
/// assert!(html.contains("katex"));
/// assert!(html.contains("aria-label=\"um mais um\""));
/// ```
pub fn to_html(
    latex: &str,
    aria_label: &str,
    options: &HtmlOptions,
) -> Result<String, RenderError> {
    crate::function_span("calc_auy_render.html");
    let settings = Settings {
        display_mode: options.style.is_display(),
        throw_on_error: matches!(options.error_policy, ErrorPolicy::Throw),
        ..Settings::default()
    };

    let rendered = render_to_string(katex_context(), latex, &settings)?;

    let mut class = String::from("calc-auy-result");
    if let Some(extra) = &options.container_class {
        let sanitized = sanitize_classes(extra);
        if !sanitized.is_empty() {
            class.push(' ');
            class.push_str(&sanitized);
        }
    }

    let label = escape_attribute(aria_label);
    let style = match options.css {
        CssMode::Inline => format!(
            "<style>{KATEX_CSS}\n.calc-auy-result {{ margin: 1em 0; overflow-x: auto; }}</style>"
        ),
        CssMode::External => String::new(),
    };

    let mut output = String::with_capacity(rendered.len().saturating_add(style.len()));
    output.push_str("<div class=\"");
    output.push_str(&class);
    output.push_str("\" aria-label=\"");
    output.push_str(&label);
    output.push_str("\">");
    output.push_str(&style);
    output.push_str(&rendered);
    output.push_str("</div>");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_self_contained_fragment() {
        let html = to_html(
            r"x = \frac{1}{2}",
            "x igual a um meio",
            &HtmlOptions::default(),
        )
        .expect("render");
        assert!(html.starts_with("<div class=\"calc-auy-result\""));
        assert!(html.contains("<style>"));
        assert!(html.contains("aria-label=\"x igual a um meio\""));
        assert!(html.ends_with("</div>"));
    }

    #[test]
    fn escapes_aria_label() {
        let html = to_html(r"1 + 1", "a \"b\" <c>", &HtmlOptions::default()).expect("render");
        assert!(html.contains("aria-label=\"a &quot;b&quot; &lt;c&gt;\""));
    }

    #[test]
    fn sanitizes_container_class() {
        assert_eq!(sanitize_classes("report card"), "report card");
        assert_eq!(sanitize_classes("x\" onmouseover=\"evil"), "");
        assert_eq!(sanitize_classes("<script>"), "");
    }

    #[test]
    fn omits_css_when_external() {
        let options = HtmlOptions::default().with_css(CssMode::External);
        let html = to_html("x", "", &options).expect("render");
        assert!(!html.contains("<style>"));
    }

    #[test]
    fn inline_style_renders() {
        let options = HtmlOptions::default().with_style(MathStyle::Text);
        let html = to_html("x", "", &options).expect("render");
        assert!(html.contains("katex"));
    }
}
