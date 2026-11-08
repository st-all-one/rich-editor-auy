//! Harness mínimo do dossiê do akribie.
//!
//! Demonstra a superfície que interessa a um consumidor headless: `akribie-core` expõe o pipeline
//! inteiro como funções puras sobre dados — nada de `std::fs`, nada de processo, nada de rede.
//!
//! Etapas usadas aqui, na ordem em que `crates/akribie-core/src/lib.rs:9-15` as declara:
//!   lex → parse → resolve → layout → paint → emit
//!
//! O harness mostra cinco coisas que o `rich-editor-auy` também precisa ter:
//!   1. diagnóstico é **dado** (código + linha/coluna + hint), não texto de log;
//!   2. a forma canônica é idempotente (`fmt::format`);
//!   3. o diff é **semântico** (campo, valor antes/depois), não textual;
//!   4. a emissão é determinística — dois `emit::svg` da mesma fonte dão os mesmos bytes;
//!   5. a descrição acessível nasce do modelo (`a11y::describe`), não da geometria.

use akribie_core::config::Config;
use akribie_core::diff;
use akribie_core::emit;
use akribie_core::fmt;
use akribie_core::layout;
use akribie_core::paint;
use akribie_core::pipeline;
use akribie_core::text::Font;

/// Fonte inicial: topologia e rótulos, zero coordenadas.
const OLD: &str = r#"chart v1 "Retry" layout=flow direction=lr lang=pt-BR
node   api   "API"   fill=blue
node retry "Retry" fill=amber shape=cloud
edge api -> retry "5xx"
"#;

/// A mesma topologia com uma mudança de rótulo e uma remoção.
const NEW: &str = r#"chart v1 "Retry" layout=flow direction=lr lang=pt-BR
node api "API Gateway" fill=blue
edge api -> retry "5xx"
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `Font::load` desempacota as duas faces OFL embutidas no crate; não há fontes de sistema.
    let font = Font::load()?;
    let config = Config::default();

    let old = pipeline::check(OLD, &config, &font);
    println!("== 1. diagnóstico é dado ==");
    println!("documento válido: {}", !akribie_core::diag::has_errors(&old.diagnostics));
    println!("diagnósticos: {}", old.diagnostics.len());

    let clean = pipeline::check(NEW, &config, &font);
    println!("segundo documento, diagnósticos: {}", clean.diagnostics.len());

    println!("\n== 2. forma canônica (idempotente) ==");
    let canonical = fmt::format(&clean.document);
    print!("{canonical}");
    let reparsed = pipeline::check(&canonical, &config, &font);
    let again = fmt::format(&reparsed.document);
    println!("fmt(fmt(x)) == fmt(x): {}", canonical == again);

    println!("\n== 3. diff semântico ==");
    for change in diff::diff(&old.chart, &clean.chart) {
        println!("{}", change.line());
    }

    println!("\n== 4. emissão determinística ==");
    let placed = layout::layout(&clean.chart, &font);
    println!("canvas: {} x {}", placed.width, placed.height);
    let scene = paint::paint(&clean.chart, &placed);
    let first = emit::svg(&scene, clean.chart.theme);
    let second = emit::svg(&scene, clean.chart.theme);
    println!("svg bytes: {}", first.len());
    println!("svg(idempotente): {}", first == second);
    println!(
        "primeira linha: {}",
        first.lines().next().unwrap_or_default()
    );

    let json = emit::scene_json(&placed, &clean.chart);
    println!("scene json bytes: {}", json.len());

    println!("\n== 5. acessibilidade derivada do modelo ==");
    println!("desc: {}", scene.desc.lines().next().unwrap_or_default());

    Ok(())
}
