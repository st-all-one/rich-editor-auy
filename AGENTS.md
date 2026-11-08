# AGENTS.md — guia de evolução do `rich-editor-auy`

> Guia para agentes e contribuidores que trabalham **neste repositório**. Ele não é a documentação
> do consumidor; é o contrato de como mexer no código sem quebrar o desenho. O plano completo está
> em [`plan/`](plan/); as decisões fechadas, em
> [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md).

---

## 0. Estado atual

O repositório está na fase de **dossiê** (`plan/`). A fundação `F0` ainda não foi materializada:
não há `Cargo.toml`, crates, `Makefile` nem CI. Quando F0 existir, este guia passa a valer como
está; até lá, ele descreve o contrato que a fundação deve cumprir.

## 1. Missão e fronteira

`rich-editor-auy` é um **engine headless de texto rico**: modelo canônico, schema, posições,
steps, transações, seleção, histórico, serialização e **acessibilidade semântica universal
(AUY)**. A UI é um cliente. Integração com PHP/TS é por JSON canônico + bindings finos; referências
entre publicações são dados (`DocId`/`NodeId` + `RefTarget`), não âncoras de DOM. A distribuição
oficial é crate no crates.io + artefato WASM + pacote Composer `rich-editor-auy/php` + workspace
Deno 2.0 que publica `@rich-editor-auy/*` no JSR (F31).

**O core não faz:** DOM, layout, fontes, shaping, bidi visual, IME do SO, clipboard do SO, ARIA
renderizada, CSS, I/O, rede, persistência, resolução de referências, colaboração, tema,
sanitização de HTML de terceiros.

**O core faz:** `A11yTree` determinística, roles semânticas, nomes, `alt`, `lang`/`dir`, ordem de
leitura, ranges e `validate_a11y`. O adaptador mapeia essa árvore para nativo/ARIA; o host audita
contraste/reflow/WCAG. A camada muda, a responsabilidade não.

Se uma mudança precisa de qualquer item de **“O core não faz”** para funcionar, ela está no crate
errado. Leia [`plan/00_panorama.md`](plan/00_panorama.md) §3,
[`plan/06_api-e-integracao.md`](plan/06_api-e-integracao.md) §12 e
[`plan/09_frontend-e-referencias.md`](plan/09_frontend-e-referencias.md), além do contrato de a11y
em [`plan/10_acessibilidade.md`](plan/10_acessibilidade.md).

## 2. Idioma

| Artefato | Idioma |
|---|---|
| `plan/`, `AGENTS.md`, `CHANGELOG.md` | português |
| Rustdoc, README público, `llms.txt`, `SKILL.md` | inglês |
| Identificadores de código (tipos, funções, módulos) | inglês |
| Mensagens de commit | inglês, Conventional Commits |
| Comentários de código | inglês, curtos e focados na intenção |

Não misture idiomas no mesmo artefato. Um comentário em português no `src/` é um erro de revisão.

## 3. Regras de ouro

1. **Vermelho = não terminou.** `make check` verde é a condição de concluir qualquer tarefa.
2. **Sem `unsafe` no core publicado.** `#![forbid(unsafe_code)]` no crate publicado; `testkit`,
   `fuzz`, `bench` e `crates/rich-editor-auy-wasm` são separados e podem usar `unsafe` de
   harness/glue com `SAFETY`. Nenhuma linha do core publicado contém `unsafe`.
2.1. **Publicação é gate.** O core é crate publicável; `cargo package`/`cargo publish --dry-run`,
   `deno publish --dry-run` e `composer validate --strict` são verificações, não passos opcionais.
3. **Sem `unwrap`/`expect`/`panic` em `src/` nem em testes.** Use `?`, `let else`, `matches!`,
   `assert!`, `unwrap_or`, `try_reserve`.
4. **Sem `HashMap`/`HashSet`/`Rc`/`Weak`/`RefCell`/`Cell`/`LinkedList` no core.** Ordem
   determinística e `Send + Sync`; use `Vec`, `BTreeMap`, `Arc`, `Box`.
5. **Sem relógio, ambiente, RNG ou processo.** Nada de `SystemTime::now`, `Instant::now`,
   `env::var`, `process::exit`. Tempo/aleatoriedade entram como dados.
6. **Sem I/O no core.** O consumidor lê/escreve; o core transforma valores.
7. **Sem float no wire.** `AttrValue` não tem `f64`; números canônicos são inteiros.
7a. **BLAKE3-256 é o hash canônico.** Fingerprint, envelope e integridade usam BLAKE3 com domain
    separation; FNV/SHA-256 não entram. `blake3` (`no_std`, `pure`) é a única dependência normal
    externa do core.
8. **Sem segunda fonte de verdade.** Schema, normalização, mapeamento e serialização têm um dono.
9. **Sem `build.rs`.** Tabelas/schemas são código versionado; geração só por ferramenta
   explícita, com resultado commitado e teste de paridade.
