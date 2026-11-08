//! Avaliação iterativa pós-ordem sobre a arena.

use crate::ast::{FlatAst, NodeData, NodeId, OperationType};
use crate::canonical::Canonical;
use crate::error::CalcAuyError;
use crate::integrity::{Digest, VerifiedAst};
use crate::limits::Limits;
use crate::numeric::Rational;

use super::cache::EvalCache;

/// Ação para um nó de passagem conforme o estado.
const fn passthrough_action(child: NodeId, state: usize) -> Action {
    if state == 0 {
        Action::Descend(child)
    } else {
        Action::PopPassthrough
    }
}

/// Erro de aridade para uma operação.
fn arity_error(op: OperationType) -> CalcAuyError {
    CalcAuyError::InvalidInput {
        message: format!("aridade inválida para {}", op.as_str()),
    }
}

/// Erro para `NodeId` inválido.
fn invalid_node(id: NodeId) -> CalcAuyError {
    CalcAuyError::InvalidInput {
        message: format!("NodeId inválido: {}", id.get()),
    }
}

/// Ação a executar em um passo da avaliação.
enum Action {
    /// Empilha um literal.
    PushLiteral(Rational),
    /// Desce para um filho.
    Descend(NodeId),
    /// Fecha um nó de passagem (`Group`/`Control`).
    PopPassthrough,
    /// Finaliza uma operação.
    Finalize(OperationType),
}

/// Memoização intra-avaliação para nós compartilhados (DAG).
struct DagMemo {
    /// Número de referências por nó (`> 1` = compartilhado).
    refs: Vec<u32>,
    /// Valores já calculados por nó.
    values: Vec<Option<Rational>>,
}

/// Visita os filhos diretos de um nó.
fn node_children(node: &NodeData, mut visit: impl FnMut(NodeId)) {
    match node {
        NodeData::Literal { .. } => {}
        NodeData::Operation { operands, .. } => operands.iter().for_each(|id| visit(*id)),
        NodeData::Group { child } | NodeData::Control { child, .. } => visit(*child),
    }
}

/// Incrementa a contagem de referências de um nó.
fn count_ref(refs: &mut [u32], child: NodeId, shared: &mut bool) {
    let Some(slot) = refs.get_mut(child.index()) else {
        return;
    };
    *slot = slot.saturating_add(1);
    *shared |= *slot > 1;
}

/// Constrói a memoização DAG se a arena tiver subárvores compartilhadas.
fn build_dag(ast: &FlatAst) -> Option<DagMemo> {
    let mut refs = vec![0_u32; ast.len()];
    let mut shared = false;
    for (_, node) in ast.iter_nodes() {
        node_children(node, |child| count_ref(&mut refs, child, &mut shared));
    }
    if shared {
        Some(DagMemo {
            refs,
            values: vec![None; ast.len()],
        })
    } else {
        None
    }
}

/// Motor iterativo de avaliação.
struct Evaluator<'a> {
    /// Arena.
    ast: &'a FlatAst,
    /// Limites.
    limits: &'a Limits,
    /// Pilha de quadros `(nó, próximo filho)`.
    stack: Vec<(NodeId, usize)>,
    /// Pilha de valores já avaliados.
    values: Vec<Rational>,
    /// Hashes de subárvore (vazio = sem memoização).
    hashes: Vec<Digest>,
    /// Cache opcional.
    cache: Option<&'a mut EvalCache>,
    /// Memoização DAG (opcional).
    dag: Option<DagMemo>,
}

