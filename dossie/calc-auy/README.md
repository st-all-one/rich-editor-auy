# calc-auy — dossiê de referência

> Calculadora **determinística, auditável e exata** para contexto financeiro,
> escrita em Rust como workspace de 13 pacotes. Não é um editor de texto e não
> tenta ser: é a demonstração mais limpa que temos de um **pipeline com tipos
> de estado, bytes canônicos e limites como invariantes** — a mesma tese de
> `plan/00_panorama.md` (do nosso repositório), aplicada a um domínio menor.
>
> Código lido e executado a partir de `ref/calc-auy/`. Nada em `ref/` foi
> alterado; builds usaram `CARGO_TARGET_DIR=/tmp/dossie-calc-target`.
> Convenção de caminhos: `crates/…`, `docs/…`, `schema/…` são relativos a
> `ref/calc-auy/`; `plan/…` é relativo à raiz deste repositório.

## Ficha técnica

| Item | Valor | Fonte |
|---|---|---|
| Repositório local | `ref/calc-auy` | — |
| Commit | `08f5a3f0a907cf503f8b2afbdae0a19f6618f79f` ("optimization") | `git log -1` |
| Data do commit | 2026-10-04 17:13:33 −03:00 | `git log -1` |
| Natureza do clone | **shallow**, 1 commit, branch `main`, sem tags | `git rev-list --count HEAD` = 1; `.git/shallow` existe |
| Versão | `0.1.0` | `Cargo.toml:21`; `calc-auy --version` |
| Edição / MSRV | 2024 / 1.98 | `Cargo.toml:22-23` |
| Licença | **MPL-2.0** | `Cargo.toml:24`; `LICENSE` |
| Publicação | `publish = false` em todos os pacotes | `Cargo.toml:25` |
| Pacotes | 13 (12 crates + `xtask`) | `Cargo.toml:2-16`; `cargo metadata` |
| Testes | **322** passando (311 `#[test]`/proptest + 11 doctests), 0 falhas | `exemplos/README.md` §2 |
| ADRs | 16 | `docs/adr/ADR-0001…ADR-016` |
| Livro do produto | 8 capítulos | `docs/book/src/SUMMARY.md` |

Observação de proveniência: `dossie/_PROVENIENCIA.md` **lista** `calc-auy` desde o adendo de
2026-11-08; os dados abaixo também foram levantados do próprio clone.

## 1. O que é, exatamente

Um binário (`calc-auy`) e uma biblioteca (`calc-auy-api`) que respondem a uma
pergunta só: *qual é o valor exato desta expressão financeira, com que
arredondamento, e como provo depois que o valor não foi adulterado?*

```console
$ calc-auy "1 + 2"
3.00

$ calc-auy --params '{"mode":"monetary","locale":"pt-BR"}' "10000 * (1 + 14.75%) ^ 3"
R$ 15.109,78
```

Saídas reais, coladas de uma execução minha; ver `exemplos/README.md` §3.
Fidelidade de bytes: o separador entre `R$` e o número é **U+00A0** (NBSP),
não espaço ASCII — `od -c` mostra `R $ 302 240 1 5 . 1 0 9 , 7 8`. Por
legibilidade ele aparece aqui (e em `exemplos/`) como espaço comum.

Três das cinco promessas do README do projeto, e onde cada uma vive:

1. **Exatidão** — racional híbrido `Small{u64,u64}` / `Large{BigInt,BigInt}`
   com `d > 0` e `gcd(n,d) == 1` sempre
   (`crates/calc-auy-core/src/numeric/rational.rs:20-36`; ADR-0001). `f64` é
   **impossível por tipo**: `IntoRational` é selado e não o implementa
   (`crates/calc-auy-builder/src/into_rational.rs:7-14`), com teste de
   compilação que falha (`tests/ui/f64_rejected.rs`).
2. **Determinismo** — a mesma entrada e o mesmo sal produzem a mesma
   assinatura BLAKE3 *keyed* (`integrity/sign.rs:9-21`), sobre bytes canônicos
   escritos à mão (`canonical/mod.rs:231-284`). Verificado por testes e por
   duas execuções do binário (`tests/cli.rs:134-140`).
3. **Custódia** — o motor só avalia `VerifiedAst`, cujo campo é privado e só
   nasce de `FlatAst::verify_custody` (`integrity/custody.rs:13-33`, `:61-75`;
   ADR-0005). O `Committed` só nasce de `commit`
   (`engine/commit.rs:13-22`, `:63`).

## 2. Que problema resolve

