//! Acessibilidade: descrição textual e contraste (ADR-0005).
//!
//! A descrição é o que um leitor de tela lê e o que um LLM recebe no lugar da imagem. Ela é
//! determinística e não depende de layout: mesma lista de nós, mesmas arestas.

use crate::diag::{Diagnostic, W007};
use crate::model::Chart;
use crate::theme::{MIN_STROKE_CONTRAST, MIN_TEXT_CONTRAST, contrast_ratio};

/// Descrição textual do grafo, em duas linhas (contagem + nós; arestas).
///
/// ```text
/// 3 nós, 3 arestas. Nós: API Gateway; Retry ×3; DLQ.
/// Arestas: API Gateway → Retry ×3 (5xx); Retry ×3 → DLQ (esgotado).
/// ```
pub fn describe(chart: &Chart) -> String {
    let nodes: Vec<&str> = chart.nodes.iter().map(|node| node.label.as_str()).collect();
    let edges: Vec<String> = chart
        .edges
        .iter()
        .map(|edge| {
            let from = chart.label_of(&edge.from);
            let to = chart.label_of(&edge.to);
            match &edge.label {
                Some(label) => format!("{from} → {to} ({label})"),
                None => format!("{from} → {to}"),
            }
        })
        .collect();
    let mut parts = vec![format!(
        "{}, {}.",
        counted(chart.nodes.len(), "nó", "nós"),
        counted(chart.edges.len(), "aresta", "arestas")
    )];
    if !nodes.is_empty() {
        parts.push(format!(" Nós: {}.", nodes.join("; ")));
    }
    if !edges.is_empty() {
        parts.push(format!("\nArestas: {}.", edges.join("; ")));
    }
    parts.join("")
}

fn counted(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

/// Diagnósticos `W007` para combinações abaixo do mínimo WCAG.
///
/// Regras executáveis (ADR-0005):
/// - texto `ink` sobre `fill` ≥ 4.5:1 — a paleta garante, o check confirma;
/// - contorno contra a superfície em que aparece ≥ 3:1 — sobre o `fill` quando o preenchimento
///   existe, sobre o canvas quando não existe (arestas, formas vazadas, grupos).
pub fn check_contrast(chart: &Chart) -> Vec<Diagnostic> {
    let theme = chart.theme;
    let mut diagnostics = Vec::new();
    for node in &chart.nodes {
        let fill = theme.hex(node.fill);
        let stroke = theme.hex(node.stroke);
        if let Some(fill) = fill {
            let ratio = contrast_ratio(theme.ink, fill);
            if ratio < MIN_TEXT_CONTRAST {
                diagnostics.push(
                    Diagnostic::new(
                        W007,
                        node.span,
                        format!(
                            "texto sobre `{}` tem contraste {ratio:.1}:1 (mínimo {MIN_TEXT_CONTRAST:.1}:1)",
                            node.fill.name()
                        ),
                    )
                    .with_hint("use um preenchimento mais claro ou outro tema"),
                );
            }
        }
        if let Some(stroke) = stroke {
            let surface = fill.unwrap_or(theme.canvas);
            let ratio = contrast_ratio(stroke, surface);
            if ratio < MIN_STROKE_CONTRAST {
                diagnostics.push(
                    Diagnostic::new(
                        W007,
                        node.span,
                        format!(
                            "contorno `{}` sobre `{}` tem contraste {ratio:.1}:1 (mínimo {MIN_STROKE_CONTRAST:.1}:1)",
                            node.stroke.name(),
                            if fill.is_some() { node.fill.name() } else { "canvas" }
                        ),
                    )
                    .with_hint("tokens de cor são acentos; para contraste garantido use `ink` ou `muted`"),
                );
            }
        }
    }
    for group in &chart.groups {
        if let Some(stroke) = theme.hex(group.stroke) {
            let ratio = contrast_ratio(stroke, theme.canvas);
            if ratio < MIN_STROKE_CONTRAST {
                diagnostics.push(
                    Diagnostic::new(
                        W007,
                        group.span,
                        format!(
                            "contorno `{}` sobre `canvas` tem contraste {ratio:.1}:1 (mínimo {MIN_STROKE_CONTRAST:.1}:1)",
                            group.stroke.name()
                        ),
                    )
                    .with_hint("tokens de cor são acentos; `ink` e `muted` são os garantidos"),
                );
            }
        }
    }
    for edge in &chart.edges {
        if let Some(stroke) = theme.hex(edge.stroke) {
            let ratio = contrast_ratio(stroke, theme.canvas);
            if ratio < MIN_STROKE_CONTRAST {
                diagnostics.push(
                    Diagnostic::new(
                        W007,
                        edge.span,
                        format!(
                            "aresta `{}` sobre `canvas` tem contraste {ratio:.1}:1 (mínimo {MIN_STROKE_CONTRAST:.1}:1)",
                            edge.stroke.name()
                        ),
                    )
                    .with_hint("tokens de cor são acentos; `ink` e `muted` são os garantidos"),
                );
            }
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::{check_contrast, describe};
    use crate::config::Config;
    use crate::model::Chart;
    use crate::parse::parse;
    use crate::resolve::resolve;

    fn chart_of(source: &str) -> Chart {
        let (document, _) = parse(source);
        let (chart, _) = resolve(&document, &Config::default());
        chart
    }

    #[test]
    fn describes_nodes_and_edges_with_labels() {
        let chart = chart_of(
            "chart v1 \"x\"\nnode api \"API Gateway\"\nnode retry \"Retry ×3\"\nedge api -> retry \"5xx\"\n",
        );
        assert_eq!(
            describe(&chart),
            "2 nós, 1 aresta. Nós: API Gateway; Retry ×3.\nArestas: API Gateway → Retry ×3 (5xx)."
        );
    }

    #[test]
    fn describes_empty_and_singular() {
        let chart = chart_of("chart v1 \"x\"\nnode a \"A\"\n");
        assert_eq!(describe(&chart), "1 nó, 0 arestas. Nós: A.");
        let chart = chart_of("chart v1 \"x\"\n");
        assert_eq!(describe(&chart), "0 nós, 0 arestas.");
    }

    #[test]
    fn colored_strokes_warn_but_ink_does_not() {
        let default_strokes = chart_of("chart v1 \"x\"\nnode a \"A\"\nedge a -> a\n");
        assert_eq!(check_contrast(&default_strokes).len(), 0);
        let colored =
            chart_of("chart v1 \"x\"\nnode a \"A\" stroke=blue\nedge a -> a stroke=blue\n");
        let codes: Vec<&str> = check_contrast(&colored)
            .iter()
            .map(|diagnostic| diagnostic.code.code)
            .collect();
        assert_eq!(codes, ["W007", "W007"]);
    }

    #[test]
    fn dark_theme_keeps_ink_contrast() {
        let chart = chart_of("chart v1 \"x\" theme=slate\nnode a \"A\" fill=blue stroke=ink\n");
        assert_eq!(check_contrast(&chart).len(), 0);
    }
}
