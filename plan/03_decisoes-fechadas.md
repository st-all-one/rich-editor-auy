# 03 — Decisões fechadas

> **Contrato.** A partir daqui, código e documentação derivam destas decisões. Mudar uma decisão
> fechada exige uma entrada nova neste arquivo (contexto → decisão → consequência), não uma
> edição silenciosa. As decisões arquiteturais ainda marcadas como `proposta` em
> [`02_decisoes.md`](02_decisoes.md) **não** estão fechadas; este documento registra apenas o que
> já é base do projeto.

---

## A. Fechadas por instrução do projeto

| # | Decisão final | Consequência |
|---|---|---|
| **F01** | **Projeto `rich-editor-auy`**: engine de texto rico **headless**, agnóstico de UI. | O core não conhece DOM, layout, fontes, IME do SO, clipboard do SO, rede ou arquivo. Ver D01/D41. |
| **F02** | **Rust com pin de desenvolvimento 1.99+ e MSRV 1.92.** | `rust-toolchain.toml` fixa 1.99; `rust-version = "1.92"`; `clippy.toml::msrv = "1.92"`; CI tem job real em 1.92. |
| **F03** | **Rigor máximo de toolchain, clippy, `cargo-deny` e gates.** | Core com `forbid(unsafe_code)`; workspace com `deny` (glue de WASM/testkit local com `reason`), deny de panic/`unwrap`/`expect`/`indexing_slicing`/`arithmetic_side_effects`, `disallowed_types`/`methods`/`macros`, supply chain, MSRV, fuzz, Miri, alocação. |
| **F04** | **Performance é requisito, não aspiração.** | Orçamentos em [`05_desempenho.md`](05_desempenho.md); gates determinísticos de alocação/instrução; nenhuma edição O(documento) no caminho comum. |
| **F05** | **Licença dual MIT OR Apache-2.0.** | `LICENSE-MIT` e `LICENSE-APACHE` na raiz e em cada crate publicado. |
| **F06** | **Documentação de plano em português; código/rustdoc/API em inglês.** | Consumidor externo lê docs.rs em inglês; o time lê o plano em PT. |
| **F07** | **O plano é registro histórico.** | `plan/brainstorm/` não é reescrito; decisão nova entra em `03_decisoes-fechadas.md` com contexto. |
| **F29** | **Acessibilidade universal (AUY) é invariante de core.** Não há flag “a11y opcional”; toda entrega declara role, nome, `alt`, `lang`/`dir`, ordem, ranges e validação. | O adaptador mapeia a árvore; o host audita. O mecanismo (`A11yTree`/`validate_a11y`) é D61/D62; a invariante é anterior e não depende de feature. Sem isso, o documento pode nascer inacessível e o defeito é invisível nos testes de pixel. |
| **F30** | **BLAKE3-256 é o hash canônico do projeto.** Fingerprint de schema, digest do envelope e hashes de conteúdo/auditoria usam BLAKE3 com domain separation; FNV e SHA-256 não entram. | `blake3` é a única dependência normal externa do core (`default-features = false`, `features = ["pure"]`, `no_std`); `integrity` usa BLAKE3 + Ed25519. Qualquer outra dependência normal exige decisão nova. O fingerprint é 32 bytes/64 hex, não uma assinatura. |
| **F31** | **Distribuição oficial: crate + WASM + PHP + TypeScript/JSR/Deno 2.0.** O core é publicado no crates.io; `rich-editor-auy-wasm` gera o artefato WASM; `packages/php` é o pacote Composer; `packages/ts` é o workspace Deno 2.0 que publica os pacotes JSR. | SDKs transportam bytes e chamam WASM/CLI; nenhum reimplementa schema, steps ou fingerprint (F28). Publicação é gate: `cargo package`, `deno publish --dry-run`, `composer validate --strict` e `composer audit`. |
| **F32** | **Toda estrutura observável tem ordem canônica.** `Vec`/`Box<[T]>` ordenado; `BTreeMap`/`BTreeSet` só para lookup/determinização; `HashMap`/`HashSet` proibidos; IDs opacos do host; BLAKE3 domain-separated. | Wire, fingerprint e relatórios são byte-stable entre execuções, arquiteturas e `wasm32`; o golden compara bytes. A tabela de escolhas e o teste de cada borda estão em [`02_decisoes.md`](02_decisoes.md) §K. |