impl Evaluator<'_> {
    /// Executa a avaliação até esvaziar a pilha.
    ///
    /// # Errors
    ///
    /// Retorna erro se algum `NodeId` for inválido, a aridade for incorreta ou
    /// um limite for excedido.
    fn run(mut self) -> Result<Rational, CalcAuyError> {
        while !self.stack.is_empty() {
            self.step()?;
        }
        self.values.pop().ok_or_else(|| CalcAuyError::InvalidInput {
            message: "avaliação não produziu valor".to_string(),
        })
    }

    /// Executa um único passo.
    ///
    /// # Errors
    ///
    /// Retorna erro se o nó for inválido ou a operação falhar.
    fn step(&mut self) -> Result<(), CalcAuyError> {
        let Some(&(id, state)) = self.stack.last() else {
            return Ok(());
        };
        if state == 0 && (self.try_memo(id) || self.try_cached(id)) {
            return Ok(());
        }

        let ast = self.ast;
        let node = ast.node(id).ok_or_else(|| invalid_node(id))?;
        let action = match node {
            NodeData::Literal { value, .. } => Action::PushLiteral(value.clone()),
            NodeData::Group { child } | NodeData::Control { child, .. } => {
                passthrough_action(*child, state)
            }
            NodeData::Operation { op, operands } => operands
                .get(state)
                .map_or(Action::Finalize(*op), |child| Action::Descend(*child)),
        };
        self.apply(action, id)
    }

    /// Aplica a ação calculada.
    ///
    /// # Errors
    ///
    /// Retorna erro se a finalização de operação falhar.
    fn apply(&mut self, action: Action, id: NodeId) -> Result<(), CalcAuyError> {
        match action {
            Action::PushLiteral(value) => {
                self.values.push(value);
                self.stack.pop();
            }
            Action::Descend(child) => {
                self.bump_state();
                self.stack.push((child, 0));
            }
            Action::PopPassthrough => {
                self.stack.pop();
            }
            Action::Finalize(op) => self.finalize(op, id)?,
        }
        Ok(())
    }

    /// Finaliza uma operação consumindo seus operandos da pilha de valores.
    ///
    /// # Errors
    ///
    /// Retorna erro se a operação falhar ou exceder um limite.
    fn finalize(&mut self, op: OperationType, id: NodeId) -> Result<(), CalcAuyError> {
        let ast = self.ast;
        let operands = ast.operands(id).unwrap_or(&[]);
        let start = self.values.len().saturating_sub(operands.len());
        let result = apply_operation(op, self.values.get(start..).unwrap_or(&[]), self.limits)?;
        self.values.truncate(start);
        self.store_memo(id, &result);
        self.store_cached(id, &result);
        self.values.push(result);
        self.stack.pop();
        Ok(())
    }

    /// Incrementa o contador de filho do quadro do topo.
    fn bump_state(&mut self) {
        if let Some(last) = self.stack.last_mut() {
            last.1 = last.1.saturating_add(1);
        }
    }

    /// Tenta reutilizar um resultado memoizado (DAG), pulando a subárvore.
    fn try_memo(&mut self, id: NodeId) -> bool {
        let Some(dag) = self.dag.as_mut() else {
            return false;
        };
        let index = id.index();
        if dag.refs.get(index).copied().unwrap_or(0) <= 1 {
            return false;
        }
        let Some(Some(value)) = dag.values.get(index) else {
            return false;
        };
        let value = value.clone();
        self.values.push(value);
        self.stack.pop();
        true
    }

    /// Armazena um resultado para nós compartilhados.
    fn store_memo(&mut self, id: NodeId, result: &Rational) {
        let Some(dag) = self.dag.as_mut() else {
            return;
        };
        let index = id.index();
        if dag.refs.get(index).copied().unwrap_or(0) > 1
            && let Some(slot) = dag.values.get_mut(index)
        {
            *slot = Some(result.clone());
        }
    }

    /// Tenta reutilizar um resultado memoizado, pulando a subárvore.
    fn try_cached(&mut self, id: NodeId) -> bool {
        let Some(hash) = self.hashes.get(id.index()).copied() else {
            return false;
        };
        let Some(value) = self.cache.as_mut().and_then(|cache| cache.get(&hash)) else {
            return false;
        };
        self.values.push(value);
        self.stack.pop();
        true
    }

    /// Armazena um resultado no cache, se habilitado.
    fn store_cached(&mut self, id: NodeId, result: &Rational) {
        let Some(hash) = self.hashes.get(id.index()).copied() else {
            return;
        };
        if let Some(cache) = self.cache.as_mut() {
            cache.insert(hash, result.clone());
        }
    }
}

/// Avalia uma AST verificada.
///
/// # Errors
///
/// Retorna erro se a avaliação falhar ou exceder um limite.
pub fn eval(verified: &VerifiedAst, limits: &Limits) -> Result<Rational, CalcAuyError> {
    eval_ast(verified.ast(), limits)
}

/// Avalia uma arena diretamente (uso interno).
///
/// # Errors
///
/// Retorna erro se a avaliação falhar ou exceder um limite.
pub(crate) fn eval_ast(ast: &FlatAst, limits: &Limits) -> Result<Rational, CalcAuyError> {
    limits.check_nodes(ast.len())?;
    limits.check_depth(ast.max_depth())?;
    Evaluator {
        ast,
        limits,
        stack: vec![(ast.root(), 0)],
        values: Vec::new(),
        hashes: Vec::new(),
        cache: None,
        dag: build_dag(ast),
    }
    .run()
}

