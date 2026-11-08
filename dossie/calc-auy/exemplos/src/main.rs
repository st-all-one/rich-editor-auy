//! Harness mínimo do CalcAUY: parse → commit → assinatura → reidratação → erro.
//!
//! Rode a partir de `dossie/calc-auy/exemplos/`:
//! `CARGO_TARGET_DIR=/tmp/dossie-calc-target cargo run --offline --locked`.

use calc_auy_api::{
    CalcAuy, CalcAuyError, CustodyVerifier, InstanceConfig, MockClock, OutputKey, OutputOptions,
    RoundingStrategy, Salt, realizer_for_code,
};

/// Custódia permissiva: o exemplo não usa nós `Control`.
struct AcceptAll;

impl CustodyVerifier for AcceptAll {
    fn verify_link(
        &self,
        _label: &str,
        _signature: &str,
        _round: RoundingStrategy,
    ) -> Result<(), CalcAuyError> {
        Ok(())
    }
}

/// Roda o caminho completo da biblioteca e imprime o que observou.
///
/// # Errors
///
/// Propaga o primeiro erro de domínio não tratado (parse, custódia, commit).
fn main() -> Result<(), CalcAuyError> {
    let config =
        InstanceConfig::new(Salt::new("dossie")).with_rounding(RoundingStrategy::Nbr5891, 2);
    let app = CalcAuy::new(config);

    // 1. Expressão financeira do README.
    let expression = "10000 * (1 + 14.75%) ^ 3";
    let committed = app.parse(expression)?.commit(&AcceptAll)?;
    let output = committed.output(OutputOptions::new());
    println!("expressao    = {expression}");
    println!("decimal      = {}", output.to_string_decimal());
    println!("monetario    = {}", output.to_monetary());
    println!("latex        = {}", output.to_latex());
    println!("assinatura   = {}", committed.signature().hex());

    // 2. Determinismo: mesma entrada + mesmo sal ⇒ mesma assinatura.
    let again = app.parse(expression)?.commit(&AcceptAll)?;
    println!("determinista = {}", committed.signature() == again.signature());

    // 3. Persistência: rascunho (AST em JSON) → reidratar → commitar.
    let draft = app.parse("1/3")?.add(1)?;
    let draft_json = draft.hibernate()?;
    println!("rascunho     = {draft_json}");
    let reopened = app.hydrate(&draft_json)?.commit(&AcceptAll)?;
    println!("reidratado   = {}", reopened.output(OutputOptions::new()).to_string_decimal());

    // 4. JSON multi-formato derivado do mesmo estado selado.
    let realizer = realizer_for_code("pt-BR");
    let json = committed.output(OutputOptions::new()).to_json_formats(
        &[OutputKey::Decimal, OutputKey::Monetary, OutputKey::Verbal],
        realizer.as_ref(),
    )?;
    println!("json         = {json}");

    // 5. Erro de domínio convertido em `ProblemDetails` (RFC 7807).
    match app.parse("1/0")?.commit(&AcceptAll) {
        Ok(_) => println!("erro         = (inesperado: 1/0 passou)"),
        Err(error) => {
            let problem = error.to_problem_details(&MockClock::new());
            println!("erro.type    = {}", problem.r#type);
            println!("erro.status  = {}", problem.status);
            println!("erro.detail  = {}", problem.detail);
        }
    }

    Ok(())
}
