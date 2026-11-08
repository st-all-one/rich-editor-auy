# 08 — Roadmap, marcos e critérios de parada

> **Regra.** Cada fase termina com `make check` verde e um aceite observável. Não existe "fase
> quase pronta". O MVP é F0–F4; formatos e colaboração são pós-MVP. Se um marco falhar, o projeto
> para ou corta escopo — não empurra o marco.

---

## 1. Corte do MVP

| Fase | Entrega | É MVP? |
|---|---|---|
| **F0** | Fundação: workspace, toolchain, lints, CI, `make check` | sim |
| **F1** | Modelo, schema, posições, normalização, JSON canônico | sim |
| **F2** | Steps, mapping, seleção, transação atômica, state | sim |
| **F3** | Comandos, input rules, histórico, IME, change set | sim |
| **F3.5** | Bindings: WASM/TS, adaptador DOM zero-CSS, SDK PHP | sim |
| **F4** | Performance: orçamentos, alocação, instruções, baseline | sim |
| F5 | HTML/Markdown + `refs` + CLI render + `ImportReport` | pós-MVP |
| F6 | Colaboração (OT/CRDT) | pós-MVP |

O MVP não tem UI visual, tema, CSS, layout, persistência ou colaboração. Ele tem o engine, um
adaptador DOM **neutro** (sem estilo) e bindings finos para TS e PHP; o consumidor estiliza e
persiste.

## 2. Fases

### F0 — Fundação (1–2 dias)

**Objetivo:** o repositório compila vazio com o rigor final, e o gate é real.

**Entregáveis:**

- `Cargo.toml` do workspace, `crates/rich-editor-auy/src/lib.rs` com `#![no_std]`,
  `extern crate alloc`, `#![forbid(unsafe_code)]`, `[lints] workspace = true`.
- Scaffolds vazios de `crates/rich-editor-auy-macros`, `crates/rich-editor-auy-wasm` e
  `crates/rich-editor-auy-testkit` para o workspace declarado fechar; o wasm só ganha API em F3.5.
- `rust-toolchain.toml` (1.99, `clippy`/`rustfmt`, target `wasm32-unknown-unknown`),
  `rustfmt.toml`, `clippy.toml` (valores de knudge adaptados à MSRV), `deny.toml` (política de
  html-auy + runtime HTTP/RNG negados), `Makefile`, `scripts/check_file_length.sh`,
  `scripts/check_links.py`, `scripts/check_zero_style.sh`.
- `.github/workflows/ci.yml` com `check`, `msrv`, `supply-chain`, `miri`, `fuzz`, `bench`; F3.5
  adiciona `wasm`, `js` e `php`.
- `fuzz/` e `bench/` com workspace próprio e alvos mínimos.
- `LICENSE-MIT`, `LICENSE-APACHE`, `README.md`, `CHANGELOG.md`, `AGENTS.md`.
- `tests/miri_smoke.rs` e `tests/smoke.rs` (um teste trivial que importa a API).

**Aceite:**

- `make check` verde (fmt, clippy, test, doc, file-length, package).
- `cargo build --workspace --no-default-features --locked` verde.
- `cargo +1.92 build --workspace --locked` e `cargo +1.92 test --workspace --lib --locked` verdes.
- `cargo deny check`, `cargo audit`, `cargo machete` verdes.
- `cargo doc -D warnings` verde.
- `cargo package -p rich-editor-auy --locked`, `-p rich-editor-auy-macros --locked` e
  `-p rich-editor-auy-wasm --locked` verdes; `cargo publish --dry-run` no release.
- `blake3` é a única dependência normal; `cargo tree -p rich-editor-auy` mostra o core + `blake3`
  (`no_std`, `pure`) e suas dependências puras.
- Nenhum `unsafe`, `unwrap`, `expect`, `panic`, `HashMap`, `Rc` no `src/`.

**Decisões:** F08–F25, F30–F32; D41–D47.

### F1 — Modelo e schema (2 semanas)

**Objetivo:** um documento válido, canônico e serializável, com posições resolvíveis.

**Entregáveis:**

- `schema`: `NodeSpec`, `MarkSpec`, `AttrSpec`, `SchemaBuilder`, grupos, expressões de conteúdo
  (NFA/DFA), checagem de produtividade, `excludes`/`inclusive`, fingerprint.
