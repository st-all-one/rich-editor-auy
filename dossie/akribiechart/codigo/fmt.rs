//! Forma canônica do `.akrc` (ADR-0002 §3.4).
//!
//! Existe **uma** escrita para cada documento. O que define a forma:
//!
//! - classes na ordem `chart`, `group`, `node`, `edge`, `note`, com a ordem de declaração preservada
//!   dentro de cada classe;
//! - exatamente uma linha em branco entre classes não vazias, nenhuma dentro de uma classe;
//! - comentários colados ao statement seguinte (e, sem statement seguinte, no fim do arquivo);
//! - um espaço como separador, sem alinhamento de colunas;
//! - strings sempre entre aspas, ids nunca;
//! - opções em ordem alfabética de chave;
//! - números com até duas casas, sem zeros à direita (`num::format`);
//! - arquivo terminado por exatamente um `\n` (documento vazio ⇒ saída vazia).

use crate::parse::Document;
use crate::syntax::{ChartStmt, EdgeStmt, GroupStmt, NodeStmt, NoteStmt, Options, escape_string};

/// Formata o documento na forma canônica.
pub fn format(document: &Document) -> String {
    let mut blocks: Vec<String> = Vec::new();
    if let Some(chart) = &document.chart {
        blocks.push(block(&chart.comments, &chart_line(chart)));
    }
    if !document.groups.is_empty() {
        blocks.push(class(&document.groups));
    }
    if !document.nodes.is_empty() {
        blocks.push(class(&document.nodes));
    }
    if !document.edges.is_empty() {
        blocks.push(class(&document.edges));
    }
    if !document.notes.is_empty() {
        blocks.push(class(&document.notes));
    }

    let mut out = blocks.join("\n");
    if !document.trailing_comments.is_empty() {
        let trailing = comments_block(&document.trailing_comments);
        if out.is_empty() {
            return trailing;
        }
        out.push('\n');
        out.push_str(&trailing);
    }
    out
}

/// Uma classe de statements, com os comentários de cada um.
fn class<T: Canonical>(items: &[T]) -> String {
    let mut out = String::new();
    for item in items {
        out.push_str(&block(item.comments(), &item.line()));
    }
    out
}

/// Comentários + linha, cada um terminado por `\n`.
fn block(comments: &[String], line: &str) -> String {
    let mut out = comments_block(comments);
    out.push_str(line);
    out.push('\n');
    out
}

fn comments_block(comments: &[String]) -> String {
    let mut out = String::new();
    for comment in comments {
        out.push_str(comment);
        out.push('\n');
    }
    out
}

/// Opções em ordem alfabética, separadas por um espaço, prefixadas por `key=`.
fn options_text(options: &Options) -> String {
    let mut keys: Vec<&str> = options.keys().copied().collect();
    keys.sort_unstable();
    let mut out = String::new();
    for key in keys {
        let Some(value) = options.get(key) else {
            continue;
        };
        out.push(' ');
        out.push_str(key);
        out.push('=');
        out.push_str(&value.render());
    }
    out
}

fn quoted(text: &str) -> String {
    format!("\"{}\"", escape_string(text))
}

fn chart_line(chart: &ChartStmt) -> String {
    let mut out = String::from("chart v1");
    if let Some(title) = &chart.title {
        out.push(' ');
        out.push_str(&quoted(title));
    }
    out.push_str(&options_text(&chart.options));
    out
}

/// Contrato comum das classes que têm comentários próprios e uma linha canônica.
trait Canonical {
    fn comments(&self) -> &[String];
    fn line(&self) -> String;
}

impl Canonical for GroupStmt {
    fn comments(&self) -> &[String] {
        &self.comments
    }

    fn line(&self) -> String {
        let mut out = format!("group {}", self.id);
        if let Some(label) = &self.label {
            out.push(' ');
            out.push_str(&quoted(label));
        }
        out.push_str(&options_text(&self.options));
        out
    }
}

impl Canonical for NodeStmt {
    fn comments(&self) -> &[String] {
        &self.comments
    }

    fn line(&self) -> String {
        let mut out = format!("node {}", self.id);
        if let Some(label) = &self.label {
            out.push(' ');
            out.push_str(&quoted(label));
        }
        out.push_str(&options_text(&self.options));
        out
    }
}

impl Canonical for EdgeStmt {
    fn comments(&self) -> &[String] {
        &self.comments
    }

    fn line(&self) -> String {
        let mut out = format!("edge {} -> {}", self.from, self.to);
        if let Some(label) = &self.label {
            out.push(' ');
            out.push_str(&quoted(label));
        }
        out.push_str(&options_text(&self.options));
        out
    }
}

impl Canonical for NoteStmt {
    fn comments(&self) -> &[String] {
        &self.comments
    }

    fn line(&self) -> String {
        let mut out = format!("note {}", quoted(&self.text));
        out.push_str(&options_text(&self.options));
        out
    }
}