10. **Sem camada advisory.** Ou é tipo/erro de compilação, ou teste que falha, ou gate de CI.
11. **Arquivo de `src/` ≤ 300 linhas.** Fatie por responsabilidade.
12. **Toda exceção de lint tem `reason`.** Prefira `#[expect]` a `#[allow]`; nunca no workspace.
13. **Sem CSS, `class`, `style` ou wrapper da lib.** O adaptador mapeia `data-auy-*` + `A11yTree`
    para nativo/ARIA; o host é a raiz; `destroy()` restaura. Se precisa de folha de estilo, está
    fora do core.
14. **Sem segunda implementação do core em SDK.** PHP/TS transportam bytes e chamam WASM/CLI; schema,
    normalização, steps e fingerprint têm um dono (Rust).
15. **Acessibilidade universal é invariante.** Toda feature declara role, nome, `alt`, `lang`/`dir`,
    ordem e ranges; `Doc::a11y_tree` + `validate_a11y` entram no `make check`; não existe flag
    “a11y opcional” nem release sem a suíte verde.

## 4. Mapa do repositório (planejado)

```text
crates/rich-editor-auy/          # core: model, schema, pos, step, transform, state, history,
                                 # command, input, wire, math, motion, a11y, error
crates/rich-editor-auy-macros/   # schema! (proc-macro opcional)
crates/rich-editor-auy-wasm/     # wasm-bindgen + artefato WASM (não é o core)
crates/rich-editor-auy-html/     # pós-MVP: import/export + ImportReport
crates/rich-editor-auy-markdown/ # pós-MVP: import/export + ImportReport
crates/rich-editor-auy-collab/   # pós-MVP: OT/CRDT bridge
crates/rich-editor-auy-testkit/  # modelo ingênuo, strategies, alocador de contagem
crates/rich-editor-auy-refs/     # pós-MVP: corpus, resolve, backlinks, dangling
crates/rich-editor-auy-diff/     # pós-MVP: diff semântico entre docs persistidos
crates/rich-editor-auy-integrity/# pós-MVP: BLAKE3 + Ed25519 do Envelope
crates/rich-editor-auy-docx/     # pós-MVP: import/export Word com relatório
crates/rich-editor-auy-odf/      # pós-MVP: import/export ODF com relatório
packages/ts/                     # workspace Deno 2.0; publica @rich-editor-auy/* no JSR
packages/php/                    # pacote Composer rich-editor-auy/php
cli/                             # validate, format, diff, render opcional
plan/                            # visão, decisões, arquitetura, desempenho, rigor, roadmap
bench/                           # fora do workspace; criterion + iai-callgrind
fuzz/                            # fora do workspace; nightly
scripts/                          # check_file_length.sh, check_links.py, check_zero_style.sh, build_wasm.sh
```

Onde mexer:

| Quero… | Vá para |
|---|---|
| modelo, blocos, runs, normalização | `model/` |
| schema, expressões, marcas, attrs | `schema/` |
| posições, ranges, UTF-16/grapheme | `pos/`, `motion/` |
| steps, mapping, invert/rebase | `step/` |
| replace/split/join/wrap/lift | `transform/` |
| estado, transação, seleção, change set | `state/` |
| undo/redo, grupos | `history/` |
| comandos embutidos | `command/` |
| input rules, keymap | `input/` |
| JSON canônico, envelope, parse | `wire/` |
| árvore semântica, roles, nomes, `alt`, `lang`/`dir`, validação | `a11y/` |
| erros, limites | `error/` |
| IDs estáveis, `RefTarget`, `references()` | `model/`, `schema/`, `wire/` |
| corpus, resolução, backlinks | `rich-editor-auy-refs` (pós-MVP) |
| WASM, tipos TS, adaptador DOM | `crates/rich-editor-auy-wasm/`, `packages/ts/` |
| pacote PHP oficial | `packages/php/` |
| CLI validate/format/diff/render | `cli/` |
| macro `schema!` | `rich-editor-auy-macros` |
| medição, orçamento | `plan/05_desempenho.md`, `bench/` |
| rigor, CI, lints | `plan/07_rigor.md`, configs |

## 5. Fluxo de trabalho

1. **Leia o plano relevante.** Para modelo/editing, `plan/04_arquitetura.md`; para bytes,
   `plan/04` §8; para performance, `plan/05`; para rigor, `plan/07`; para integração PHP/TS,
   zero estilo e referências, `plan/09`; para a11y, `plan/10` — e aplique o gate de decisão dele.
2. **Confira a decisão.** Se a mudança toca uma decisão `🔴` ainda não ratificada, pare e registre
   a proposta em `plan/02_decisoes.md`; não implemente por conta própria.