O problema declarado é auditoria financeira: um cálculo que alimenta uma
decisão precisa de (a) valor exato, (b) reprodutibilidade byte a byte,
(c) prova de que o valor não foi trocado e (d) prova de que ele veio de um
contexto cuja assinatura anterior foi conferida. Nada disso é "calcular":
é **custodiar o cálculo**.

Isso produz um desenho incomum e útil de estudar:

- a saída não é uma string, é um `Committed` assinado; todo formato deriva
  dele (`crates/calc-auy-output/src/output.rs:62-70`);
- o erro não é uma exceção, é um `ProblemDetails` RFC 7807 com `type` URI
  estável (`crates/calc-auy-core/src/error.rs:109-160`);
- a configuração não é global mutável, é `InstanceConfig` emprestada
  (`crates/calc-auy-api/src/lib.rs:38-95`);
- e o "wire" é versionado (`trace_version`, `SignaturePayload` ×
  `SignaturePayloadV2`), com política explícita de evolução
  (`docs/adr/ADR-013-schema-versionado.md`).

**O que ele não resolve** (e não é falha): edição de texto, layout, cursor,
seleção, IME, colaboração, HTML de entrada. Um editor não tem análogo de
"racional exato"; tem análogo de "documento canônico".

## 3. Resultado real

Executado neste dossiê (transcrições completas em `exemplos/README.md`):

| O que | Resultado |
|---|---|
| `cargo metadata --offline --no-deps` | 13 pacotes, todos `MPL-2.0`, versão `0.1.0` |
| `cargo test --workspace --locked` | **322 passaram, 0 falharam** (311 + 11 doctests), 41 alvos, ~7 s em cache |
| `cargo test -p calc-auy-core --lib` | 167 passaram em 2,64 s |
| Binário | `calc-auy 0.1.0`; `1 + 2` → `3.00`; monetário → `R$ 15.109,78` |
| Assinatura | `7*6` → `82b1836d…724d` (64 hex), idêntica em duas execuções |
| Verificação | traço válido → `true`/exit 0; traço adulterado → `false`/exit 6 |
| Erros | divisão por zero → 5; parse → 3; irracional → 7; overflow → 4; uso → 2 |
| Lote NDJSON | 4 linhas → 4 linhas na ordem, erro no corpo, exit 0 |
| HTML | `aria-label` com descrição verbal real em pt-BR |
| SVG | documento válido **sem** `title`/`desc`/`role` |
| Harness da API | parse → commit → assinatura → hibernate/hydrate → JSON → `ProblemDetails` |

Falha real encontrada: a **primeira** execução do workspace falhou em um
doctest com erro de *linker*; a causa era `Disk quota exceeded` no `/tmp`,
não o projeto. Depois de liberar espaço, o doctest passou. Registrado em
`exemplos/README.md` §2.4 — não escondido.

## 4. Como usar

### 4.1 CLI em 30 segundos

```console
$ calc-auy "1 + 2"                                   # BODY posicional; stdin se omitido
$ calc-auy --params '{"mode":"json"}' "1 + 2"        # multi-formato + signature
$ echo "1/3" | calc-auy --params '{"decimals":4}'    # config em JSON, nunca flags espalhadas
$ calc-auy --batch < lote.ndjson                     # NDJSON → NDJSON, ordem preservada
$ calc-auy --params '{"op":"commit"}' "1+2" | calc-auy verify
```

A convenção é dura e boa: **um** argumento posicional (`BODY`); todo o resto
em `--params='{...}'`; campo desconhecido é recusado
(`crates/calc-auy/src/config.rs:58`; `crates/calc-auy/src/cli.rs:1-40`).
Precedência `--params` > ambiente (`CALC_AUY_*`) > arquivo `chave = valor`
apontado por `CALC_AUY_CONFIG` > padrões (`config.rs:1-5`, `:112-159`).

### 4.2 Biblioteca

```rust
use calc_auy_api::{CalcAuy, CustodyVerifier, InstanceConfig, OutputOptions,
                   RoundingStrategy, Salt};

struct AcceptAll;
impl CustodyVerifier for AcceptAll {
    fn verify_link(&self, _l: &str, _s: &str, _r: RoundingStrategy)
        -> Result<(), calc_auy_api::CalcAuyError> { Ok(()) }
}

let app = CalcAuy::new(
    InstanceConfig::new(Salt::new("demo")).with_rounding(RoundingStrategy::Nbr5891, 2),
);
let committed = app.parse("10000 * (1 + 14.75%) ^ 3")?.commit(&AcceptAll)?;
assert_eq!(committed.output(OutputOptions::new()).to_string_decimal(), "15109.78");
```

