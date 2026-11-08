//! Projeção *wire* recursiva e desserialização com limites em *streaming*.
//!
//! A arena é serializada como a forma recursiva do ADR-002 (`kind` primeiro).
//! A desserialização usa [`DeserializeSeed`] para impor os [`Limits`] **durante**
//! a leitura, abortando cedo payloads hostis (profundidade, nós, operandos e
//! bytes de metadata).

use std::fmt;

use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};

use super::builder::AstBuilder;
use super::flat::{FlatAst, NodeData};
use super::types::{Metadata, NodeId, OperationType};
use crate::error::{CalcAuyError, LimitKind};
use crate::limits::Limits;
use crate::numeric::{Rational, RoundingStrategy};

/// Campos aceitos por um nó `literal`.
const LITERAL_FIELDS: &[&str] = &["value", "original_input", "metadata"];
/// Campos aceitos por um nó `operation`.
const OPERATION_FIELDS: &[&str] = &["op_type", "operands", "metadata"];
/// Campos aceitos por um nó `group`.
const GROUP_FIELDS: &[&str] = &["child", "metadata"];
/// Campos aceitos por um nó `control`.
const CONTROL_FIELDS: &[&str] = &[
    "child",
    "previous_context_label",
    "previous_signature",
    "previous_round_strategy",
    "metadata",
];

/// Representação *wire* de um racional (`n`/`d` como strings decimais).
#[derive(Debug, Deserialize)]
struct RationalWire {
    /// Numerador decimal.
    n: String,
    /// Denominador decimal.
    d: String,
}

/// Serializa a projeção recursiva de um nó.
struct NodeRef<'a> {
    /// Arena de origem.
    ast: &'a FlatAst,
    /// Nó projetado.
    id: NodeId,
}

impl Serialize for NodeRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.ast.serialize_node(self.id, serializer)
    }
}

/// Serializa a lista de operandos de uma operação.
struct OperandsRef<'a> {
    /// Arena de origem.
    ast: &'a FlatAst,
    /// Operandos.
    ids: &'a [NodeId],
}

impl Serialize for OperandsRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.ids.len()))?;
        for id in self.ids {
            seq.serialize_element(&NodeRef {
                ast: self.ast,
                id: *id,
            })?;
        }
        seq.end()
    }
}

impl Serialize for FlatAst {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.serialize_node(self.root(), serializer)
    }
}

impl FlatAst {
    /// Projeta um nó (e seus descendentes) na forma *wire* recursiva.
    ///
    /// # Errors
    ///
    /// Retorna erro do serializador se o `NodeId` for inválido ou a saída
    /// não puder ser escrita.
    fn serialize_node<S: Serializer>(&self, id: NodeId, serializer: S) -> Result<S::Ok, S::Error> {
        let node = self
            .node(id)
            .ok_or_else(|| serde::ser::Error::custom(format!("NodeId inválido: {}", id.get())))?;
        let metadata = self.metadata(id);

        let mut map = serializer.serialize_map(None)?;
        match node {
            NodeData::Literal {
                value,
                original_input,
            } => {
                map.serialize_entry("kind", "literal")?;
                map.serialize_entry("value", value)?;
                map.serialize_entry("original_input", original_input.as_ref())?;
            }
            NodeData::Operation { op, operands } => {
                map.serialize_entry("kind", "operation")?;
                map.serialize_entry("op_type", op)?;
                map.serialize_entry(
                    "operands",
                    &OperandsRef {
                        ast: self,
                        ids: operands,
                    },
                )?;
            }
            NodeData::Group { child } => {
                map.serialize_entry("kind", "group")?;
                map.serialize_entry(
                    "child",
                    &NodeRef {
                        ast: self,
                        id: *child,
                    },
                )?;
            }
            NodeData::Control {
                child,
                previous_context_label,
                previous_signature,
                previous_round_strategy,
            } => {
                map.serialize_entry("kind", "control")?;
                map.serialize_entry(
                    "child",
                    &NodeRef {
                        ast: self,
                        id: *child,
                    },
                )?;
                map.serialize_entry("previous_context_label", previous_context_label.as_ref())?;
                map.serialize_entry("previous_signature", previous_signature.as_ref())?;
                map.serialize_entry("previous_round_strategy", previous_round_strategy)?;
            }
        }
        if let Some(metadata) = metadata.filter(|map| !map.is_empty()) {
            map.serialize_entry("metadata", metadata)?;
        }
        map.end()
    }