## B. Fechadas por decisão do dossiê (operacionais, reversíveis com nova entrada)

| # | Decisão final | Consequência |
|---|---|---|
| **F08** | **`no_std + alloc` no core.** `#![no_std]`, `extern crate alloc`; feature `std` só para conveniência; CI builda sem default features. | O core não pode usar `std::io`, `std::time`, `std::env`, `std::thread` nem `std::collections`. `Arc` vem de `alloc::sync`; `BTreeMap`/`BTreeSet`/`VecDeque` vêm de `alloc::collections` (targets com atomics). |
| **F09** | **Duas camadas de lint.** Níveis em `[workspace.lints]`; valores em `clippy.toml`. Cada crate declara `[lints] workspace = true`. | Sem o `[lints]`, o gate é ficção. O `clippy.toml` sozinho não liga `restriction`. |
| **F10** | **`forbid(unsafe_code)` no core publicado.** | Nem `#[allow]` local reverte. `unsafe` não é uma opção de implementação; se um dia for indispensável, exige nova decisão e um crate separado. O `testkit`/`fuzz`/`bench` não são o core publicado e podem usar `unsafe` no harness com `SAFETY`; nenhuma linha publicada o contém. |
| **F11** | **Sem `unwrap`/`expect`/`panic` em `src/` nem em testes.** Testes usam `?`, `let else`, `matches!`, `assert!`, `unwrap_or`. | O erro é valor; o panic é bug. O teste também prova que o caminho de erro funciona sem panic. |
| **F12** | **Sem `HashMap`/`HashSet`/`Rc`/`Weak`/`RefCell`/`Cell`/`LinkedList` no core.** | Ordem determinística e `Send + Sync`; `Vec`/`BTreeMap`/`Arc`/`Box`. Exceção local exige `#[expect(..., reason = "…")]` e teste. |
| **F13** | **Sem relógio, ambiente, RNG ou processo.** | Nada de `SystemTime::now`, `Instant::now`, `env::var`, `process::exit`. O tempo e a aleatoriedade entram como dados do consumidor. |
| **F14** | **Sem float no caminho de serialização.** | `AttrValue` não tem `f64`; números canônicos são inteiros. Dimensões/cores são inteiros ou strings. |
| **F15** | **Bytes canônicos.** JSON compacto, ordem de schema, UTF-8 cru, sem `HashMap`. | Golden byte-for-byte; regeneração explícita; hash estável entre máquinas. |
| **F16** | **Erros tipados, `Send + Sync + 'static`, sem `Box<dyn Error>` na API pública.** | `EditError`/`SchemaError`/`ParseError` fechados, com contexto de path/posição. `core::error::Error` (não `std::error::Error`). |
| **F17** | **Testes em cinco camadas:** unidade, propriedade, diferencial contra modelo ingênuo, golden de bytes, fuzz + Miri + alocação/instrução. | Nenhuma camada substitui a outra; a diferencial pega “plausível mas errado”; a de alocação pega O(n) escondido. |
| **F18** | **Gate de 300 linhas para arquivos de `src/`.** Tabelas geradas e testes isentos com justificativa. | `scripts/check_file_length.sh` no `make check`; módulos pequenos por responsabilidade. |
| **F19** | **Gate único local:** `make check` = `fmt --check` + `clippy --all-targets -D warnings` + `test` + `file-length` + `doc`. | Um comando verde é a condição de “terminei”. CI repete o gate e adiciona MSRV, supply chain, Miri e fuzz. |
| **F20** | **Sem dependência normal no core além de `blake3`.** `serde`/`regex`/`html5ever`/`pulldown-cmark` são features ou crates separados. | O core é `no_std + alloc` e compila sem rede. `blake3` (`no_std`, `pure`) é a exceção de F30; qualquer outra dependência normal exige decisão e justificativa de orçamento. |
| **F21** | **`#[expect]` em vez de `#[allow]` sempre que possível; toda exceção tem `reason`.** | A supressão vira erro quando deixa de ser necessária; exceção local, nunca de workspace. |
| **F22** | **Sem camada advisory.** Ou é tipo/erro de compilação, ou teste que falha, ou gate de CI. | Aviso que não bloqueia não entra. |
| **F23** | **Sem `build.rs` e sem geração em tempo de build.** | Tabelas/schemas são código versionado; se houver geração, é por ferramenta explícita e o resultado é commitado, com teste de paridade. |
| **F24** | **Sem segunda fonte de verdade.** Schema, normalização, mapeamento e serialização têm um dono cada. | Mudança de contrato exige decisão registrada e atualização dos testes de paridade. |
| **F25** | **O core não mede nem desenha.** Nada de layout, fonte, glifo, hit testing, ARIA renderizada, foco, contraste ou CSS no core. | A geometria e a acessibilidade renderizada são da UI. O core **possui** a acessibilidade semântica: `A11yTree`, roles, nomes, `alt`, `lang`/`dir`, ordem de leitura e `validate_a11y`. O adaptador traduz a árvore para ARIA/nativo; o host audita contraste/reflow/WCAG. |
| **F26** | **Zero estilo autoral.** O core não emite DOM; o adaptador não usa CSS, `class`, `style` ou wrapper. O host é a raiz e `destroy()` restaura. | "Não interferir na estilização" vira gate: snapshot de DOM e grep de `style=`/`class=`/`<style`/`insertRule` no CI. Defaults do user agent são documentados; o adaptador `mount` é **sempre semântico** (default nativo, `neutral` opt-in); `data`/`bind` só transfere a responsabilidade explicitamente. |
| **F27** | **O core preserva referências, não as resolve.** `RefTarget` tipado e IDs opacos; resolução, backlinks e dangling são do crate `refs`. | Sem rede, DB ou corpus no core; referência não vira URL nem âncora de DOM. O core valida forma e mapeia ranges. |
| **F28** | **SDKs não reimplementam o core.** PHP/TS transportam bytes e chamam WASM/CLI; schema, steps e normalização têm um dono (Rust). | Evita segunda fonte de verdade; validação estrita é do CLI/WASM; bytes canônicos são preservados, não resserializados. |