O `?` só compila porque `CalcAuyError` é o tipo único de erro de todas as
camadas — uma fachada, um erro (`crates/calc-auy-api/src/lib.rs:13-22`).
O harness completo e executado está em `exemplos/src/main.rs`.

### 4.3 Superfícies que existem

- **CLI**: `format`/`commit`/`sign` + subcomandos `verify`, `hydrate`, `repl`,
  `schema`, `man [--completions bash|zsh|fish]`.
- **WASM**: `WasmCalc` (`parse/from/hydrate/commit/verify/format/monetary`),
  erros como `ProblemDetails` JS (`crates/calc-auy-wasm/src/lib.rs:120-175`).
- **Codecs**: Protobuf, CBOR, MessagePack e JSON sobre um único `WireNode`
  (`crates/calc-auy-processors/src/wire.rs:10-66`).
- **Schema**: `schema/calc_auy.{cddl,proto,schema.json}`, versionado.

Não existe: servidor HTTP, MCP, plugin runtime, importação de expressão de
terceiros.

## 5. O que funciona / o que não funciona

### Funciona (verificado por execução ou teste)

- Aritmética exata com promoção `Small → Large` e invariantes preservados
  (`rational.rs:98-114`; 167 testes de core, incluindo proptests
  `small_and_large_agree`).
- Canonicalização determinística e assinatura verificável
  (`canonical/mod.rs`; `sign.rs`; CLI §3.2 de `exemplos/README.md`).
- Custódia ponta a ponta com `VerifiedAst` e `cross_context_add`
  (`custody.rs:112-160`; testes `invalid_link_rejected`,
  `cross_context_requires_control_operands`).
- Limites em 5 eixos, impostos na construção e **durante** a desserialização
  (`ast/wire.rs:172-183`, `:217`; `limits.rs:34-192`).
- Exit codes estáveis e `ProblemDetails` em JSON, com relógio injetável
  (`crates/calc-auy/src/error.rs:22-59`; observados na §3.3).
- 9 locales, 6 estratégias de arredondamento, 10 modos de saída no CLI.
- HTML acessível de verdade: `aria-label` = descrição verbal do realizer.
- Lote NDJSON ordenado com erros por item.

### Não funciona / não existe (e importa)

| Limitação | Evidência | Impacto |
|---|---|---|
| `SystemClock::now_rfc3339` devolve `"2024-01-01T00:00:00Z"` fixo, com `TODO` | `clock/mod.rs:25-34` | timestamp de produção não é real; só o eixo determinístico funciona |
| `schema` anuncia 7 modos e omite `json`, `html`, `svg` | `commands.rs:316-331` vs saída real | introspecção incompleta; quem gera UI pelo `schema` perde 3 modos |
| SVG/PNG sem `title`/`desc`/`role` | `svg.rs:88-99`; saída real | a11y só existe no caminho HTML |
| `publish = false` | `Cargo.toml:25` | não há consumo por crates.io; só path/git |
| Assinatura simétrica | `sign.rs:16-21` | quem tem o sal pode forjar; não há verificação por terceiro sem o segredo |
| `rust-toolchain.toml` fixa 1.98, sem `cargo` no ambiente | §0 de `exemplos/README.md` | build exigiu 1.99; MSRV 1.98 **não verificada** |
| Livro mostra `丸め：nbr5891`, código imprime `NBR-5891` | `docs/book/src/i18n.md:12-14` vs saída real | drift doc↔código |
| Fuzz/Miri/bench/clippy não executados aqui | `exemplos/README.md` §5 | performance e higiene são afirmações do projeto |

## 6. Prós e contras como referência para o `rich-editor-auy`

### Copiar (conceito, não arquivo)

1. **Estados como tipos, não como flags.** `Draft → Committed → Verified`
   (`calculation.rs:27-45`) é o mesmo problema que `EditorState` +
   `Transaction` + `Applied` têm: um estado não pode oferecer operação que
   não faz sentido nele, e o erro deve ser de compilação. O `trybuild`
   (`builder/tests/ui/draft_has_no_result.rs`) é a prova de que a garantia
   existe.
2. **Um escritor canônico à mão, com modo.** `Canonical` + `Trace`/`Value`
   (`canonical/mod.rs:54-66`) responde à pergunta que vamos ter no wire do
   documento: *o que entra no hash?* No nosso caso: o documento entra,
   `doc_id`/timestamps não (`plan/04_arquitetura.md:447-485`).
