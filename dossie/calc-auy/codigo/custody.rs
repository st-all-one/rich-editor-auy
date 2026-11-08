//! Verificação de custódia (*parse, don't validate*).

use crate::ast::{FlatAst, NodeData, NodeId, OperationType};
use crate::error::CalcAuyError;
use crate::limits::Limits;
use crate::numeric::RoundingStrategy;

/// AST que passou pela verificação de todos os elos de custódia.
///
/// O campo é privado: a única forma de obter um `VerifiedAst` é via
/// [`FlatAst::verify_custody`]. O motor de avaliação (E04) só aceita este tipo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedAst {
    /// Árvore verificada.
    inner: FlatAst,
}

impl VerifiedAst {
    /// Retorna a árvore subjacente.
    #[must_use]
    pub const fn ast(&self) -> &FlatAst {
        &self.inner
    }

    /// Consome o invólucro e devolve a árvore.
    #[must_use]
    pub fn into_inner(self) -> FlatAst {
        self.inner
    }
}

/// Verificador de elos de custódia.
///
/// Implementações resolvem a assinatura anterior (ex.: consultando um log) e
/// confirmam sua validade.
pub trait CustodyVerifier {
    /// Verifica um elo de custódia.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::CustodyViolation`] se o elo for inválido.
    fn verify_link(
        &self,
        previous_context_label: &str,
        previous_signature: &str,
        previous_round_strategy: RoundingStrategy,
    ) -> Result<(), CalcAuyError>;
}

impl FlatAst {
    /// Verifica a custódia e produz um [`VerifiedAst`].
    ///
    /// Percorre todos os nós `Control`, delegando a verificação de cada elo ao
    /// [`CustodyVerifier`], e exige que `cross_context_add` tenha apenas
    /// operandos de controle.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::LimitExceeded`] se os limites forem excedidos ou
    /// [`CalcAuyError::CustodyViolation`] se algum elo/operando for inválido.
    pub fn verify_custody(
        self,
        verifier: &dyn CustodyVerifier,
        limits: &Limits,
    ) -> Result<VerifiedAst, CalcAuyError> {
        limits.check_nodes(self.len())?;
        limits.check_depth(self.max_depth())?;

        for (_, node) in self.iter_nodes() {
            self.check_node(node, verifier)?;
        }

        Ok(VerifiedAst { inner: self })
    }

    /// Verifica um único nó (controle ou `cross_context_add`).
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::CustodyViolation`] se o nó violar a custódia.
    fn check_node(
        &self,
        node: &NodeData,
        verifier: &dyn CustodyVerifier,
    ) -> Result<(), CalcAuyError> {
        match node {
            NodeData::Control {
                previous_context_label,
                previous_signature,
                previous_round_strategy,
                ..
            } => Self::check_control(
                previous_context_label,
                previous_signature,
                *previous_round_strategy,
                verifier,
            ),
            NodeData::Operation {
                op: OperationType::CrossContextAdd,
                operands,
            } => self.check_cross_context(operands),
            _ => Ok(()),
        }
    }

    /// Valida os campos de um nó de controle e delega ao verificador.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::CustodyViolation`] se algum campo for vazio ou o
    /// elo for rejeitado.
    fn check_control(
        label: &str,
        signature: &str,
        round: RoundingStrategy,
        verifier: &dyn CustodyVerifier,
    ) -> Result<(), CalcAuyError> {
        if label.is_empty() {
            return Err(CalcAuyError::CustodyViolation {
                reason: "rótulo de contexto anterior vazio".to_string(),
            });
        }
        if signature.is_empty() {
            return Err(CalcAuyError::CustodyViolation {
                reason: "assinatura anterior vazia".to_string(),
            });
        }
        verifier.verify_link(label, signature, round)
    }

    /// Exige que todos os operandos de `cross_context_add` sejam `Control`.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::CustodyViolation`] se algum operando não for um
    /// nó de controle.
    fn check_cross_context(&self, operands: &[NodeId]) -> Result<(), CalcAuyError> {
        for operand in operands {
            self.check_control_operand(*operand)?;
        }
        Ok(())
    }

    /// Exige que um operando de `cross_context_add` seja um nó de controle.
    ///
    /// # Errors
    ///
    /// Retorna [`CalcAuyError::CustodyViolation`] se o operando não for um nó
    /// de controle.
    fn check_control_operand(&self, operand: NodeId) -> Result<(), CalcAuyError> {
        if matches!(self.node(operand), Some(NodeData::Control { .. })) {
            return Ok(());
        }
        Err(CalcAuyError::CustodyViolation {
            reason: "cross_context_add exige operandos de controle verificados".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstBuilder;
    use crate::numeric::Rational;

    /// Verificador que aceita apenas a assinatura "ok".
    struct MockVerifier {
        /// Assinatura aceita.
        accepted: &'static str,
    }

    impl CustodyVerifier for MockVerifier {
        fn verify_link(
            &self,
            label: &str,
            signature: &str,
            _round: RoundingStrategy,
        ) -> Result<(), CalcAuyError> {
            if label == "financial" && signature == self.accepted {
                Ok(())
            } else {
                Err(CalcAuyError::CustodyViolation {
                    reason: format!("elo rejeitado: {label}/{signature}"),
                })
            }
        }
    }

    fn control_ast(label: &str, signature: &str) -> FlatAst {
        let limits = Limits::default();
        let mut builder = AstBuilder::new(&limits);
        let leaf = builder
            .literal(Rational::from_i64(1), "1")
            .expect("literal");
        let control = builder
            .control(leaf, label, signature, RoundingStrategy::Nbr5891)
            .expect("control");
        builder.finish(control).expect("árvore")
    }

    #[test]
    fn valid_custody_passes() {
        let ast = control_ast("financial", "ok");
        let verified = ast
            .verify_custody(&MockVerifier { accepted: "ok" }, &Limits::default())
            .expect("custódia válida");
        assert_eq!(verified.ast().len(), 2);
    }

    #[test]
    fn invalid_link_rejected() {
        let ast = control_ast("financial", "bad");
        assert!(
            ast.verify_custody(&MockVerifier { accepted: "ok" }, &Limits::default())
                .is_err()
        );
    }

    #[test]
    fn cross_context_requires_control_operands() {
        let limits = Limits::default();
        let mut builder = AstBuilder::new(&limits);
        let leaf = builder
            .literal(Rational::from_i64(1), "1")
            .expect("literal");
        let op = builder
            .operation(OperationType::CrossContextAdd, &[leaf])
            .expect("operação");
        let ast = builder.finish(op).expect("árvore");
        assert!(
            ast.verify_custody(&MockVerifier { accepted: "ok" }, &limits)
                .is_err()
        );
    }
}
