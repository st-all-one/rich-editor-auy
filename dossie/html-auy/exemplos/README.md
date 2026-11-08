# Exemplos — `html-auy`

Tudo aqui foi **executado** na máquina do dossiê (cargo 1.99.0, rustc 1.99.0), com o crate lido por
caminho (`../../../ref/html-auy/crates/html-auy`, três níveis a partir de `exemplos/`), ou seja, sem
publicar nada e sem rede.

## 1. `exemplos/` — gera HTML tipado (compila e roda)

```
exemplos/
  Cargo.toml      # html-auy por caminho; [workspace] vazio para não herdar workspace
  src/main.rs     # fragmento html!, Img, Field::text, Page/ShellData, determinismo
```

Comando e saída observada (recortada; o texto completo está abaixo em forma de blocos):

```console
$ cargo run --offline
   Compiling html-auy-macros-core v0.1.1 (.../ref/html-auy/crates/html-auy-macros-core)
   Compiling html-auy-macros v0.1.1 (.../ref/html-auy/crates/html-auy-macros)
   Compiling html-auy v0.1.1 (.../ref/html-auy/crates/html-auy)
   Compiling exemplo-html-auy v0.1.0 (.../dossie/html-auy/exemplos)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.13s
     Running `target/debug/exemplo-html-auy`
== 1. fragmento (`html!`) ==
<section class="card flat"><h2>Vendas &lt;trimestre&gt; &amp; cia</h2><p>aspas &quot; e apóstrofo &#x27; — o texto é escapado, não interpretado</p><a href="/relatorios/2026">Ver relatório</a><img src="/media/grafico.avif" alt="Gráfico de vendas" width="720" height="480"></section>

== 2. primitivo (`Img`) ==
<img src="/media/grafico.avif" alt="" width="64" height="64">

== 3. campo (`Field::text`) ==
<div><label for="f-nome">Nome</label><span aria-hidden="true">*</span><input type="text" id="f-nome" name="nome" value="Ana" required aria-describedby="f-nome-help"><span id="f-nome-help">Como você quer ser chamado?</span></div>

== 4. página (`Page`) ==
<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Demo html-auy</title></head><body><div role="status" aria-atomic="true"></div><main id="main" tabindex="-1"><h1>Demonstração</h1><section class="card flat">...</section></main></body></html>

== 5. determinismo ==
mesmos dados => mesmos bytes (577 bytes)
```

O que a saída mostra, sem interpretação:

- `<` `>` `&` `"` `'` do texto viraram `&lt; &gt; &amp; &quot; &#x27;` — e **só** esses
  (`escape.rs:15-24`). Acento e `—` passam intactos (UTF-8 não é escapado).
- `class="card flat"` saiu **ordenado** (`flat` antes de `card`): `Class::new` ordena
  (`value/class.rs:33-38`).
- `alt=""` do `Img::decorative()` continua sendo emitido: vazio é declaração explícita, não omissão.
- `Field::text` gerou `label for` → `id`, `aria-describedby` e o `*` com `aria-hidden="true"` — nada
  disso foi escrito à mão; veio de `FieldState` (`field.rs:205-207`, `field.rs:215-241`).
- `Page` emitiu `role="status" aria-atomic="true"` vazio e `<main id="main" tabindex="-1">`
  (`shell.rs:755`, `shell.rs:770`) — a região viva e o alvo do skip link existem por construção,
  mesmo sem o consumidor pedir.

## 2. `exemplos/falha-compilacao/` — o vocabulário recusando (não compila de propósito)

```console
$ cargo check --offline 2>&1
error: atributo proibido `onclick` em `div`
         = regra: spec:element.div
         = ajuda: estilo é token e comportamento é asset declarado
  --> src/main.rs:10:25

error: `disabled` é booleano; use `disabled=(expr)` com `expr: bool`
         = regra: spec:boolean_attributes (D19)
         = ajuda: presença = true; `disabled="false"` é verdadeiro em HTML
  --> src/main.rs:13:51

error: atributo desconhecido `alt` em `span`
         = ajuda: atributos válidos em `span`: class, id, lang

error: `alt` não aceita literal em `span`
         = regra: contexto de escape (D1)
         = ajuda: use `alt=(Alt::new("…"))` ou `Alt::decorative()`

error: `<div>` não é permitido dentro de `<p>`
         = regra: conteúdo-modelo (spec:element.p)
         = ajuda: use `<span>` (phrasing) ou mova o `<div>` para fora do `<p>`

error: `<form>` não é elemento de `html!`
         = regra: spec:form_requires_csrf
         = ajuda: use `Form::open(&action, Method::Post(csrf), children)`

error: could not compile `falha-compilacao` (bin "falha-compilacao") due to 6 previous errors
```