3. **Escreva o teste primeiro** (quando for bug): o menor caso que reproduz. Para feature, escreva
   a propriedade/invariante que deve valer.
4. **Implemente a menor mudança** que satisfaz o contrato, sem “de passagem”.
5. **Rode o gate:** `make check`. Para hot path, rode também o bench de alocação/instruções.
6. **Atualize o registro:** decisão (se mudou), docs, `CHANGELOG.md`, e o checklist do roadmap.
7. **Não commite** até o gate estar verde. Se não puder rodar algo, diga o que faltou.

## 6. Contratos que não podem regredir

1. **Documento canônico.** Mesmo conteúdo + schema = mesmos bytes. Ordem de attrs/marcas é a do
   schema; sem `HashMap`, sem float, sem relógio.
2. **Transação atômica.** `apply` devolve estado novo ou erro; nunca deixa estado pela metade.
3. **`apply(invert(step))`** reproduz o doc para o subconjunto documentado; teste de propriedade.
4. **Normalização idempotente.** `normalize(normalize(d)) == normalize(d)`.
5. **Posição é byte UTF-8 + token estrutural.** Conversão para UTF-16/grapheme é explícita.
6. **Nenhum panic em input hostil.** JSON, step, slice, doc persistido: `Result`.
7. **Nenhuma importação silenciosamente lossy.** `ImportReport` obrigatório.
8. **Nenhum formato sem versão.** Envelope com `format`, `version`, `schema_fingerprint`.
9. **Nenhuma edição O(documento) no caminho comum.** Gate de alocação/visitas/instruções.
10. **Nenhuma API pública sem docs e teste.** `missing_docs = deny`; exemplos compilam.
11. **Zero estilo autoral.** O core não emite DOM; o adaptador não usa CSS, `class`, `style` ou
    wrapper; `destroy()` restaura o host. Se um teste de DOM encontrar estilo da lib, é regressão.
12. **IDs e referências preservados, não resolvidos.** `NodeId` sobrevive a split/join/undo; o core
    valida `RefTarget` e expõe `references()`, mas não conhece corpus, rede ou banco.
13. **A11y semântica é contrato.** `a11y_tree` preserva ordem/ranges/IDs/`lang`/`dir`/nomes/`alt`;
    `validate_a11y` falha fechado para `alt`/nome obrigatório; nenhuma feature entra sem teste de
    árvore + validação; `mount` não tem opt-out de semântica.

## 7. Validação

```sh
make check                 # fmt --check + clippy -D warnings + test + doc
                           # + file-length + links + zero-style + a11y + package
make check-ecosystem       # wasm + deno task check + composer validate/audit/test
cargo test --workspace --locked
cargo test -p rich-editor-auy --locked a11y
make a11y                  # gate dedicado; release sem ele é bloqueada
cargo +1.92 build --workspace --locked
cargo +1.92 test --workspace --lib --locked
cargo deny check
cargo audit
cargo machete
cargo +nightly miri test -p rich-editor-auy --test miri_smoke
cargo +nightly fuzz run json_parse -- -max_total_time=30
cargo +nightly fuzz run apply_step -- -max_total_time=30
cd packages/ts && deno task check && deno task publish:dry
cd packages/php && composer validate --strict && composer audit
```

Os alvos `deny`/`audit`/`machete`/`miri`/`fuzz` pulam com mensagem se a ferramenta não estiver
instalada; o CI não pula. `make check` não pula nada.

MSRV:

```sh
RUSTUP_TOOLCHAIN=1.92 cargo build --workspace --locked
RUSTUP_TOOLCHAIN=1.92 cargo test --workspace --lib --locked
```

`RUSTUP_TOOLCHAIN` é obrigatório nesses comandos: sem ele, o `rust-toolchain.toml` (1.99) vence e
o teste não prova a MSRV.

## 8. Testes

- **Unidade** em `src/**/tests.rs` com `#[cfg(test)] mod tests;`.
- **Integração** em `crates/*/tests/*.rs`; nomes descrevem comportamento, não método.
- **Sem `unwrap`/`expect`/`panic`.** Use `?` em teste que retorna `Result`, `let Some(x) = … else`,
  `matches!`, `assert!`.
- **Propriedade** para funções puras: normalize, mapping, serialize, apply/invert.
- **Diferencial** contra o modelo ingênuo do `testkit` para o subconjunto coberto.
- **Golden** byte-for-byte para wire e fingerprint; regeneração explícita, nunca automática no CI.
- **Fuzz** para parser e apply; um panic é falha de build.
- **Miri** smoke; **alocação** e **visitas** para o caminho quente.
- **Zero estilo** no adaptador: snapshot de DOM, grep de `style=`/`class=`/`<style`/`insertRule`,
  pacote sem `.css`; `destroy()` restaura o host.