- `model`: `Doc`, `Block`, `TextBlock`, `Run`, `InlineItem`, `Attrs`, `MarkSet`, iteradores,
  normalização, `Limits`, `size`/`len`; `DocId`/`NodeId`/`VersionTag` e o atributo reservado `id`.
- `pos`: `Pos`, `Size`, `ResolvedPos`, `Range`, `NodeRange`.
- `wire`: writer canônico, parser JSON atrás de feature, `Envelope`, `ParseError`.
- `a11y`: `A11yRole`, `A11ySpec`, `A11yNode`, `A11yTree`; `Doc::a11y_tree` inicial; `lang`/`dir`/
  `alt` no schema; goldens.
- `error`: `SchemaError`, `LimitError`, `ParseError`, `InvariantError`.
- `tests`: golden de JSON, propriedades de normalização/round-trip, fuzz do parser, teste de
  memória, teste de limites, unicidade de `NodeId`, round-trip de referência e golden de a11y.

**Aceite:**

- Um schema básico (doc, paragraph, heading, text, strong, em, link, ref) constrói e rejeita
  schemas inválidos com erro estruturado.
- `Doc` construído a partir de JSON válido; `to_json` reproduz os mesmos bytes.
- `normalize` é idempotente (proptest) e não altera um doc já canônico.
- `validate` rejeita cada invariante I1–I12 com path/posição; I18/I19 são cobertos por
  `a11y_tree`/`validate_a11y`.
- `from_json` de 1 MB ≤ 15 ms e `to_json` ≤ 5 ms (alvos; calibrar).
- Memória de um doc de 1 MB dentro da fórmula.
- Fuzz do parser sem panic; Miri smoke verde.
- Nenhum `unwrap`/`panic` no caminho de parse.
- `NodeId` duplicado no mesmo doc falha; `id` sobrevive a `normalize` e `to_json`.
- `a11y_tree` de um doc básico preserva ordem, ranges, `lang`/`dir`, heading/lista e `alt`;
  `Schema` rejeita `A11ySpec` sem role ou com `alt` obrigatório impossível.
- `schema_fingerprint` é BLAKE3-256 de 64 hex e é idêntico em 32-bit, 64-bit e `wasm32`.
- Nenhum nó acessível depende de layout/cor; golden determinístico.

**Decisões:** D01–D15, D35–D37, D53, D58, D61.

### F2 — Transações (2 semanas)

**Objetivo:** editar o documento por uma álgebra pequena, invertível e mapeável.

**Entregáveis:**

- `step`: enum de 6 variantes, `Slice`, `StepMap`, `Mapping`, `apply`/`invert`/`map`.
- `transform`: `Transform`, `replace`, `replace_around`, `split`, `join`, `wrap`, `lift`,
  `add_mark`, `remove_mark`, `set_attr`.
- `state`: `EditorState`, `Transaction`, `Selection`, `ChangeSet`, `apply`, `apply_owned`.
- reference data (core): `AttrValue::Ref`, `RefTarget`, `references()`, `anchors()`; `set_node_id`.
- `pos`: `NodeRange`, `resolve`, `pos_before/after`.
- `tests`: propriedades de apply/invert/map, diferencial contra o modelo ingênuo, seleção mapeada,
  fuzz de steps, alocação, preservação da `A11yTree` e ranges de anúncio.

**Aceite:**

- `apply(invert(step))` reproduz o doc original para o subconjunto documentado (proptest).
- `Mapping` mapeia seleção e posições sem sair do doc; `assoc` testado nas bordas.
- Uma transação com falha no meio não altera o estado (`apply`) e restaura o estado
  (`apply_owned`), coberto por teste.
- 10.000 operações aleatórias no modelo real e no ingênuo produzem o mesmo JSON.
- Fuzz de `apply_step` sem panic; nenhum doc inválido após apply.
- Inserir 1 char em S2 dentro do orçamento; zero alocação no caso de capacidade.
- Split/join respeitam a política de IDs; `RefTarget` round-trip e `references()` encontram o alvo.
- `a11y_tree(doc)` antes/depois de apply preserva a ordem de leitura e os ranges; `ChangeSet` tem
  range antigo/novo suficiente para anúncio, sem depender de hash/relógio.