Conclusões observadas (não inferidas):

1. **Todos os erros do arquivo saem de uma vez** (6 erros), não um por `cargo check`: `diagnostic::emit`
   percorre a lista (`diagnostic.rs:76-84`).
2. Cada mensagem tem o formato `o que` + `= regra:` (a invariante do `spec`) + `= ajuda:`.
3. Um único `html!` pode gerar **dois** erros (`alt` em `span`: atributo desconhecido **e** literal
   recusado). É ruído real de UX, registrado no `avaliacao.md`.
4. Custo colateral observado: o `unreachable!()` da expansão gera `warning: unreachable statement` nas
   linhas seguintes, e o `use` fica "unused" quando o bloco que o usava falhou. O compilador não
   compila, mas o output tem warnings que não são do usuário.
5. `cargo check` do exemplo que falha: **~5,0 s a frio** (reobservado na revisão) e **0,06 s a
   quente**; do exemplo que roda: **2,09–2,13 s** (primeiro build, com as três crates). Build
   incremental é da ordem de décimos de segundo.

## 3. Verificações executadas no próprio `ref/html-auy`

```console
$ python3 tools/validate_spec.py
0 erro(s), 0 aviso(s)                     # exit=0

$ python3 tools/gen_spec.py --check
ok 2 tabela(s) em dia com elements.toml   # exit=0

$ python3 tools/check_links.py
396 link(s) relativos verificados, 0 quebrado(s)   # exit=0

$ python3 -m unittest discover -s tools/tests
Ran 18 tests in 0.814s — OK

$ cargo check -p html-auy --offline
Finished `dev` profile ... in 13.38s      # exit=0

$ cargo test -p html-auy --offline
24 alvos (23 arquivos de `tests/` + unidade); 186 testes + 3 doctests = 189 `ok`, 0 failed
$ cargo test -p html-auy --features audit --offline
207 testes, todos `ok`
```

Contagem por alvo do `cargo test` (observada, `tests/*.rs`): `page` 24, `values` 24, `golden` 18,
`media` 14, `spec_parity` 14, `vocabulary` 13, `form` 10, `forms` 10, `field` 9, `hints` 9,
`determinism` 5, `escape_props` 4, `markup` 4, `acceptance_port` 4, `response` 5, `value_props` 5,
`acceptance_forms` 3, `acceptance_m2` 2, `acceptance_m1` 1, `acceptance_media` 1, `miri_smoke` 1,
`ui` 1 (harness `trybuild` sobre 76 casos), unidade 5, `audit` 0 sem a feature.

## 4. O que **não** foi executado

| Item | Motivo |
|---|---|
| `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo doc` | Não é o foco do dossiê e a árvore já está no CI do projeto; não rodei para não alongar. |
| `cargo deny check`, `cargo audit`, `cargo machete` | Exigem rede e binários auxiliares não instalados. |
| `cargo miri test` | Requer nightly + componente miri; não instalado nesta máquina. |
| `cargo fuzz run parser` | Requer nightly e `cargo-fuzz`; não instalado. O alvo existe em `fuzz/fuzz_targets/parser.rs` e foi lido, não executado. |
| `cargo +1.92 test` (MSRV) | Toolchain 1.92 não instalado; só 1.99 está presente (`rustc --version`). A MSRV é afirmação de doc (`Cargo.toml:21`), **não verificada**. |
| Build com `--features audit` de fato usando `html5ever` | A feature compilou e os 207 testes passaram; não instrumentei o `audit` sobre uma página do exemplo. |
| Benchmark de render | O projeto não versiona `criterion` no workspace; as metas de `docs/09` §1 são declaração, não medição. |

## 5. Como reproduzir

```bash
# exemplos/ (gera HTML)
cd dossie/html-auy/exemplos && cargo run --offline

# exemplos/falha-compilacao/ (deve falhar)
cd dossie/html-auy/exemplos/falha-compilacao && cargo check --offline

# suíte do crate
cd ref/html-auy && cargo test -p html-auy --offline
cd ref/html-auy && cargo test -p html-auy --features audit --offline
```

Os diretórios `target/` gerados por essas execuções foram removidos depois da captura (o
`Cargo.lock` de cada exemplo foi mantido para reproduzir as mesmas versões).
