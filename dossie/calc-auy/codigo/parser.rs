//! Parser por precedência (*climbing*) que emite direto na arena.

use smallvec::SmallVec;

use calc_auy_core::numeric::parse::parse_rational;
use calc_auy_core::{AstBuilder, FlatAst, Limits, NodeId, OperationType, Rational};

use crate::error::{ParseError, Span};
use crate::lexer::{Lexer, Token, TokenKind};

/// *Binding power* do operador unário (mais forte que `^` para o operando).
const UNARY_BP: u8 = 16;

/// Sinal unário.
#[derive(Debug, Clone, Copy)]
enum Sign {
    /// `+`.
    Plus,
    /// `-`.
    Minus,
}

/// Converte o token binário em operação, se aplicável.
const fn token_operation(kind: TokenKind) -> Option<OperationType> {
    match kind {
        TokenKind::Plus => Some(OperationType::Add),
        TokenKind::Minus => Some(OperationType::Sub),
        TokenKind::Star => Some(OperationType::Mul),
        TokenKind::Slash => Some(OperationType::Div),
        TokenKind::Percent => Some(OperationType::Mod),
        TokenKind::Caret => Some(OperationType::Pow),
        _ => None,
    }
}

/// Calcula o *binding power* `(esquerda, direita)` de uma operação.
const fn binding_power(op: OperationType) -> (u8, u8) {
    let strength = 10_u8.saturating_sub(op.precedence()).saturating_mul(2);
    if op.is_right_associative() {
        (strength, strength.saturating_sub(1))
    } else {
        (strength, strength.saturating_add(1))
    }
}

/// Coleta todos os tokens, parando no primeiro erro léxico.
///
/// # Errors
///
/// Retorna [`ParseError`] se houver caractere inválido.
fn collect_tokens(input: &str) -> Result<Vec<Token<'_>>, ParseError> {
    Lexer::new(input).collect()
}

/// Parser com estado.
struct Parser<'a, 'limits> {
    /// Tokens coletados.
    tokens: Vec<Token<'a>>,
    /// Índice do próximo token.
    index: usize,
    /// Construtor da arena.
    builder: AstBuilder<'limits>,
    /// Limites.
    limits: &'limits Limits,
    /// Comprimento da entrada (para o span de EOF).
    input_len: usize,
}

impl<'a, 'limits> Parser<'a, 'limits> {
    /// Cria o parser.
    const fn new(tokens: Vec<Token<'a>>, limits: &'limits Limits, input_len: usize) -> Self {
        Self {
            tokens,
            index: 0,
            builder: AstBuilder::new(limits),
            limits,
            input_len,
        }
    }