**Decisões:** D16–D26, D54–D55.

### F3 — Edição utilizável (1–2 semanas)

**Objetivo:** o consumidor consegue implementar um editor de verdade em cima do core.

**Entregáveis:**

- `command`: insert/delete/split/join/wrap/lift/marks/block type/select all.
- `input`: input rules literais (`# `, `> `, `- `, `1. `, `**`), `Key`/`Keymap` declarativo.
- `history`: undo/redo, grupos, orçamento, `can_undo`/`can_redo`.
- `state`: composição de IME, `stored_marks`, `ChangeSet` com ranges.
- `a11y`: `Doc::a11y_tree` completa, `validate_a11y`, `A11yCode`/`A11yIssue`/`A11yReport`,
  nomes/`alt`/`lang`/`dir`, tabela/cabeçalho; comando/CLI `validate --a11y`. A tabela aqui é o
  schema/preset de tabela no modelo genérico; a UX de célula/spans/seleção de tabela é pós-MVP.
- `testkit`: harness headless que simula eventos e verifica o doc **e a árvore semântica**.
- `examples/headless`: um binário de exemplo (fora do crate publicado) que edita e serializa.

**Aceite:**

- Matriz de comandos: cada comando tem teste de seleção vazia, seleção parcial, seleção entre
  blocos, borda de bloco e doc vazio.
- Undo/redo restaura os bytes exatos em 10.000 operações aleatórias.
- Uma composição de IME vira **um** grupo de undo; cancelar restaura o texto original.
- Input rules não disparam dentro de código literal (se o schema tiver code block) e não disparam
  no meio de uma palavra.
- O harness headless renderiza blocos/runs/ranges de um doc e aplica uma sequência de eventos sem
  panic.
- Nenhum comando depende de relógio, ambiente ou ordem de hash.
- `validate_a11y` cobre todos os `A11yCode` com severidade e `path`/`range`; `a11y_tree` é golden
  e determinística; `make check` inclui o alvo `a11y`.

**Decisões:** D27–D34, D62.

### F3.5 — Bindings e zero estilo (2 semanas)

**Objetivo:** provar que a integração é fácil e que a lib não interfere na estilização.

**Entregáveis:**

- `crates/rich-editor-auy-wasm`: crate `cdylib`/`rlib` com `wasm-bindgen`; API síncrona; sem DOM;
  artefato `.wasm` copiado para o pacote JSR. **Não é o core publicado**; glue pode ter `unsafe`
  local com `SAFETY`, e o core continua `forbid(unsafe_code)`.
- `scripts/build_wasm.sh`: `cargo build --target wasm32-unknown-unknown --release`, `wasm-bindgen
  --target web`, `wasm-opt -Oz`, cópia para `packages/ts/packages/core/wasm/` e checksum do
  artefato; roda no CI `wasm` e no release.
- `packages/ts`: workspace Deno 2.0 (JSR) com `@rich-editor-auy/core`,
  `@rich-editor-auy/schema` e `@rich-editor-auy/dom`; `deno.json` por pacote; tipos estritos;
  `mount`/`bind`/`data`.
- `packages/ts/packages/dom`: render **semântico nativo** por default (`neutral` opt-in),
  mapeamento da `A11yTree`, input, IME, seleção, clipboard, `destroy()`; zero
  CSS/`class`/`style`/wrapper; sem flag que desligue semântica.
- `packages/php`: pacote Composer `rich-editor-auy/php`; envelope, bytes preservados, validador via
  CLI; `composer.json` sem dependência de runtime; PHPUnit/PHPStan no CI.
- `cli/`: `validate`/`format`/`diff` mínimo; base para PHP e CI.
- `examples/`: TS/Deno vanilla, PHP round-trip, host com CSS do consumidor.
- gate de estilo: snapshot de DOM, grep de `style=`/`class=`/`<style`/`insertRule`, pacote JSR sem
  `.css`.
- testes de a11y: `axe-core`, teclado, foco, IME; mapeamento da `A11yTree` conferido no DOM.

**Aceite:**

