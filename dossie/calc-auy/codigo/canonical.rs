//! Canonicalização determinística v1.
//!
//! A forma canônica é um texto JSON sem espaços, com **chaves ordenadas por
//! bytes UTF-8** e sem `f64` (garantido pelo tipo [`MetaValue`] e pela
//! ausência de `Canonical` para floats). O trait [`Canonical`] torna a
//! canonicalização uma propriedade do tipo, derivável com
//! `#[derive(calc_auy_macros::Canonical)]`.

use std::collections::BTreeMap;

use crate::ast::{FlatAst, Metadata, NodeData, NodeId};
use crate::meta_value::MetaValue;
use crate::numeric::{Rational, RoundingStrategy};

/// Escreve a forma canônica v1 de um valor em `out`.
pub trait Canonical {
    /// Escreve os bytes canônicos do valor.
    fn write_canonical(&self, out: &mut Vec<u8>);
}

/// Escreve uma string JSON canônica (escape mínimo).
pub fn write_json_string(out: &mut Vec<u8>, value: &str) {
    out.push(b'"');
    for ch in value.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\u{08}' => out.extend_from_slice(b"\\b"),
            '\u{0c}' => out.extend_from_slice(b"\\f"),
            control if u32::from(control) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", u32::from(control)).as_bytes());
            }
            other => {
                let mut buffer = [0_u8; 4];
                out.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    out.push(b'"');
}

/// Escreve uma chave de objeto canônica (`"chave":`).
pub fn write_key(out: &mut Vec<u8>, key: &str) {
    write_json_string(out, key);
    out.push(b':');
}

/// Modo de canonicalização: com ou sem metadados.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalMode {
    /// Inclui metadados (`trace_hash`).
    Trace,
    /// Ignora metadados (`value_hash`).
    Value,
}

impl CanonicalMode {
    /// Retorna `true` se os metadados devem ser incluídos.
    #[must_use]
    pub const fn includes_metadata(self) -> bool {
        matches!(self, Self::Trace)
    }
}

/// Escreve uma vírgula de separação, exceto antes do primeiro item.
fn comma_if_needed(out: &mut Vec<u8>, index: usize) {
    if index > 0 {
        out.push(b',');
    }
}

/// Escreve um inteiro decimal canônico.
fn write_integer(out: &mut Vec<u8>, value: &dyn std::fmt::Display) {
    out.extend_from_slice(value.to_string().as_bytes());
}

impl Canonical for () {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(b"null");
    }
}

impl Canonical for bool {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(if *self { b"true" } else { b"false" });
    }
}

impl Canonical for i64 {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_integer(out, self);
    }
}

impl Canonical for u64 {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_integer(out, self);
    }
}

impl Canonical for u32 {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_integer(out, self);
    }
}

impl Canonical for u16 {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_integer(out, self);
    }
}

impl Canonical for usize {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_integer(out, self);
    }
}

impl Canonical for str {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_json_string(out, self);
    }
}

impl Canonical for String {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_json_string(out, self);
    }
}

impl Canonical for Box<str> {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_json_string(out, self);
    }
}

impl<T: Canonical> Canonical for Option<T> {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        match self {
            Some(value) => value.write_canonical(out),
            None => out.extend_from_slice(b"null"),
        }
    }
}

impl<T: Canonical> Canonical for Vec<T> {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        out.push(b'[');
        for (index, item) in self.iter().enumerate() {
            comma_if_needed(out, index);
            item.write_canonical(out);
        }
        out.push(b']');
    }
}

impl<T: Canonical> Canonical for [T] {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        out.push(b'[');
        for (index, item) in self.iter().enumerate() {
            comma_if_needed(out, index);
            item.write_canonical(out);
        }
        out.push(b']');
    }
}

impl<K: AsRef<str>, V: Canonical> Canonical for BTreeMap<K, V> {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        out.push(b'{');
        for (index, (key, value)) in self.iter().enumerate() {
            comma_if_needed(out, index);
            write_key(out, key.as_ref());
            value.write_canonical(out);
        }
        out.push(b'}');
    }
}

impl Canonical for Rational {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(b"{\"d\":");
        write_json_string(out, &self.denominator());
        out.extend_from_slice(b",\"n\":");
        write_json_string(out, &self.numerator());
        out.push(b'}');
    }
}

impl Canonical for RoundingStrategy {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_json_string(out, self.as_str());
    }
}

impl Canonical for crate::ast::OperationType {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        write_json_string(out, self.as_str());
    }
}

impl Canonical for MetaValue {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        match self {
            Self::Null => out.extend_from_slice(b"null"),
            Self::Bool(value) => value.write_canonical(out),
            Self::Int(value) => value.write_canonical(out),
            Self::Uint(value) => value.write_canonical(out),
            Self::Str(value) => write_json_string(out, value),
            Self::Array(values) => values.write_canonical(out),
            Self::Object(map) => map.write_canonical(out),
        }
    }
}

impl FlatAst {
    /// Escreve a forma canônica **com** metadados (`trace_hash`).
    pub fn write_canonical_trace(&self, out: &mut Vec<u8>) {
        self.write_node_canonical(self.root(), out, CanonicalMode::Trace);
    }

    /// Escreve a forma canônica **sem** metadados (`value_hash`).
    pub fn write_canonical_value(&self, out: &mut Vec<u8>) {
        self.write_node_canonical(self.root(), out, CanonicalMode::Value);
    }