## C. Ratificação e adiamento

Em **2026-11-08**, por instrução do projeto, as decisões não adiadas de
[`02_decisoes.md`](02_decisoes.md) foram ratificadas como `aceita`:

- **D01–D07** — modelo de dois níveis, COW, runs com teto, posição em byte UTF-8, IDs opcionais,
  `Vec<Arc<Block>>` e `Limits`.
- **D09–D15** — schema runtime + macro, expressões compiladas, máscara de marcas, attrs ordenados,
  fingerprint BLAKE3, unknown estrito/lossy explícito e flags de nó.
- **D16–D25** — enum de steps, `Slice`, mapping com viés, transação atômica, estado imutável,
  seleção, `storedMarks`, normalização local, autofill e `ChangeSet`.
- **D27, D29–D33** — undo sem relógio, orçamento de histórico, comandos declarativos, input rules
  literais, keymap e IME como protocolo.
- **D35–D39** — JSON canônico, writer/parser próprio, envelope versionado, formatos separados e
  `SchemaMismatch` explícito.
- **D48–D49** — licença dual e idioma (plano PT; API/rustdoc EN).
- **D51–D57, D59–D62** — ABI JSON/WASM/PHP, adaptador zero-CSS, IDs/`RefTarget`, `refs`, render
  opt-in, a11y, presets, wrappers e validação.