    /// Espia o próximo token.
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.index).copied()
    }

    /// Consome o próximo token.
    const fn bump(&mut self) {
        self.index = self.index.saturating_add(1);
    }

    /// Retorna o span do token atual (ou o fim da entrada).
    fn current_span(&self) -> Span {
        self.peek().map_or_else(
            || Span::new(self.input_len, self.input_len),
            |token| token.span,
        )
    }

    /// Cria um erro sintático na posição atual.
    fn syntax(&self, message: impl Into<String>) -> ParseError {
        ParseError::syntax(message, self.current_span())
    }

    /// Retorna a operação binária do próximo token, se houver.
    fn peek_binary(&self) -> Option<OperationType> {
        self.peek().and_then(|token| token_operation(token.kind))
    }

    /// Retorna `true` se `%` deve ser tratado como percentual (pós-fixo).
    fn percent_is_postfix(&self) -> bool {
        if self.peek().map(|token| token.kind) != Some(TokenKind::Percent) {
            return false;
        }
        let next = self.tokens.get(self.index.saturating_add(1));
        !matches!(
            next.map(|token| token.kind),
            Some(TokenKind::Number | TokenKind::LParen)
        )
    }

    /// Analisa uma expressão com *binding power* mínimo.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se a sintaxe for inválida ou um limite for
    /// excedido.
    fn parse_expression(&mut self, min_bp: u8) -> Result<NodeId, ParseError> {
        let mut lhs = self.parse_prefix()?;
        while let Some(op) = self
            .peek_binary()
            .filter(|op| binding_power(*op).0 >= min_bp)
        {
            let (_, right_bp) = binding_power(op);
            let span = self.current_span();
            self.bump();
            lhs = self.build_chain(op, right_bp, lhs, span)?;
        }
        Ok(lhs)
    }

    /// Constrói um nó binário (ou achatado para operações associativas).
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se um operando for inválido ou um limite for
    /// excedido.
    fn build_chain(
        &mut self,
        op: OperationType,
        right_bp: u8,
        lhs: NodeId,
        span: Span,
    ) -> Result<NodeId, ParseError> {
        let mut operands: SmallVec<[NodeId; 4]> = SmallVec::new();
        operands.push(lhs);
        operands.push(self.parse_expression(right_bp)?);

        if op.is_associative() {
            self.collect_associative(op, right_bp, &mut operands)?;
            self.builder
                .flattened_operation(op, &operands)
                .map_err(|error| ParseError::Limit { error, span })
        } else {
            self.builder
                .operation(op, &operands)
                .map_err(|error| ParseError::Limit { error, span })
        }
    }

    /// Consome operandos adicionais de uma cadeia associativa.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se algum operando for inválido.
    fn collect_associative(
        &mut self,
        op: OperationType,
        right_bp: u8,
        operands: &mut SmallVec<[NodeId; 4]>,
    ) -> Result<(), ParseError> {
        while self.peek_binary() == Some(op) {
            self.bump();
            operands.push(self.parse_expression(right_bp)?);
        }
        Ok(())
    }

    /// Analisa um operando primário (com percentual pós-fixo).
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se a sintaxe for inválida.
    fn parse_prefix(&mut self) -> Result<NodeId, ParseError> {
        let mut node = self.parse_primary()?;
        while self.percent_is_postfix() {
            let span = self.current_span();
            self.bump();
            node = self.apply_percent(node, span)?;
        }
        Ok(node)
    }

    /// Multiplica um nó por `1/100`.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se um limite for excedido.
    fn apply_percent(&mut self, node: NodeId, span: Span) -> Result<NodeId, ParseError> {
        let hundredth = Rational::new_checked(1, 100, self.limits)
            .map_err(|error| ParseError::Limit { error, span })?;
        let factor = self.build_literal(hundredth, "1/100", span)?;
        self.builder
            .operation(OperationType::Mul, &[node, factor])
            .map_err(|error| ParseError::Limit { error, span })
    }

    /// Analisa um operando primário.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se a sintaxe for inválida.
    fn parse_primary(&mut self) -> Result<NodeId, ParseError> {
        let token = self
            .peek()
            .ok_or_else(|| self.syntax("operando esperado"))?;
        match token.kind {
            TokenKind::Number => {
                self.bump();
                let value =
                    parse_rational(token.text, self.limits).map_err(|error| ParseError::Limit {
                        error,
                        span: token.span,
                    })?;
                self.build_literal(value, token.text, token.span)
            }
            TokenKind::LParen => {
                self.bump();
                let inner = self.parse_expression(0)?;
                self.expect(TokenKind::RParen)?;
                self.builder
                    .group(inner)
                    .map_err(|error| ParseError::Limit {
                        error,
                        span: token.span,
                    })
            }
            TokenKind::Minus => {
                self.bump();
                self.parse_unary(Sign::Minus, token.span)
            }
            TokenKind::Plus => {
                self.bump();
                self.parse_unary(Sign::Plus, token.span)
            }
            _ => Err(self.syntax("operando esperado")),
        }
    }

    /// Analisa um operando com sinal unário.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se a sintaxe for inválida.
    fn parse_unary(&mut self, sign: Sign, span: Span) -> Result<NodeId, ParseError> {
        let operand = self.parse_expression(UNARY_BP)?;
        match sign {
            Sign::Plus => Ok(operand),
            Sign::Minus => {
                let zero = self.build_literal(Rational::ZERO, "0", span)?;
                self.builder
                    .operation(OperationType::Sub, &[zero, operand])
                    .map_err(|error| ParseError::Limit { error, span })
            }
        }
    }

    /// Cria um nó literal, mapeando erros de limite.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se um limite for excedido.
    fn build_literal(
        &mut self,
        value: Rational,
        text: &str,
        span: Span,
    ) -> Result<NodeId, ParseError> {
        self.builder
            .literal(value, text)
            .map_err(|error| ParseError::Limit { error, span })
    }

    /// Exige um token de um tipo, consumindo-o.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se o token não corresponder.
    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        let token = self
            .peek()
            .ok_or_else(|| self.syntax(format!("esperado `{}`", kind.as_str())))?;
        if token.kind == kind {
            self.bump();
            Ok(())
        } else {
            Err(ParseError::syntax(
                format!(
                    "esperado `{}`, encontrado `{}`",
                    kind.as_str(),
                    token.kind.as_str()
                ),
                token.span,
            ))
        }
    }

    /// Exige o fim da entrada.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se houver tokens restantes.
    fn expect_eof(&self) -> Result<(), ParseError> {
        self.peek().map_or(Ok(()), |token| {
            Err(ParseError::syntax(
                format!("token inesperado `{}`", token.kind.as_str()),
                token.span,
            ))
        })
    }

    /// Finaliza a construção da arena.
    ///
    /// # Errors
    ///
    /// Retorna [`ParseError`] se a raiz for inválida.
    fn finish(self, root: NodeId) -> Result<FlatAst, ParseError> {
        let span = Span::new(self.input_len, self.input_len);
        self.builder
            .finish(root)
            .map_err(|error| ParseError::Limit { error, span })
    }
}