/// Avalia uma AST verificada com memoização por conteúdo.
///
/// # Errors
///
/// Retorna erro se a avaliação falhar ou exceder um limite.
pub fn eval_memoized(
    verified: &VerifiedAst,
    limits: &Limits,
    cache: &mut EvalCache,
) -> Result<Rational, CalcAuyError> {
    let ast = verified.ast();
    limits.check_nodes(ast.len())?;
    limits.check_depth(ast.max_depth())?;
    let hashes = compute_subtree_hashes(ast)?;
    Evaluator {
        ast,
        limits,
        stack: vec![(ast.root(), 0)],
        values: Vec::new(),
        hashes,
        cache: Some(cache),
        dag: build_dag(ast),
    }
    .run()
}

/// Calcula hashes de conteúdo por subárvore, em ordem de construção.
///
/// Ignora metadados (compatível com `value_hash`), permitindo memoização sem
/// comprometer a integridade. Exige que os filhos tenham índice menor que o pai
/// (invariante de construção da arena).
///
/// # Errors
///
/// Retorna erro se a arena não estiver em ordem pós-ordem.
pub(crate) fn compute_subtree_hashes(ast: &FlatAst) -> Result<Vec<Digest>, CalcAuyError> {
    let mut hashes: Vec<Digest> = Vec::with_capacity(ast.len());
    // Buffer reutilizado entre literais: evita uma alocação por nó.
    let mut buffer = Vec::new();
    for (_, node) in ast.iter_nodes() {
        let mut node_hasher = blake3::Hasher::new();
        match node {
            NodeData::Literal { value, .. } => {
                node_hasher.update(b"literal");
                buffer.clear();
                value.write_canonical(&mut buffer);
                node_hasher.update(&buffer);
            }
            NodeData::Operation { op, operands } => {
                hash_operation(&mut node_hasher, *op, operands, &hashes)?;
            }
            NodeData::Group { child } => {
                node_hasher.update(b"group");
                node_hasher.update(child_hash(&hashes, *child)?.as_bytes());
            }
            NodeData::Control {
                child,
                previous_context_label,
                previous_signature,
                previous_round_strategy,
            } => {
                node_hasher.update(b"control");
                node_hasher.update(child_hash(&hashes, *child)?.as_bytes());
                node_hasher.update(previous_context_label.as_bytes());
                node_hasher.update(previous_signature.as_bytes());
                node_hasher.update(previous_round_strategy.as_str().as_bytes());
            }
        }
        hashes.push(Digest::from_bytes(*node_hasher.finalize().as_bytes()));
    }
    Ok(hashes)
}

/// Alimenta o hasher com uma operação e os hashes dos operandos.
///
/// # Errors
///
/// Retorna erro se algum operando ainda não tiver hash calculado.
fn hash_operation(
    hasher: &mut blake3::Hasher,
    op: OperationType,
    operands: &[NodeId],
    child_hashes: &[Digest],
) -> Result<(), CalcAuyError> {
    hasher.update(b"operation");
    hasher.update(op.as_str().as_bytes());
    for operand in operands {
        hasher.update(child_hash(child_hashes, *operand)?.as_bytes());
    }
    Ok(())
}

/// Busca o hash de um filho já calculado.
///
/// # Errors
///
/// Retorna erro se o filho ainda não tiver sido processado.
fn child_hash(hashes: &[Digest], child: NodeId) -> Result<Digest, CalcAuyError> {
    hashes
        .get(child.index())
        .copied()
        .ok_or_else(|| CalcAuyError::InvalidInput {
            message: "arena fora de ordem pós-ordem".to_string(),
        })
}

/// Aplica uma operação a um *slice* de operandos, sem alocar `Vec` por operação.
///
/// # Errors
///
/// Retorna erro em divisão/resto por zero, aridade inválida ou resultado
/// irracional.
pub(crate) fn apply_operation(
    op: OperationType,
    operands: &[Rational],
    limits: &Limits,
) -> Result<Rational, CalcAuyError> {
    let result = match op {
        OperationType::Add | OperationType::CrossContextAdd => fold_add(operands),
        OperationType::Sub => fold_sub(operands),
        OperationType::Mul => {
            check_mul_budget(operands, limits)?;
            fold_mul(operands)
        }
        OperationType::Div => fold_div(operands)?,
        OperationType::Mod => fold_rem(operands)?,
        OperationType::DivInt => fold_div_int(operands)?,
        OperationType::Pow => pow_operands(operands, limits)?,
    };
    limits.check_bits(result.bits())?;
    Ok(result)
}