- **D63–D67** — escopo de formatos de escritório, integridade, diff, conteúdo rico e i18n
  estrutural. D63/D65 têm implementação pós-MVP; D64 é um crate pós-MVP; D66/D67 entram no
  schema/semântica de F1/F3.

Continuam **adiadas** e não podem ser antecipadas: **D08** (codec binário), **D26** (steps no wire),
**D28** (undo seletivo), **D34** (multi-seleção), **D40** (formato binário) e **D50** (colaboração).

A base de F1 está fechada; F1 começa depois de F0. D62 (validação de a11y) entra antes de F3. A
avaliação determinística de cada estrutura de dados está em
[`02_decisoes.md`](02_decisoes.md) §K.

## D. Regras duras que não podem regredir

1. **Nenhum `unsafe` no core publicado.** `#![forbid(unsafe_code)]`; o workspace usa `deny` para
   permitir apenas glue de WASM/testkit local, com `SAFETY` e sem linha publicada no core.
2. **Nenhum panic em entrada não confiável.** JSON, slice colado, step arbitrário, doc persistido:
   tudo devolve `Result`.
3. **Nenhuma iteração não determinística em caminho de bytes.** Sem `HashMap`/`HashSet`, sem
   relógio, sem ambiente, sem RNG.
4. **Nenhuma edição O(documento) no caminho comum.** Se um benchmark de alocação/instrução
   regredir, a mudança não entra.
5. **Nenhum formato sem versão.** O envelope tem `format`, `version` e `schema_fingerprint`.
6. **Nenhuma importação silenciosamente lossy.** `ImportReport` é obrigatório.
7. **Nenhuma API pública sem docs e sem teste de comportamento.** `missing_docs = deny`; exemplos
   compilam em doc-test.
8. **Nenhuma decisão fechada alterada sem nova entrada.** Contexto → decisão → consequência.
9. **Nenhuma dependência nova sem orçamento.** `cargo machete` e `cargo deny` no CI; justificativa
   no PR.
10. **Nenhum arquivo de `src/` acima de 300 linhas.** Fatie por responsabilidade.
11. **Nenhum CSS, `class`, `style` ou wrapper da lib.** O adaptador só toca `data-auy-*`, ARIA e
    `contenteditable`; `destroy()` restaura o host. Se precisa de folha de estilo para funcionar,
    está fora do escopo.
12. **Nenhuma referência resolvida no core.** O core preserva `RefTarget` e IDs; o corpus resolve.
    Sem rede, banco, auth ou publicação no core.
13. **Nenhuma entrega sem árvore semântica e validação de a11y.** Toda feature declara role, nome,
    `alt`, `lang`/`dir`, ordem de leitura e ranges; `Doc::a11y_tree` + `validate_a11y` entram no
    `make check`. A11y não é flag, feature opcional ou “pós-MVP”.
14. **Todo hash do projeto é BLAKE3-256 com domain separation.** Fingerprint, envelope e integridade
    usam BLAKE3; FNV/SHA-256 não entram. `blake3` é a única dependência normal externa do core.

## E. Consequências de longo prazo

- **Posição em byte** significa que a UI precisa converter para UTF-16/grapheme; a conversão é
  explícita e testada. Não há “posição mágica” que sirva a todos.
- **Runs com teto** significa que o mesmo texto pode ter mais de um run adjacente com as mesmas
  marcas; a UI deve mesclar visualmente, e a serialização canônica é a do core (não a da UI).
- **COW** significa que guardar snapshots é barato, mas editar com snapshot retido paga cópia de
  ponteiros; o benchmark mede os dois caminhos.
- **Enum de steps** significa que um step customizado não é uma variante nova sem decisão; ele é
  expresso com os steps existentes.
- **Sem plugin registry** significa que o MVP é mais fechado, mas a API é menor e mais estável; o
  registry pode entrar depois sem quebrar o modelo.
- **Sem CRDT no core** significa que colaboração é um crate separado; o core só garante steps
  serializáveis e mapeáveis.