- **A11y tree/validação**: golden de `a11y_tree`, um caso por `A11yCode`, ordem/ranges/`alt`/
  `lang`/`dir`; fuzz sem panic. Não é opcional.
- **A11y nos exemplos do adaptador**: `axe-core`, teclado, foco, IME e leitor de tela manual;
  axe verde não é certificado WCAG. O mapeamento de cada role da árvore é conferido no DOM.
- **Referências**: `NodeId` sobrevive a split/join/undo; `refs` resolve, dangling e backlinks com
  corpus de dois documentos; nenhum teste depende de rede.
- **Bindings**: PHP round-trip byte a byte; TS/WASM não reimplementa schema; CLI devolve `path`/
  `offset`. `deno task check`/`deno publish --dry-run`, `composer validate/audit`, PHPUnit e
  PHPStan são gates dos subprojetos oficiais.
- **Bench** com `iai-callgrind` para regressão; `criterion` é informativo.
- **Bug encontrado** ⇒ adicione o teste de regressão que o pega antes de fechar.

## 9. Documentação

- Todo item público tem rustdoc; `missing_docs = deny`; `broken_intra_doc_links = deny`.
- Doc de erro tem `# Errors`; doc de função que pode panic (não deveria) tem `# Panics`.
- Exemplos de rustdoc compilam; exemplos maiores ficam em `examples/`.
- Não prometa o que o core não garante: layout, ARIA/foco/contraste renderizados, performance de
  parede. O core **garante** a árvore semântica e a validação; o adaptador mapeia para nativo/ARIA;
  WCAG/contraste/leitor são auditoria do consumidor. axe verde não é conformidade.
- Mudança de API pública atualiza README, `CHANGELOG.md` e, se for instrução de uso, `SKILL.md`/
  `llms.txt` quando existirem.
- Link relativo quebrado é doc quebrada; rode o `check_links` quando houver docs suficientes.

## 10. Commits e PRs

- Conventional Commits em inglês: `feat(model): …`, `fix(step): …`, `docs(plan): …`,
  `test(state): …`, `chore(ci): …`.
- Um PR resolve **um** problema; refactor/cleanup separado.
- Descrição: problema, comportamento novo, como verificar, riscos. Sem inventário de arquivos.
- Não commite `target/`, `.knudge/` (se local-only), `Cargo.lock` de `fuzz/`/`bench/` se o
  workspace não versionar.
- Mudança de contrato de bytes/decisão exige entrada em `plan/03_decisoes-fechadas.md`.

## 11. Checklist antes de concluir

- [ ] `make check` verde, incluindo `a11y` e `package`.
- [ ] `make check-ecosystem` verde quando `packages/` existir.
- [ ] Teste novo cobre o comportamento (e o bug, se houve).
- [ ] Se mudou conteúdo/schema: `a11y_tree`/`validate_a11y` atualizados; role/nome/`alt`/`lang`/
      `dir`/ordem cobertos.
- [ ] Nenhum `unwrap`/`expect`/`panic`/`unsafe`/`HashMap`/`Rc` novo.
- [ ] Nenhum arquivo de `src/` acima de 300 linhas.
- [ ] Se hot path: alocação/instruções sem regressão.
- [ ] Se contrato de bytes: golden atualizado + decisão.
- [ ] Se API pública: docs, exemplo, `CHANGELOG`.
- [ ] Se schema: paridade/fingerprint e docs.
- [ ] Se dependência nova: justificativa + `cargo deny/machete`.
- [ ] Se API pública: `cargo package`/`cargo publish --dry-run` verdes.
- [ ] Se TS/WASM: `deno task check`/`deno publish --dry-run` verdes.
- [ ] Se PHP: `composer validate --strict`, `composer audit`, PHPUnit e PHPStan verdes.
- [ ] `plan/` e `AGENTS.md` atualizados se a fronteira mudou.

## 12. Referências

- Visão e não-objetivos: [`plan/00_panorama.md`](plan/00_panorama.md)
- Referências e lições: [`plan/01_referencias.md`](plan/01_referencias.md)
- Decisões abertas: [`plan/02_decisoes.md`](plan/02_decisoes.md)
- Decisões fechadas: [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md)
- Arquitetura: [`plan/04_arquitetura.md`](plan/04_arquitetura.md)
- Desempenho: [`plan/05_desempenho.md`](plan/05_desempenho.md)
- API/UI: [`plan/06_api-e-integracao.md`](plan/06_api-e-integracao.md)
- Rigor/CI: [`plan/07_rigor.md`](plan/07_rigor.md)
- Acessibilidade universal: [`plan/10_acessibilidade.md`](plan/10_acessibilidade.md)
- Roadmap: [`plan/08_roadmap.md`](plan/08_roadmap.md)
- Bordas: [`plan/brainstorm/02_bordas.md`](plan/brainstorm/02_bordas.md)