- Quick start TS/Deno em ≤ 20 linhas, sem import de CSS, com host do consumidor.
- `mount` não cria wrapper, não usa `class`/`style`, não remove `outline`; `destroy()` restaura.
- `deno task check` verde (`fmt --check`, `lint`, `check`, `test`); `deno publish --dry-run` sem
  erro; pacote JSR não contém `.css` nem exporta estilo.
- `wasm32-unknown-unknown` compila com `--locked`; o artefato é reprodutível e cabe no orçamento
  acordado (medido, não “parece pequeno”).
- PHP: `fromBytes($bytes)->bytes() === $bytes` byte a byte; envelope incompatível falha tipado;
  `composer validate --strict`, `composer audit`, PHPUnit e PHPStan verdes.
- CLI valida documento inválido com `path`/`offset`, sem panic.
- `axe-core` verde nos exemplos; teclado e IME cobertos.
- `a11y_tree`/`validate_a11y` verdes no corpus; cada nó renderizado tem role/nome/`alt` coerente
  com a árvore; não existe `mount({ aria: false })`.
- Nenhum schema/steps reimplementado em TS/PHP; o Rust continua sendo a autoridade.

**Decisões:** D51, D52, D57–D58, D61–D62; D53/D54 entram em F1/F2; D56 em F5; F30–F32.

### F4 — Performance e confiança (1 semana)

**Objetivo:** provar os orçamentos e travar regressões.

**Entregáveis:**

- `bench/` com cenários S0–S4, `criterion` e `iai-callgrind`.
- `tests/alloc.rs` com alocador de contagem; `tests/visits.rs` com contador de visitas.
- `tests/memory.rs` com orçamento de memória.
- fuzz de `a11y_tree`/`validate_a11y` sem panic; corpus a11y no gate de release.
- baseline de instruções versionado; CI com `bench`.
- decisão sobre `ChunkedVec` (D06) com base nas medições.
- relatório `bench/ULTIMO.md` com os números e a calibração.

**Aceite:**

- Todos os alvos de [`05_desempenho.md`](05_desempenho.md) atingidos **ou** uma mudança de desenho
  registrada (ex.: chunking) com novo alvo.
- Regressão de instruções > 5% falha o CI.
- Alocação no caminho quente dentro do orçamento.
- Memória dentro da fórmula.
- `make check` verde do zero; MSRV, Miri, fuzz e supply chain verdes.
- `make a11y` verde no corpus; nenhuma release sem `a11y_tree`/`validate_a11y`.

**Decisões:** D06–D08 (chunking/binário); calibração.

### F5 — Interop, referências e SSR (pós-MVP, 2–3 semanas)

- `rich-editor-auy-html`: import/export, `parse_dom`/`to_dom` no schema, `ImportReport`.
- `rich-editor-auy-markdown`: import/export, `ImportReport`.
- `rich-editor-auy-refs`: corpus, `resolve`, `backlinks`, `dangling`, `version_mismatch`,
  `CorpusStore` (sem rede/DB).
- CLI `render`: HTML com `SanitizePolicy` explícita; `validate`, `format`, `diff`.
- Testes de round-trip com perdas declaradas; fuzz dos parsers; nenhum panic.
- A UI ganha colagem HTML/Markdown com fallback para texto puro; `refs` tem demo de duas
  publicações.

### F5.1 — Extensões de escopo (ratificadas, implementação pós-MVP)

- **D63 — Word/ODF:** `-docx`/`-odf` separados, com `ImportReport`/`ExportReport`; não entram no
  core e não prometem fidelidade de estilo.
- **D64 — integridade:** crate `integrity` sobre o `Envelope` canônico; **BLAKE3-256 + Ed25519**;
  chaves/política no host. O fingerprint de schema (D13) também é BLAKE3, não FNV nem SHA-256.
- **D65 — diff/auditoria:** crate `diff` semântico por `NodeId`/caminho e CLI que agrega
  `validate`/`a11y`/`diff`; `ChangeSet` continua por steps.

Essas extensões não são pré-requisito de F1–F4.

### F6 — Colaboração (pós-MVP, design primeiro)

- Documento de desenho de rebase/OT/CRDT; escolha de transporte; `rich-editor-auy-collab`.
- Steps serializáveis e mapeáveis já existem; o crate novo não muda o core.
- Testes de convergência contra um par de réplicas; sem IDs de CRDT no doc canônico.
- Só começa depois de F4 e de uma necessidade real de consumidor.