    /// Escreve um nó canonicamente, com chaves ordenadas.
    fn write_node_canonical(&self, id: NodeId, out: &mut Vec<u8>, mode: CanonicalMode) {
        let metadata = self.metadata(id).filter(|map| !map.is_empty());
        match self.node(id) {
            Some(NodeData::Literal {
                value,
                original_input,
            }) => {
                out.extend_from_slice(b"{\"kind\":\"literal\"");
                write_metadata(out, metadata, mode);
                out.extend_from_slice(b",\"original_input\":");
                write_json_string(out, original_input);
                out.extend_from_slice(b",\"value\":");
                value.write_canonical(out);
                out.push(b'}');
            }
            Some(NodeData::Operation { op, operands }) => {
                out.extend_from_slice(b"{\"kind\":\"operation\"");
                write_metadata(out, metadata, mode);
                out.extend_from_slice(b",\"op_type\":");
                op.write_canonical(out);
                out.extend_from_slice(b",\"operands\":");
                write_operands_canonical(self, operands, out, mode);
                out.push(b'}');
            }
            Some(NodeData::Group { child }) => {
                out.extend_from_slice(b"{\"child\":");
                self.write_node_canonical(*child, out, mode);
                out.extend_from_slice(b",\"kind\":\"group\"");
                write_metadata(out, metadata, mode);
                out.push(b'}');
            }
            Some(NodeData::Control {
                child,
                previous_context_label,
                previous_signature,
                previous_round_strategy,
            }) => {
                out.extend_from_slice(b"{\"child\":");
                self.write_node_canonical(*child, out, mode);
                out.extend_from_slice(b",\"kind\":\"control\"");
                write_metadata(out, metadata, mode);
                out.extend_from_slice(b",\"previous_context_label\":");
                write_json_string(out, previous_context_label);
                out.extend_from_slice(b",\"previous_round_strategy\":");
                previous_round_strategy.write_canonical(out);
                out.extend_from_slice(b",\"previous_signature\":");
                write_json_string(out, previous_signature);
                out.push(b'}');
            }
            None => out.extend_from_slice(b"null"),
        }
    }
}

/// Escreve a lista canônica de operandos.
fn write_operands_canonical(
    ast: &FlatAst,
    operands: &[NodeId],
    out: &mut Vec<u8>,
    mode: CanonicalMode,
) {
    out.push(b'[');
    for (index, operand) in operands.iter().enumerate() {
        comma_if_needed(out, index);
        ast.write_node_canonical(*operand, out, mode);
    }
    out.push(b']');
}

/// Escreve o campo `metadata`, se presente e permitido pelo modo.
fn write_metadata(out: &mut Vec<u8>, metadata: Option<&Metadata>, mode: CanonicalMode) {
    if let Some(metadata) = metadata.filter(|_| mode.includes_metadata()) {
        out.extend_from_slice(b",\"metadata\":");
        metadata.write_canonical(out);
    }
}

impl Canonical for FlatAst {
    fn write_canonical(&self, out: &mut Vec<u8>) {
        self.write_canonical_trace(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstBuilder, OperationType};
    use crate::limits::Limits;

    fn sample_ast() -> FlatAst {
        let limits = Limits::default();
        let mut builder = AstBuilder::new(&limits);
        let one = builder
            .literal(Rational::from_i64(1), "1")
            .expect("literal");
        let two = builder
            .literal(Rational::from_i64(2), "2")
            .expect("literal");
        let sum = builder
            .operation(OperationType::Add, &[one, two])
            .expect("soma");
        builder.finish(sum).expect("árvore")
    }

    #[test]
    fn meta_value_golden() {
        let mut out = Vec::new();
        MetaValue::Object(BTreeMap::from([
            ("b".into(), MetaValue::Uint(2)),
            ("a".into(), MetaValue::Str("x\"y".into())),
        ]))
        .write_canonical(&mut out);
        assert_eq!(
            String::from_utf8(out).expect("utf8"),
            r#"{"a":"x\"y","b":2}"#
        );
    }

    #[test]
    fn rational_golden_keys_sorted() {
        let mut out = Vec::new();
        Rational::new_checked(3, 4, &Limits::default())
            .expect("racional")
            .write_canonical(&mut out);
        assert_eq!(
            String::from_utf8(out).expect("utf8"),
            r#"{"d":"4","n":"3"}"#
        );
    }

    #[test]
    fn ast_canonical_is_deterministic() {
        let ast = sample_ast();
        let mut first = Vec::new();
        let mut second = Vec::new();
        ast.write_canonical_trace(&mut first);
        ast.write_canonical_trace(&mut second);
        assert_eq!(first, second);
        let text = String::from_utf8(first).expect("utf8");
        assert!(text.starts_with(r#"{"kind":"operation","op_type":"add","operands":["#));
    }

    #[test]
    fn value_hash_ignores_metadata() {
        let limits = Limits::default();
        let mut builder = AstBuilder::new(&limits);
        let one = builder
            .literal(Rational::from_i64(1), "1")
            .expect("literal");
        let two = builder
            .literal(Rational::from_i64(2), "2")
            .expect("literal");
        let sum = builder
            .operation(OperationType::Add, &[one, two])
            .expect("soma");
        builder
            .insert_metadata(one, "ref", MetaValue::Str("SELIC".into()))
            .expect("metadata");
        let ast = builder.finish(sum).expect("árvore");

        let mut trace = Vec::new();
        let mut value = Vec::new();
        ast.write_canonical_trace(&mut trace);
        ast.write_canonical_value(&mut value);
        assert_ne!(trace, value);
        assert!(!String::from_utf8(value).expect("utf8").contains("SELIC"));
    }
}