3. **Limites medidos enquanto o dado é lido.** O `DeserializeSeed` com
   orçamento (`ast/wire.rs:172-183`) é a defesa certa para abrir documento de
   terceiro sem confiar no payload — e é mais forte que validar depois.
4. **Erro com `type` estável + código de saída estável.** `ProblemDetails` +
   tabela de exit codes (`crates/calc-auy-core/src/error.rs:109-160`;
   `crates/calc-auy/src/error.rs:29-37`) é o que permite um adaptador
   (CLI/MCP) ser testável de fora.
5. **Fixture de conformidade como especificação executável.** 32 vetores
   (`crates/calc-auy-api/tests/corpus/vectors.json`), 3 deles com digest
   canônico e um comando para regenerar (`crates/calc-auy-api/tests/corpus.rs`).
   Nós precisaríamos disso para steps/wire.
6. **Relógio e aleatoriedade atrás de uma porta.** `Clock` com `MockClock`
   (`clock/mod.rs:1-20`) é como se testa determinismo sem dormir.
7. **Processo**: `#[expect(reason)]`, `HashMap` proibido onde a ordem importa,
   proptest diferencial contra oráculo independente
   (`engine/differential.rs:1-7`).

### Evitar

1. **Copiar o código.** MPL-2.0 por arquivo, domínio de calculadora, e o
   código é específico de AST aritmética. `ideias.md` pede reimplementação.
2. **O culto ao `BigInt`.** Nosso documento tem texto, não inteiros de 1 Mbit;
   o equivalente útil é a *arena* e o *tipo de id*, não o racional.
3. **O `deny_unknown_fields` como dogma no wire de interop.** Recusar campo
   desconhecido é ótimo para *assinatura* e péssimo para *leitura de documento
   antigo*. Precisamos das duas políticas explícitas, não de uma só
   (`ast/wire.rs:342`).
4. **Introspecção escrita à mão** (`commands.rs:316-331`). Se publicarmos
   `schema` para MCP/CLI, ele tem de ser gerado do mesmo registrador que a
   implementação usa — senão mente.
5. **Anotação de arredondamento colada em todo formato.** O `round_{...}(…) = …`
   vaza decisão de apresentação para dentro da string. No editor, o análogo
   seria vazar "schema fingerprint" dentro do texto serializado.
6. **Um relógio de mentira em produção.** `SystemClock` fixo é dívida que
   aparece na auditoria; um editor também precisa de `created_at` confiável.

### Adequação

Como **editor**: nenhuma (não tem superfície de edição). Como **referência de
arquitetura de modelo canônico + persistência + erros**: alta. A nota da
rubrica em `avaliacao.md` separa as duas perguntas para não confundir
"calculadora boa" com "editor".

## 7. Licença e maturidade

**MPL-2.0** (`Cargo.toml:24`, `LICENSE` com o texto canônico). Em termos
práticos: usar em produto maior é permitido; arquivos **modificados** do
projeto continuam MPL-2.0; a MPL permite distribuir uma "Larger Work" sob
outros termos, mas não é aconselhamento jurídico. Os arquivos em `codigo/` são
cópias verbatim, sem modificação, e não são compilados aqui (detalhes em
`codigo/INDEX.md`).

Maturidade: versão `0.1.0`, sem publicação, clone shallow de 1 commit, sem
tags, sem histórico de uso em produção verificável. O que há é rigor de
processo muito acima da média para 0.1.x: 16 ADRs, 322 testes, CI com 8 jobs
(fmt, clippy `-D warnings`, test, doc `-D warnings` + doctests, wasm, deny,
audit, Miri opcional), lints de workspace em `deny` (`Cargo.toml:100-160`),
`clippy.toml` com 178 linhas de configuração, 3 alvos de fuzz.

## 8. Mapa do dossiê

| Arquivo | Conteúdo |
|---|---|
| [`arquitetura.md`](arquitetura.md) | pipeline lex→parse→build→verify→commit→output→render, modelo de dados, canonicalização, limites, erros, CLI/WASM, testes, performance, limites |
| [`avaliacao.md`](avaliacao.md) | rubrica 0–5 nas 10 dimensões, matriz de 13 itens de usabilidade com status, confiança |
| [`codigo/INDEX.md`](codigo/INDEX.md) | 6 arquivos reais copiados, origem, licença e por que explicam o coração |
| [`exemplos/README.md`](exemplos/README.md) | tudo o que foi executado, com saída real; o que não foi e por quê |
| [`ideias.md`](ideias.md) | 10 ideias priorizadas + 3 rejeitadas para o `rich-editor-auy`, com hipótese, evidência, encaixe e custo/risco |