## 3. Marcos e critérios numéricos

### M1 — "o modelo e a álgebra funcionam" (fim de F2)

1. 1 MB / 10k blocos: inserir 1 char ≤ **200 ns** no caminho owned e ≤ **10 µs** no caminho com
   snapshot.
2. `apply(invert(step)) == doc` para 100.000 operações geradas.
3. `to_json(from_json(doc)) == to_json(doc)` byte-for-byte para 10.000 docs gerados.
4. Fuzz de parse/apply sem panic por 30 s em CI.
5. Alocação ≤ 1 por edição de 1 char no caso comum.
6. Memória ≤ fórmula de [`05`](05_desempenho.md).

**Se falhar:** parar e cortar escopo (ex.: remover `ReplaceAround` do MVP, simplificar o schema)
antes de seguir. Não mascarar com `apply` mais lento.

### M2 — "dá para construir um editor" (fim de F3)

1. 20 comandos cobertos pela matriz de aceite.
2. 10.000 operações aleatórias sem divergência entre modelo real e ingênuo.
3. Undo/redo restaura os bytes exatos em 10.000 operações.
4. IME: uma composição = um grupo de undo; cancelar restaura.
5. Harness headless aplica 1.000 eventos de UI simulados sem panic e com seleção válida.
6. Nenhum comando depende de relógio/ambiente/hash.
7. `a11y_tree` + `validate_a11y` verdes; nenhuma mudança de comando altera a ordem de leitura sem
   teste.

**Se falhar:** o problema é de API/comandos, não de performance; revisar D30–D34 antes de seguir.

### M2.5 — "dá para integrar sem CSS" (fim de F3.5)

1. Quick start TS/Deno ≤ 20 linhas, sem import de CSS; PHP round-trip byte a byte.
2. `mount` sem wrapper, `class`, `style`, `<style>` ou `insertRule`; `destroy()` restaura o host.
3. `axe-core` verde nos exemplos; teclado, foco e IME cobertos.
4. Nenhum schema/steps em TS/PHP; validação estrita via WASM/CLI.
5. `deno task check` e `deno publish --dry-run` verdes; `composer validate --strict`,
   `composer audit`, PHPUnit e PHPStan verdes.
6. `testkit` com corpus mínimo resolve dois documentos, com dangling e backlinks; o crate `refs`
   completo é F5.
7. Cada nó renderizado tem role/nome/`alt` coerente com a `A11yTree`; não há opt-out de semântica.
8. `neutral` é opt-in, testado com leitor; o default é nativo.

**Se falhar:** cortar extras/`neutral`, não o `mount` semântico nem a `A11yTree`; `data`/`bind`
transferem responsabilidade explicitamente. Não compensar com CSS.

### M3 — "confiável para publicar 0.1" (fim de F4)

1. Todos os gates de F4 verdes.
2. `make check` verde em clone limpo.
3. MSRV 1.92, Miri, fuzz, `cargo deny/audit/machete` verdes.
4. Documentação pública completa; exemplos compilam.
5. Nenhum kill criterion acionado.
6. Changelog e `AGENTS.md` atualizados.
7. `make a11y` verde; `a11y_tree`/`validate_a11y` no corpus e no gate de release.

## 4. Critérios de abandono (kill criteria)

O projeto para — e o que existe vira registro/vendor — se qualquer um ocorrer:

1. **Performance:** uma edição de 1 caractere em 1 MB não couber em 10 µs (owned) ou exigir mais
   de 3× o texto em memória, mesmo após chunking/runs limitados.
2. **Correção:** o modelo não conseguir garantir `apply(invert(step)) == identidade` para o
   subconjunto documentado, com o teste diferencial verde.
3. **Complexidade:** o schema macro custar mais LOC/manutenção do que o builder, sem ganho de
   segurança demonstrável.
4. **Fronteira:** o core precisar de DOM, layout, I/O ou APIs do SO para funcionar.
5. **Manutenção:** o custo do parser/JSON/normalização superar o custo dos defeitos que ele evita
   em dois ciclos de `rustc`.
6. **Escopo:** o projeto virar framework de UI/colaboração/servidor por pressão de features, sem
   uma decisão explícita que aceite o novo escopo.

