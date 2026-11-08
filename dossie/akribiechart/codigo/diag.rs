//! Diagnósticos de schema.
//!
//! Um diagnóstico é **dado**, não log: `check` o imprime em `stdout` e o devolve em `--json`. Todo
//! código é estável, nomeado e documentado aqui — a tabela [`ALL`] alimenta `akr schema` e
//! `akr prime`, então código novo entra na tabela, no teste de paridade e na documentação.

use crate::span::Span;

/// Gravidade de um diagnóstico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Impede a compilação (exit 8).
    Error,
    /// Aceitável; `--strict` promove a erro.
    Warn,
}

impl Severity {
    /// Identificador textual usado em `--json` e em `akr schema`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
        }
    }
}

/// Código estável de diagnóstico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Code {
    /// `E001`…`W007`.
    pub code: &'static str,
    /// Nome curto, usado na mensagem (`E003 invalid-value: …`).
    pub name: &'static str,
    /// Gravidade intrínseca do código.
    pub severity: Severity,
}

const fn err(code: &'static str, name: &'static str) -> Code {
    Code {
        code,
        name,
        severity: Severity::Error,
    }
}

const fn warn(code: &'static str, name: &'static str) -> Code {
    Code {
        code,
        name,
        severity: Severity::Warn,
    }
}

/// Keyword desconhecida (`chart`, `group`, `node`, `edge`, `note`).
pub const E001: Code = err("E001", "unknown-keyword");
/// Chave desconhecida no statement.
pub const E002: Code = err("E002", "unknown-key");
/// Valor fora do conjunto fechado, tipo inválido ou fora da faixa.
pub const E003: Code = err("E003", "invalid-value");
/// Id duplicado entre `group`, `node` e `note`.
pub const E004: Code = err("E004", "duplicate-id");
/// Referência ausente (`in=`, `near=`, `edge` para grupo/nota).
pub const E005: Code = err("E005", "missing-reference");
/// `layout=free` sem `at`.
pub const E006: Code = err("E006", "missing-position");
/// Linha malformada (tab, aspas não fechadas, `->` ausente).
pub const E007: Code = err("E007", "malformed-line");
/// A primeira afirmação não é `chart`.
pub const E008: Code = err("E008", "missing-chart");
/// Versão do documento diferente de `v1`.
pub const E009: Code = err("E009", "unsupported-version");
/// Nó implícito criado por uma aresta.
pub const W001: Code = warn("W001", "implicit-node");
/// Nó sem arestas em layout que espera grafo.
pub const W002: Code = warn("W002", "isolated-node");
/// Codepoint sem glifo na fonte embutida.
pub const W003: Code = warn("W003", "missing-glyph");
/// `doc=` ausente (arquivo ou âncora), só com `--docs`.
pub const W004: Code = warn("W004", "doc-missing");
/// Cor do `.excalidraw` sem token equivalente.
pub const W005: Code = warn("W005", "color-unmapped");
/// Elemento não representável descartado na importação.
pub const W006: Code = warn("W006", "element-dropped");
/// Contraste abaixo do mínimo WCAG.
pub const W007: Code = warn("W007", "low-contrast");

/// Todos os códigos, na ordem em que aparecem na documentação.
pub const ALL: [Code; 16] = [
    E001, E002, E003, E004, E005, E006, E007, E008, E009, W001, W002, W003, W004, W005, W006, W007,
];

/// Um diagnóstico posicionado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Código estável.
    pub code: Code,
    /// Posição no arquivo.
    pub span: Span,
    /// O que se esperava e por quê.
    pub message: String,
    /// Como corrigir (impresso como `hint:`).
    pub hint: Option<String>,
}

impl Diagnostic {
    /// Cria um diagnóstico com a gravidade do próprio código.
    pub fn new(code: Code, span: Span, message: impl Into<String>) -> Self {
        Self {
            code,
            span,
            message: message.into(),
            hint: None,
        }
    }

    /// Acrescenta a correção sugerida.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Acrescenta a correção apenas quando houver uma.
    pub fn with_optional_hint(mut self, hint: Option<String>) -> Self {
        self.hint = hint;
        self
    }

    /// `true` quando o diagnóstico é um erro (impede a compilação).
    pub const fn is_error(&self) -> bool {
        matches!(self.code.severity, Severity::Error)
    }

    /// Linha de texto: `<file>:<line>:<col>: <CODE> <name>: <message>` (+ `hint:`).
    pub fn render(&self, file: &str) -> String {
        let head = format!(
            "{file}:{}:{}: {} {}: {}",
            self.span.line, self.span.col, self.code.code, self.code.name, self.message
        );
        match &self.hint {
            Some(hint) => format!("{head}\n  hint: {hint}"),
            None => head,
        }
    }

    /// Forma JSON de `--json`: `{code, severity, line, col, message, hint?}`.
    pub fn to_json(&self) -> serde_json::Value {
        let mut value = serde_json::json!({
            "code": self.code.code,
            "severity": self.code.severity.as_str(),
            "line": self.span.line,
            "col": self.span.col,
            "message": self.message,
        });
        if let (Some(hint), Some(map)) = (&self.hint, value.as_object_mut()) {
            map.insert("hint".to_string(), serde_json::Value::String(hint.clone()));
        }
        value
    }
}

/// `true` se algum diagnóstico da lista é erro.
pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(Diagnostic::is_error)
}

/// Ordena por posição (linha, coluna) e, no empate, por código — determinístico por construção.
pub fn sort(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|a, b| {
        (a.span.line, a.span.col, a.code.code).cmp(&(b.span.line, b.span.col, b.code.code))
    });
}