/// Pré-checa o orçamento de bits de uma multiplicação.
///
/// O numerador e o denominador do produto são limitados pela soma dos bits dos
/// operandos (a redução só diminui). Se essa soma já excede o orçamento, a
/// operação é rejeitada **sem computar** o produto — defesa contra `DoS` análoga
/// ao orçamento de [`pow_operands`].
///
/// # Errors
///
/// Retorna [`CalcAuyError::MathOverflow`] se a soma dos bits exceder o limite.
fn check_mul_budget(operands: &[Rational], limits: &Limits) -> Result<(), CalcAuyError> {
    let mut total: u64 = 0;
    for operand in operands {
        total = total.saturating_add(operand.bits());
    }
    if total > limits.max_bits {
        return Err(CalcAuyError::MathOverflow {
            max_bits: limits.max_bits,
            actual: total,
        });
    }
    Ok(())
}

/// Soma todos os operandos (vazio = `0`) via redução em árvore exata.
fn fold_add(operands: &[Rational]) -> Rational {
    crate::numeric::reduce::sum_tree(operands)
}

/// Multiplica todos os operandos (vazio = `1`) via redução em árvore exata.
fn fold_mul(operands: &[Rational]) -> Rational {
    crate::numeric::reduce::prod_tree(operands)
}

/// Subtrai da esquerda para a direita (vazio = `0`).
fn fold_sub(operands: &[Rational]) -> Rational {
    let Some((first, rest)) = operands.split_first() else {
        return Rational::ZERO;
    };
    let mut accumulator = first.clone();
    for operand in rest {
        accumulator = accumulator.sub(operand);
    }
    accumulator
}

/// Divide da esquerda para a direita.
///
/// # Errors
///
/// Retorna erro se houver menos de dois operandos ou divisão por zero.
fn fold_div(operands: &[Rational]) -> Result<Rational, CalcAuyError> {
    let Some((first, rest)) = operands.split_first() else {
        return Err(arity_error(OperationType::Div));
    };
    if rest.is_empty() {
        return Err(arity_error(OperationType::Div));
    }
    let mut accumulator = first.clone();
    for operand in rest {
        accumulator = accumulator.checked_div(operand)?;
    }
    Ok(accumulator)
}

/// Resto da divisão da esquerda para a direita.
///
/// # Errors
///
/// Retorna erro se houver menos de dois operandos ou divisão por zero.
fn fold_rem(operands: &[Rational]) -> Result<Rational, CalcAuyError> {
    let Some((first, rest)) = operands.split_first() else {
        return Err(arity_error(OperationType::Mod));
    };
    if rest.is_empty() {
        return Err(arity_error(OperationType::Mod));
    }
    let mut accumulator = first.clone();
    for operand in rest {
        accumulator = accumulator.checked_rem(operand)?;
    }
    Ok(accumulator)
}

/// Divisão inteira (truncada) da esquerda para a direita.
///
/// # Errors
///
/// Retorna erro se houver menos de dois operandos ou divisão por zero.
fn fold_div_int(operands: &[Rational]) -> Result<Rational, CalcAuyError> {
    let Some((first, rest)) = operands.split_first() else {
        return Err(arity_error(OperationType::DivInt));
    };
    if rest.is_empty() {
        return Err(arity_error(OperationType::DivInt));
    }
    let mut accumulator = first.clone();
    for operand in rest {
        accumulator = accumulator.checked_div(operand)?.trunc();
    }
    Ok(accumulator)
}

/// Potenciação com exatamente dois operandos.
///
/// # Errors
///
/// Retorna erro se a aridade não for 2 ou a potência for inválida.
fn pow_operands(operands: &[Rational], limits: &Limits) -> Result<Rational, CalcAuyError> {
    let (base, rest) = operands
        .split_first()
        .ok_or_else(|| arity_error(OperationType::Pow))?;
    let exponent = rest
        .first()
        .ok_or_else(|| arity_error(OperationType::Pow))?;
    if rest.len() != 1 {
        return Err(arity_error(OperationType::Pow));
    }
    base.pow(exponent, limits)
}

