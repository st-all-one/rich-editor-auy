//! Orquestração dos estágios para `check` e mecânica do lote JSONL.
//!
//! Tudo aqui é **puro**: recebe fonte, configuração e fonte tipográfica; devolve diagnósticos.
//! I/O fica na CLI, o que mantém `check --batch` e os testes do domínio no mesmo caminho.

use crate::a11y;
use crate::config::Config;
use crate::diag::{self, Diagnostic, W002, W003};
use crate::model::Chart;
use crate::parse::{self, Document};
use crate::resolve::resolve;
use crate::span::Span;
use crate::text::Font;

/// Resultado de `check`: o que foi lido, o que foi resolvido e o que há para corrigir.
#[derive(Debug)]
pub struct CheckReport {
    /// Documento sintático (para `fmt` e para o relatório).
    pub document: Document,
    /// Modelo resolvido (para `render`/`diff`).
    pub chart: Chart,
    /// Diagnósticos ordenados por posição.
    pub diagnostics: Vec<Diagnostic>,
}

/// Analisa, resolve e audita um `.akrc`.
pub fn check(source: &str, config: &Config, font: &Font) -> CheckReport {
    let (document, mut diagnostics) = parse::parse(source);
    let (chart, resolved) = resolve(&document, config);
    diagnostics.extend(resolved);
    diagnostics.extend(isolated(&chart));
    diagnostics.extend(missing_glyphs(&chart, font));
    diagnostics.extend(a11y::check_contrast(&chart));
    diag::sort(&mut diagnostics);
    CheckReport {
        document,
        chart,
        diagnostics,
    }
}

/// `W002`: nó sem arestas num layout que espera grafo.
fn isolated(chart: &Chart) -> Vec<Diagnostic> {
    if !chart.layout.expects_graph() {
        return Vec::new();
    }
    chart
        .nodes
        .iter()
        .filter(|node| chart.degree(&node.id) == 0)
        .map(|node| {
            Diagnostic::new(
                W002,
                node.span,
                format!(
                    "nó `{}` sem arestas em `layout={}`",
                    node.id,
                    chart.layout.name()
                ),
            )
            .with_hint("conecte o nó ou use `layout=grid`, `layout=stack` ou `layout=free`")
        })
        .collect()
}

/// `W003`: codepoint sem glifo na fonte embutida.
fn missing_glyphs(chart: &Chart, font: &Font) -> Vec<Diagnostic> {
    let family = font.gauge(chart.font);
    let mut diagnostics = Vec::new();
    let mut audit = |text: &str, span: Span, what: &str| {
        for character in family.missing_glyphs(text) {
            diagnostics.push(
                Diagnostic::new(
                    W003,
                    span,
                    format!("o caractere `{character}` não tem glifo na fonte embutida ({what})"),
                )
                .with_hint("o texto será desenhado com o glifo de substituição (tofu)"),
            );
        }
    };
    if let Some(title) = &chart.title {
        audit(title, Span::line_start(1), "`chart`");
    }
    for group in &chart.groups {
        audit(&group.label, group.span, &group.id);
    }
    for node in &chart.nodes {
        audit(&node.label, node.span, &node.id);
    }
    for edge in &chart.edges {
        if let Some(label) = &edge.label {
            audit(label, edge.span, "rótulo de aresta");
        }
    }
    for note in &chart.notes {
        audit(&note.text, note.span, "nota");
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::config::Config;
    use crate::text::Font;

    fn codes(source: &str) -> Vec<&'static str> {
        let font = Font::load().expect("fonte");
        let report = check(source, &Config::default(), &font);
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.code)
            .collect()
    }

    #[test]
    fn clean_document_has_no_diagnostics() {
        assert_eq!(
            codes("chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n"),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn isolated_nodes_warn_only_in_graph_layouts() {
        assert_eq!(
            codes("chart v1 \"x\"\nnode a\nnode b\nedge a -> b\nnode solto\n"),
            ["W002"]
        );
        assert_eq!(
            codes("chart v1 \"x\" layout=grid\nnode a\n"),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn missing_glyphs_are_reported_per_element() {
        assert_eq!(
            codes("chart v1 \"x\" font=hand\nnode a \"🦀\"\n"),
            ["W002", "W003"]
        );
    }

    #[test]
    fn diagnostics_are_sorted_by_position() {
        assert_eq!(
            codes("chart v1 \"x\" font=hand\nnode a \"🦀\"\nnode b \"🦀\"\n"),
            ["W002", "W003", "W002", "W003"]
        );
    }

    #[test]
    fn contrast_rule_reaches_the_pipeline() {
        assert_eq!(
            codes("chart v1 \"x\"\nnode a \"A\" stroke=blue\nnode b \"B\"\nedge a -> b\n"),
            ["W007"]
        );
    }
}

/// Uma linha de um lote JSONL, já separada do texto.
///
/// O núcleo não conhece as chaves de cada verbo (`render` aceita `format`, `check` aceita
/// `docs`): ele entrega o objeto e a CLI valida com a mesma tabela de `--params`. `index` conta
/// apenas linhas de item (linhas vazias não interferem) e `number` é a linha física do arquivo.
#[derive(Debug)]
pub struct BatchLine {
    /// Posição do item no lote (0-based, sem contar linhas vazias).
    pub index: usize,
    /// Linha física do arquivo (1-based).
    pub number: usize,
    /// Objeto JSON da linha; `None` quando a linha não é um objeto JSON.
    pub value: Option<serde_json::Value>,
    /// Erro de JSONL (linha malformada, ou conteúdo que não é objeto).
    pub error: Option<String>,
}

/// Separa um lote JSONL em linhas de item, preservando a ordem e os erros por linha.
pub fn batch_lines(input: &str) -> Vec<BatchLine> {
    let mut lines = Vec::new();
    for (position, raw) in input.lines().enumerate() {
        let number = position.saturating_add(1);
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let index = lines.len();
        match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(value) if value.is_object() => lines.push(BatchLine {
                index,
                number,
                value: Some(value),
                error: None,
            }),
            Ok(_) => lines.push(BatchLine {
                index,
                number,
                value: None,
                error: Some(format!(
                    "linha {number}: o item do lote precisa ser um objeto JSON"
                )),
            }),
            Err(error) => lines.push(BatchLine {
                index,
                number,
                value: None,
                error: Some(format!("linha {number}: JSONL inválido: {error}")),
            }),
        }
    }
    lines
}