    /// Serializa a AST para uma string JSON canônica.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::SerializationError`] se a serialização falhar.
    pub fn to_json_string(&self) -> Result<String, CalcAuyError> {
        serde_json::to_string(self).map_err(|error| CalcAuyError::SerializationError {
            message: error.to_string(),
        })
    }

    /// Desserializa de uma string JSON impondo os limites fornecidos.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::SerializationError`] se o payload for inválido
    /// ou exceder qualquer limite.
    pub fn from_json_str(input: &str, limits: &Limits) -> Result<Self, CalcAuyError> {
        let mut deserializer = serde_json::Deserializer::from_str(input);
        let ast = AstSeed { limits }
            .deserialize(&mut deserializer)
            .map_err(|error| CalcAuyError::SerializationError {
                message: error.to_string(),
            })?;
        deserializer
            .end()
            .map_err(|error| CalcAuyError::SerializationError {
                message: error.to_string(),
            })?;
        Ok(ast)
    }

    /// Desserializa de um *slice* JSON impondo os limites fornecidos.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::SerializationError`] se o payload for inválido
    /// ou exceder qualquer limite.
    pub fn from_json_slice(input: &[u8], limits: &Limits) -> Result<Self, CalcAuyError> {
        let mut deserializer = serde_json::Deserializer::from_slice(input);
        let ast = AstSeed { limits }
            .deserialize(&mut deserializer)
            .map_err(|error| CalcAuyError::SerializationError {
                message: error.to_string(),
            })?;
        deserializer
            .end()
            .map_err(|error| CalcAuyError::SerializationError {
                message: error.to_string(),
            })?;
        Ok(ast)
    }
}

impl<'de> Deserialize<'de> for FlatAst {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let limits = Limits::default();
        AstSeed { limits: &limits }.deserialize(deserializer)
    }
}

/// Semente de desserialização de uma AST completa.
struct AstSeed<'limits> {
    /// Limites aplicados.
    limits: &'limits Limits,
}

impl<'de> DeserializeSeed<'de> for AstSeed<'_> {
    type Value = FlatAst;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<FlatAst, D::Error> {
        let mut builder = AstBuilder::new(self.limits);
        let root = NodeSeed {
            limits: self.limits,
            builder: &mut builder,
            depth: 1,
        }
        .deserialize(deserializer)?;
        builder.finish(root).map_err(de::Error::custom)
    }
}

/// Semente de desserialização de um nó.
struct NodeSeed<'a, 'limits> {
    /// Limites aplicados.
    limits: &'limits Limits,
    /// Construtor compartilhado.
    builder: &'a mut AstBuilder<'limits>,
    /// Profundidade corrente (raiz = 1).
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for NodeSeed<'_, '_> {
    type Value = NodeId;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<NodeId, D::Error> {
        deserializer.deserialize_map(NodeVisitor {
            limits: self.limits,
            builder: self.builder,
            depth: self.depth,
        })
    }
}

/// Visitante de um nó da AST.
struct NodeVisitor<'a, 'limits> {
    /// Limites aplicados.
    limits: &'limits Limits,
    /// Construtor compartilhado.
    builder: &'a mut AstBuilder<'limits>,
    /// Profundidade corrente (raiz = 1).
    depth: usize,
}

impl NodeVisitor<'_, '_> {
    /// Rejeita profundidades acima do limite antes de recursar.
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se a profundidade exceder o limite.
    fn check_depth<E: de::Error>(&self) -> Result<(), E> {
        if self.depth > self.limits.max_depth {
            return Err(E::custom(CalcAuyError::LimitExceeded {
                kind: LimitKind::MaxDepth,
                limit: u64::try_from(self.limits.max_depth).unwrap_or(u64::MAX),
                actual: u64::try_from(self.depth).unwrap_or(u64::MAX),
            }));
        }
        Ok(())
    }