/// Constrói um `SmallVec` de operandos a partir de uma slice (uso em testes).
#[cfg(test)]
fn to_operands(operands: &[Rational]) -> smallvec::SmallVec<[Rational; 4]> {
    operands.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstBuilder;

    fn evaluate(
        build: impl FnOnce(&mut AstBuilder<'_>) -> NodeId,
    ) -> Result<Rational, CalcAuyError> {
        let limits = Limits::default();
        let mut builder = AstBuilder::new(&limits);
        let root = build(&mut builder);
        let ast = builder.finish(root)?;
        eval_ast(&ast, &limits)
    }

    #[test]
    fn evaluates_shared_subtree_once() {
        let result = evaluate(|builder| {
            let base = builder
                .literal(Rational::from_i64(2), "2")
                .expect("literal");
            let exponent = builder
                .literal(Rational::from_i64(100), "100")
                .expect("literal");
            let power = builder
                .operation(OperationType::Pow, &[base, exponent])
                .expect("pow");
            // Mesmo nó duas vezes: a memoização DAG deve reaproveitar o valor.
            builder
                .operation(OperationType::Mul, &[power, power])
                .expect("mul")
        });
        let expected = Rational::from_bigint(num_bigint::BigInt::from(2).pow(200));
        assert_eq!(result.expect("avaliação"), expected);
    }

    #[test]
    fn evaluates_nested_expression() {
        // (1 + 2) * 3 - 4 = 5
        let result = evaluate(|builder| {
            let one = builder
                .literal(Rational::from_i64(1), "1")
                .expect("literal");
            let two = builder
                .literal(Rational::from_i64(2), "2")
                .expect("literal");
            let three = builder
                .literal(Rational::from_i64(3), "3")
                .expect("literal");
            let four = builder
                .literal(Rational::from_i64(4), "4")
                .expect("literal");
            let sum = builder
                .operation(OperationType::Add, &[one, two])
                .expect("soma");
            let product = builder
                .operation(OperationType::Mul, &[sum, three])
                .expect("produto");
            builder
                .operation(OperationType::Sub, &[product, four])
                .expect("subtração")
        })
        .expect("avaliação");
        assert_eq!(result, Rational::from_i64(5));
    }

    #[test]
    fn evaluates_group_passthrough() {
        let result = evaluate(|builder| {
            let value = builder
                .literal(Rational::from_i64(7), "7")
                .expect("literal");
            builder.group(value).expect("grupo")
        })
        .expect("avaliação");
        assert_eq!(result, Rational::from_i64(7));
    }

    #[test]
    fn division_by_zero_is_error() {
        let result = evaluate(|builder| {
            let one = builder
                .literal(Rational::from_i64(1), "1")
                .expect("literal");
            let zero = builder.literal(Rational::ZERO, "0").expect("literal");
            builder
                .operation(OperationType::Div, &[one, zero])
                .expect("divisão")
        });
        assert!(matches!(result, Err(CalcAuyError::DivisionByZero)));
    }

    #[test]
    fn pow_requires_two_operands() {
        let result = evaluate(|builder| {
            let two = builder
                .literal(Rational::from_i64(2), "2")
                .expect("literal");
            builder.operation(OperationType::Pow, &[two]).expect("pow")
        });
        assert!(result.is_err());
    }

    #[test]
    fn bit_budget_is_enforced() {
        let limits = Limits::new(8, 500, 1_000, 100, 16_384).expect("limites");
        let mut builder = AstBuilder::new(&limits);
        let big = builder
            .literal(Rational::from_i64(1_000_000), "1000000")
            .expect("literal");
        let big2 = builder
            .literal(Rational::from_i64(1_000_000), "1000000")
            .expect("literal");
        let product = builder
            .operation(OperationType::Mul, &[big, big2])
            .expect("produto");
        let ast = builder.finish(product).expect("árvore");
        assert!(eval_ast(&ast, &limits).is_err());
    }

    #[test]
    fn mul_prechecks_bit_budget() {
        let limits = Limits::new(50, 500, 1_000, 100, 16_384).expect("limites");
        let mut builder = AstBuilder::new(&limits);
        let big = builder
            .literal(Rational::from_i64(1_000_000_000), "1000000000")
            .expect("literal");
        let one = builder.literal(Rational::ONE, "1").expect("literal");
        let ok = builder
            .operation(OperationType::Mul, &[big, one])
            .expect("produto");
        let ast_ok = builder.finish(ok).expect("árvore");
        assert!(eval_ast(&ast_ok, &limits).is_ok());

        let mut builder = AstBuilder::new(&limits);
        let left = builder
            .literal(Rational::from_i64(1_000_000_000), "1000000000")
            .expect("literal");
        let right = builder
            .literal(Rational::from_i64(1_000_000_000), "1000000000")
            .expect("literal");
        let product = builder
            .operation(OperationType::Mul, &[left, right])
            .expect("produto");
        let ast = builder.finish(product).expect("árvore");
        assert!(matches!(
            eval_ast(&ast, &limits),
            Err(CalcAuyError::MathOverflow { .. })
        ));
    }

    #[test]
    fn to_operands_round_trips() {
        let values = [Rational::from_i64(1), Rational::from_i64(2)];
        let small = to_operands(&values);
        assert_eq!(small.len(), 2);
    }
}