/// Analisa uma expressão textual em uma `FlatAst`.
///
/// # Errors
///
/// Retorna [`ParseError`] se a entrada for léxica ou sintaticamente inválida,
/// ou se exceder os limites.
pub fn parse(input: &str, limits: &Limits) -> Result<FlatAst, ParseError> {
    let tokens = collect_tokens(input)?;
    let mut parser = Parser::new(tokens, limits, input.len());
    let root = parser.parse_expression(0)?;
    parser.expect_eof()?;
    parser.finish(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use calc_auy_core::engine::eval;

    fn parse_value(input: &str) -> Result<Rational, ParseError> {
        let limits = Limits::default();
        let ast = parse(input, &limits)?;
        let verified =
            ast.verify_custody(&AcceptAll, &limits)
                .map_err(|error| ParseError::Limit {
                    error,
                    span: Span::new(0, 0),
                })?;
        eval(&verified, &limits).map_err(|error| ParseError::Limit {
            error,
            span: Span::new(0, 0),
        })
    }

    /// Verificador permissivo.
    struct AcceptAll;

    impl calc_auy_core::CustodyVerifier for AcceptAll {
        fn verify_link(
            &self,
            _label: &str,
            _signature: &str,
            _round: calc_auy_core::RoundingStrategy,
        ) -> Result<(), calc_auy_core::CalcAuyError> {
            Ok(())
        }
    }

    fn value(input: &str) -> Rational {
        parse_value(input).expect("expressão válida")
    }

    #[test]
    fn parses_precedence() {
        assert_eq!(value("1 + 2 * 3"), Rational::from_i64(7));
        assert_eq!(value("(1 + 2) * 3"), Rational::from_i64(9));
        assert_eq!(value("2 * 3 + 1"), Rational::from_i64(7));
    }

    #[test]
    fn power_is_right_associative() {
        assert_eq!(value("2 ^ 3 ^ 2"), Rational::from_i64(512));
        assert_eq!(value("(2 ^ 3) ^ 2"), Rational::from_i64(64));
    }

    #[test]
    fn subtraction_is_left_associative() {
        assert_eq!(value("10 - 3 - 2"), Rational::from_i64(5));
    }

    #[test]
    fn division_is_left_associative() {
        assert_eq!(value("100 / 5 / 2"), Rational::from_i64(10));
    }

    #[test]
    fn unary_minus() {
        assert_eq!(value("-5"), Rational::from_i64(-5));
        assert_eq!(value("-2 ^ 2"), Rational::from_i64(-4));
        assert_eq!(value("3 - -2"), Rational::from_i64(5));
    }

    #[test]
    fn percent_is_postfix() {
        assert_eq!(value("50%"), Rational::HALF);
        assert_eq!(value("200% * 2"), Rational::from_i64(4));
        assert_eq!(value("10 % 3"), Rational::ONE);
    }

    #[test]
    fn errors_have_positions() {
        let limits = Limits::default();
        let error = parse("1 + * 2", &limits).expect_err("erro esperado");
        assert_eq!(error.span(), Span::new(4, 5));
        let error = parse("(1 + 2", &limits).expect_err("erro esperado");
        assert_eq!(error.span().start, 6);
    }

    #[test]
    fn rejects_empty_input() {
        let limits = Limits::default();
        assert!(parse("", &limits).is_err());
        assert!(parse("   ", &limits).is_err());
    }
}