    /// Lê os operandos de uma operação (sequência de nós).
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se a sequência for inválida.
    fn read_operands<'de, A: MapAccess<'de>>(
        &mut self,
        map: &mut A,
    ) -> Result<Vec<NodeId>, A::Error> {
        map.next_value_seed(SeqSeed {
            limits: self.limits,
            builder: self.builder,
            depth: self.depth.saturating_add(1),
        })
    }

    /// Lê um nó filho único.
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se o nó for inválido.
    fn read_child<'de, A: MapAccess<'de>>(&mut self, map: &mut A) -> Result<NodeId, A::Error> {
        map.next_value_seed(NodeSeed {
            limits: self.limits,
            builder: &mut *self.builder,
            depth: self.depth.saturating_add(1),
        })
    }

    /// Lê o valor racional de um nó `literal`.
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se o valor for inválido ou exceder os
    /// limites de bits.
    fn read_value<'de, A: MapAccess<'de>>(&self, map: &mut A) -> Result<Rational, A::Error> {
        let wire: RationalWire = map.next_value()?;
        Rational::from_decimal_strings(&wire.n, &wire.d, self.limits).map_err(de::Error::custom)
    }

    /// Processa um nó `literal`.
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se campos faltarem ou limites forem
    /// excedidos.
    fn visit_literal<'de, A: MapAccess<'de>>(&mut self, map: &mut A) -> Result<NodeId, A::Error> {
        let mut value: Option<Rational> = None;
        let mut original_input: Option<Box<str>> = None;
        let mut metadata = Metadata::new();

        while let Some(key) = map.next_key::<Box<str>>()? {
            match key.as_ref() {
                "value" => value = Some(self.read_value(map)?),
                "original_input" => original_input = Some(map.next_value()?),
                "metadata" => metadata = map.next_value()?,
                other => return Err(de::Error::unknown_field(other, LITERAL_FIELDS)),
            }
        }

        let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
        let original_input =
            original_input.ok_or_else(|| de::Error::missing_field("original_input"))?;
        let id = self
            .builder
            .literal(value, original_input)
            .map_err(de::Error::custom)?;
        self.builder
            .set_metadata(id, metadata)
            .map_err(de::Error::custom)?;
        Ok(id)
    }

    /// Processa um nó `operation`.
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se campos faltarem ou limites forem
    /// excedidos.
    fn visit_operation<'de, A: MapAccess<'de>>(&mut self, map: &mut A) -> Result<NodeId, A::Error> {
        let mut op: Option<OperationType> = None;
        let mut operands: Option<Vec<NodeId>> = None;
        let mut metadata = Metadata::new();

        while let Some(key) = map.next_key::<Box<str>>()? {
            match key.as_ref() {
                "op_type" => op = Some(map.next_value()?),
                "operands" => operands = Some(self.read_operands(map)?),
                "metadata" => metadata = map.next_value()?,
                other => return Err(de::Error::unknown_field(other, OPERATION_FIELDS)),
            }
        }

        let op = op.ok_or_else(|| de::Error::missing_field("op_type"))?;
        let operands = operands.ok_or_else(|| de::Error::missing_field("operands"))?;
        let id = self
            .builder
            .operation(op, &operands)
            .map_err(de::Error::custom)?;
        self.builder
            .set_metadata(id, metadata)
            .map_err(de::Error::custom)?;
        Ok(id)
    }