## 5. Riscos e mitigação

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| Overengineering do schema/expressões | média | alto | começar com a gramática mínima; testar com schemas reais; cortar se não pagar |
| Unicode (grafemas, bidi, IME) | alta | médio | separar `motion` do modelo; `snap_to_grapheme`; testes de borda; não prometer layout |
| Autofill/expressões recursivas | média | alto | checagem de produtividade na construção; teste de schema inválido |
| COW estourar em documentos extremos | média | médio | medir S4; chunking só se necessário; orçamento explícito |
| Parser JSON próprio com bug | média | alto | fuzz + diferencial contra `serde_json` + limites + erros com offset |
| API pública instável | alta | médio | 0.x; `#[non_exhaustive]` em erros; documentar contratos; não expor internals |
| IME desincronizado | média | alto | protocolo explícito; harness de composição; testes de cancelamento |
| Escopo virar UI/framework | alta | alto | F26/F28; gate de zero CSS; `data`/`bind` como caminho de controle |
| Adaptador DOM virar UI kit | alta | alto | `mount` mínimo; sem `class`/`style`/wrapper; `data` é o caminho de controle total |
| A11y do adaptador | média | alto | `A11yTree`/`validate_a11y` no core; default nativo; axe + teclado + leitores; `neutral` opt-in testado; não prometer WCAG |
| PHP divergir do core | média | alto | F28; PHP só envelope/bytes/CLI; nenhum schema/steps em PHP |
| IDs sem migração | média | alto | D53 em F1; IDs opcionais; `set_node_id`; demo de corpus em F3.5 |
| Colaboração contaminar o core | média | alto | steps como fronteira; crate separado; sem IDs de CRDT no doc |
| Performance de parede flaky | alta | baixo | gates determinísticos (alocação/instruções); bench de parede informativo |
| Manutenção do parser/JSON | média | alto | parser opcional; fallback serde; fuzz; orçamento de LOC |
| Tempo do mantenedor | alta | alto | fases curtas; marcos com kill criteria; cortar pós-MVP sem culpa |

## 6. Orçamento de LOC

| Peça | MVP | Pós-MVP | Observação |
|---|---:|---:|---|
| `model` + `schema` + `pos` | 3.800 | — | inclui normalização e limites |
| `step` + `transform` + `state` | 3.300 | — | inclui mapping e seleção |
| `history` + `command` + `input` | 1.400 | — | comandos mínimos |
| `wire` + `error` + `math` + `motion` | 2.000 | — | JSON writer/parser, erros, unidades |
| `a11y` | 400 | — | árvore, roles, nomes, validação |
| **core MVP** | **~10.900** | — | produção, sem testes |
| `macros` | 800 | — | `schema!` + acessores |
| `testkit` | 800 | — | modelo ingênuo, strategies, alocador |
| testes (integração/prop/golden/fuzz) | 5.000 | 2.000 | |
| `bench` | 500 | 300 | |
| `cli` + scripts de gate | 1.000 | — | validate/format/diff, check_links, zero-style |
| `crates/rich-editor-auy-wasm` + tipos TS | 800 | — | wasm-bindgen, tipos, erros |
| `packages/ts/packages/dom` | 1.500 | — | render semântico, mapeamento `A11yTree`, input, zero CSS |
| `packages/php` | 400 | — | envelope, bytes, CLI, Composer |
| testes de integração/a11y/estilo | 1.000 | — | axe, teclado, snapshot |
| `refs` | — | 1.200 | corpus, resolve, backlinks, dangling |
| `html` / `markdown` | — | 2.500 | parsers + relatórios |
| `collab` | — | 4.000 | design + implementação |
| **total aproximado** | **~22.300** | **~32.300** | |

O taino-edit core tem 6.667 LOC; o nosso core é maior porque inclui `no_std`, wire próprio,
erros estruturados, limites, normalização canônica e instrumentação. Se o core passar de ~12.000
LOC sem entregar o M2, o escopo está errado.

## 7. Matriz de aceite por artefato