    /// Processa um nó `group`.
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se campos faltarem ou limites forem
    /// excedidos.
    fn visit_group<'de, A: MapAccess<'de>>(&mut self, map: &mut A) -> Result<NodeId, A::Error> {
        let mut child: Option<NodeId> = None;
        let mut metadata = Metadata::new();

        while let Some(key) = map.next_key::<Box<str>>()? {
            match key.as_ref() {
                "child" => child = Some(self.read_child(map)?),
                "metadata" => metadata = map.next_value()?,
                other => return Err(de::Error::unknown_field(other, GROUP_FIELDS)),
            }
        }

        let child = child.ok_or_else(|| de::Error::missing_field("child"))?;
        let id = self.builder.group(child).map_err(de::Error::custom)?;
        self.builder
            .set_metadata(id, metadata)
            .map_err(de::Error::custom)?;
        Ok(id)
    }

    /// Processa um nó `control`.
    ///
    /// # Errors
    ///
    /// Retorna erro de desserialização se campos faltarem ou limites forem
    /// excedidos.
    fn visit_control<'de, A: MapAccess<'de>>(&mut self, map: &mut A) -> Result<NodeId, A::Error> {
        let mut child: Option<NodeId> = None;
        let mut previous_context_label: Option<Box<str>> = None;
        let mut previous_signature: Option<Box<str>> = None;
        let mut previous_round_strategy: Option<RoundingStrategy> = None;
        let mut metadata = Metadata::new();

        while let Some(key) = map.next_key::<Box<str>>()? {
            match key.as_ref() {
                "child" => child = Some(self.read_child(map)?),
                "previous_context_label" => previous_context_label = Some(map.next_value()?),
                "previous_signature" => previous_signature = Some(map.next_value()?),
                "previous_round_strategy" => previous_round_strategy = Some(map.next_value()?),
                "metadata" => metadata = map.next_value()?,
                other => return Err(de::Error::unknown_field(other, CONTROL_FIELDS)),
            }
        }

        let child = child.ok_or_else(|| de::Error::missing_field("child"))?;
        let label = previous_context_label
            .ok_or_else(|| de::Error::missing_field("previous_context_label"))?;
        let signature =
            previous_signature.ok_or_else(|| de::Error::missing_field("previous_signature"))?;
        let round = previous_round_strategy
            .ok_or_else(|| de::Error::missing_field("previous_round_strategy"))?;
        let id = self
            .builder
            .control(child, label, signature, round)
            .map_err(de::Error::custom)?;
        self.builder
            .set_metadata(id, metadata)
            .map_err(de::Error::custom)?;
        Ok(id)
    }
}

impl<'de> Visitor<'de> for NodeVisitor<'_, '_> {
    type Value = NodeId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("um nó da AST")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<NodeId, A::Error> {
        let mut visitor = self;
        visitor.check_depth::<A::Error>()?;

        let kind_key = map
            .next_key::<Box<str>>()?
            .ok_or_else(|| de::Error::invalid_length(0, &"um nó da AST"))?;
        if kind_key.as_ref() != "kind" {
            return Err(de::Error::custom("o campo 'kind' deve ser o primeiro"));
        }
        let kind: Box<str> = map.next_value()?;

        match kind.as_ref() {
            "literal" => visitor.visit_literal(&mut map),
            "operation" => visitor.visit_operation(&mut map),
            "group" => visitor.visit_group(&mut map),
            "control" => visitor.visit_control(&mut map),
            other => Err(de::Error::unknown_variant(
                other,
                &["literal", "operation", "group", "control"],
            )),
        }
    }
}

/// Semente de desserialização de uma sequência de nós.
struct SeqSeed<'a, 'limits> {
    /// Limites aplicados.
    limits: &'limits Limits,
    /// Construtor compartilhado.
    builder: &'a mut AstBuilder<'limits>,
    /// Profundidade dos nós da sequência.
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for SeqSeed<'_, '_> {
    type Value = Vec<NodeId>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Vec<NodeId>, D::Error> {
        deserializer.deserialize_seq(SeqVisitor {
            limits: self.limits,
            builder: self.builder,
            depth: self.depth,
        })
    }
}

/// Visitante de uma sequência de nós.
struct SeqVisitor<'a, 'limits> {
    /// Limites aplicados.
    limits: &'limits Limits,
    /// Construtor compartilhado.
    builder: &'a mut AstBuilder<'limits>,
    /// Profundidade dos nós da sequência.
    depth: usize,
}

impl<'de> Visitor<'de> for SeqVisitor<'_, '_> {
    type Value = Vec<NodeId>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("uma sequência de nós da AST")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<NodeId>, A::Error> {
        let mut ids = Vec::new();
        while let Some(id) = seq.next_element_seed(NodeSeed {
            limits: self.limits,
            builder: &mut *self.builder,
            depth: self.depth,
        })? {
            ids.push(id);
        }
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::flat::NodeKind;

    fn sample(limits: &Limits) -> FlatAst {
        let mut builder = AstBuilder::new(limits);
        let left = builder
            .literal(Rational::from_i64(10), "10")
            .expect("literal");
        let right = builder
            .literal(Rational::from_i64(3), "3")
            .expect("literal");
        let sum = builder
            .operation(OperationType::Add, &[left, right])
            .expect("operação");
        let group = builder.group(sum).expect("grupo");
        builder
            .insert_metadata(
                left,
                "ref",
                crate::meta_value::MetaValue::Str("SELIC".into()),
            )
            .expect("metadata");
        builder.finish(group).expect("árvore")
    }

    #[test]
    fn round_trip_is_byte_stable() {
        let limits = Limits::default();
        let ast = sample(&limits);
        let first = ast.to_json_string().expect("serializa");
        let parsed = FlatAst::from_json_str(&first, &limits).expect("desserializa");
        let second = parsed.to_json_string().expect("serializa");
        assert_eq!(first, second);
        assert_eq!(parsed.kind(parsed.root()), Some(NodeKind::Group));
    }

    #[test]
    fn rejects_hostile_depth() {
        // `{"kind":"group","child":` repetido além do limite.
        let limits = Limits::new(1_000_000, 3, 10_000, 100, 16_384).expect("limites");
        let mut json = String::new();
        for _ in 0..10 {
            json.push_str(r#"{"kind":"group","child":"#);
        }
        json.push_str(r#"{"kind":"literal","value":{"n":"1","d":"1"},"original_input":"1"}"#);
        for _ in 0..10 {
            json.push('}');
        }
        assert!(FlatAst::from_json_str(&json, &limits).is_err());
    }

    #[test]
    fn rejects_unknown_field() {
        let limits = Limits::default();
        let json =
            r#"{"kind":"literal","value":{"n":"1","d":"1"},"original_input":"1","evil":true}"#;
        assert!(FlatAst::from_json_str(json, &limits).is_err());
    }

    #[test]
    fn rejects_bad_kind_first() {
        let limits = Limits::default();
        let json = r#"{"value":{"n":"1","d":"1"},"kind":"literal","original_input":"1"}"#;
        assert!(FlatAst::from_json_str(json, &limits).is_err());
    }

    #[test]
    fn rejects_float_literal() {
        let limits = Limits::default();
        let json = r#"{"kind":"literal","value":{"n":"1.5","d":"1"},"original_input":"1.5"}"#;
        assert!(FlatAst::from_json_str(json, &limits).is_err());
    }

    #[test]
    fn control_round_trip() {
        let limits = Limits::default();
        let mut builder = AstBuilder::new(&limits);
        let leaf = builder
            .literal(Rational::from_i64(7), "7")
            .expect("literal");
        let control = builder
            .control(leaf, "financial", "abc123", RoundingStrategy::Nbr5891)
            .expect("control");
        let ast = builder.finish(control).expect("árvore");

        let json = ast.to_json_string().expect("serializa");
        assert!(json.contains("nbr5891"));
        let parsed = FlatAst::from_json_str(&json, &limits).expect("desserializa");
        assert_eq!(json, parsed.to_json_string().expect("serializa"));
        assert_eq!(parsed.kind(parsed.root()), Some(NodeKind::Control));
    }
}