| Artefato | Entrada | Saída | Erro | Invariantes | Teste |
|---|---|---|---|---|---|
| `SchemaBuilder` | declaração de nós/marcas/attrs | `Schema` + fingerprint | `SchemaError` com path/nome | ids, grupos, produtividade, attrs | unidade + trybuild (macro) |
| `Doc::from_json` | `&str` | `Doc` | `ParseError` com offset/path | I1–I12 | proptest + fuzz + golden |
| `Doc::to_json` | `&Doc` | bytes canônicos | `LimitError` | I11, ordem | golden byte-for-byte |
| `normalize` | doc + região | doc canônico | `InvariantError` | I3–I10, I12 | proptest de idempotência |
| `validate` | `&Doc` | `Ok`/`Vec<Violation>` | violação com path | I1–I12 | unidade por invariante |
| `Step::apply` | doc + step | doc' + `StepMap` | `StepError` | I13, atomicidade | proptest + fuzz |
| `Step::invert` | doc antes + step | step inverso | `StepError` | I14 | proptest |
| `Step::map` | step + `Mapping` | step' ou `None` | — | I15 | proptest |
| `Selection::map` | seleção + `Mapping` | seleção' | — | nunca fora do doc | proptest de bordas |
| `Transaction` | comandos | steps | `StepError` | seleção/ordem | unidade |
| `apply` | estado + tx | `Applied` | `ApplyError` | atomicidade, I13 | proptest + fuzz |
| `History` | applied | undo/redo | `ApplyError` | bytes exatos | stress 10k |
| `Command` | estado | `Transaction` ou `None` | — | matriz de seleção | tabela de testes |
| `InputRule` | texto antes do caret | `Transaction` | — | prefixo, não no meio de palavra | unidade |
| IME | `CompositionEvent` | `Applied` | `ApplyError` | um grupo de undo | harness |
| `Envelope` | schema + doc | bytes | `ParseError`/`SchemaMismatch` | fingerprint/versão | golden |
| `ImportReport` (pós) | HTML/MD | doc + relatório | `ParseError` | sem perda silenciosa | golden + fuzz |

## 8. Ordem de execução e dependências

```text
F0 ──► F1 ──► F2 ──► F3 ──► F3.5 ──► F4
                                  │
                                  ├──► F5 (pós-MVP)
                                  └──► F6 (pós-MVP, design)
```

- F0 materializa o workspace, `blake3`, o toolchain 1.99 + `wasm32-unknown-unknown` e os gates.
- F1 começa após F0; D01–D15, D35–D37, D53, D58 e D61 estão ratificadas (03 §C).
- F2 começa após F1 verde; D16–D19 e D54–D55 estão ratificadas.
- F3 começa após F2 verde; D27–D34 e D62 estão ratificadas, exceto D28/D34 (adiadas).
- F3.5 começa após F3 verde; D51–D52, D56–D61 estão ratificadas; D56 é F5.
- F4 começa após F3.5 verde; mede o que F1–F3.5 construíram.
- F5/F6 não começam sem F4 e sem uma necessidade de consumidor; D63–D65 são pós-MVP.

## 9. Definition of Done (por PR)

- [ ] O comportamento novo/changed tem teste no nível correto (unidade, prop, diferencial, golden,
      fuzz).
- [ ] `make check` verde, incluindo `a11y`.
- [ ] Se mudou conteúdo/schema: `a11y_tree` + `validate_a11y` atualizados; role/nome/`alt`/`lang`/
      `dir`/ordem cobertos.
- [ ] Nenhum `unwrap`/`expect`/`panic`/`unsafe`/`HashMap`/`Rc` novo.
- [ ] Nenhum arquivo de `src/` acima de 300 linhas.
- [ ] Se tocou hot path: bench de alocação/instruções sem regressão.
- [ ] Se mudou contrato de bytes: golden atualizado e decisão registrada.
- [ ] Se mudou API pública: docs, exemplo e `CHANGELOG`.
- [ ] Se mudou schema: teste de paridade/fingerprint e docs.
- [ ] Se mudou uma decisão fechada: nova entrada em `03_decisoes-fechadas.md`.
- [ ] Se adicionou dependência: justificativa e `cargo deny/machete`.
- [ ] Se tocou API pública: `cargo package`/`cargo publish --dry-run` verdes; docs.rs builda.
- [ ] Se tocou TS/WASM: `deno task check` e `deno publish --dry-run` verdes.
- [ ] Se tocou PHP: `composer validate --strict`, `composer audit`, PHPUnit e PHPStan verdes.
