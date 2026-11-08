# taino-edit — revisão independente

> Revisor: rodada extra, 2026-11-07. Escopo: `dossie/taino-edit/` contra
> `ref/taino-edit/` (v0.7.0, commit `90ba5ed`, clone shallow de 1 commit,
> `git status` limpo). Nada em `ref/`, `plan/` ou em outros dossiês foi tocado.
> Artefatos de build em `/tmp/taino-target`; benchmark reexecutado em
> `/tmp/taino-bench2` (cópia do harness, sem tocar no original).

## 0. Resumo

O dossiê estava **substancialmente correto no que importa**: o modelo Rust, a
separação core/dom/adapter, os steps invertíveis, os limites do adapter e a
matriz de 85% sobrevivem à auditoria. Os erros encontrados são de **contagem e
de citação**, não de tese: número de extensões, LOC por camada, versões no
CHANGELOG, commits do clone, questões abertas de colaboração, contagem de testes
e a procedência do JSON do exemplo. Também havia uma inconsistência de
documentação do próprio projeto (README diz 214 testes; o comando dá 212) que o
dossiê não registrava.

Confiança final: **85%** — core 92%, adapter/a11y 78%. O que impede 90%+ é não
ter rodado nenhum teste de browser.

## 1. O que foi reexecutado (e a saída)

| Verificação | Resultado |
| --- | --- |
| `cargo test -p taino-edit-core -p taino-edit-extensions` | **212 passed**, 0 failed (soma de todos os `test result`; 208 `#[test]` + 4 doc-tests) |
| o mesmo com `--all-features` | **212 passed** — a feature `collab` é vazia, não adiciona testes |
| `cargo test -p taino-edit-dom --lib` | **6 passed** |
| `cargo run -p headless-core -q` | saída idêntica à do dossiê (`HiHello world`, JSON 221 bytes, HTML/undo) |
| `cd dossie/taino-edit/exemplos && cargo run -q` | saída bate com `exemplos/README.md` **exceto** o JSON, que estava truncado no README (corrigido) |
| benchmark (`/tmp/taino-bench2`, release, melhor de 3) | 100→**39 µs**, 1 000→**29 µs**, 5 000→**94 µs**, 10 000→**198 µs**, 20 000→**565 µs** (reexecução do harness original: 191 e 862 µs) |
| `diff -q` dos 6 arquivos de `codigo/` contra as origens | **idênticos**, byte a byte |
| `find` por `*.css`/`*.scss` no repo | **nenhum** |
| `grep` por `role=`/`aria-` em `crates/` | **nenhum** |
| `git rev-list --count HEAD` / `.git/shallow` | **1 commit** (o dossiê dizia 4) |
| `wc -l` por `crates/*/src` | core 6 667, dom 1 685, extensions 3 050, leptos 496, dioxus 571, table-view 146; `src/` total 12 659; `*.rs` total 23 732 |

## 2. Achados (e o que foi feito)

### Corrigidos nos docs

1. **Contagem de extensões (erro factual).** O dossiê dizia "13 extensões
   embutidas" e "12 vs 13 reais". O código tem **14** `impl Extension for`
   (`align, blockquote, bold, code_block, code_mark, heading, history, image,
   italic, link, lists, paragraph, table, transform_case`); o demo
   `basic-leptos` compõe **13** (não usa `TransformCase`). O README upstream é
   auto-inconsistente: "Twelve built-in extensions" (l.118) e "13 built-in
   extensions" (l.39/144). Corrigido em `README.md` §1/§6, `avaliacao.md`
   (extensibilidade) e mantida a contagem de 13 onde ela é sobre o demo.
2. **"212 testes" era ambíguo.** São 208 `#[test]` + 4 doc-tests. Registrado em
   `README.md` §2 e `avaliacao.md` §1. Aproveitei para registrar que o README
   do projeto anuncia **214** (`README.md:39`) e que não reproduzi esse número
   por nenhum caminho (nem com `--all-features`).
3. **LOC errados em `arquitetura.md`.** Dizia core ~5 900 (real 6 667), dom
   ~1 900 (real 1 685), extensions ~3 400 (real 3 050). Tabela corrigida; a soma
   de `src/` (12 659) e o total com testes (23 732) agora aparecem separados.
4. **Citações de `DESIGN_NOTES.md` fora de lugar.** `:104-108` apontava para
   "Reusable building blocks", não para a regra de que `core` não depende de
   web; o correto é `:28-38`. O orçamento de LOC está em `:62-68`, não `:52-88`.
   `deny(unsafe_code)` está em `:127`, não `:122-130`. Corrigido.
5. **`#![deny(unsafe_code)]` não está em "todos exceto dom".** Está em 5 dos 7
   crates (core, extensions, dioxus, table-view, umbrella); **`leptos` também
   não declara o lint** (apenas não usa `unsafe`). `avaliacao.md` ajustado para
   dizer a verdade sem inflar a disciplina.
6. **Questões abertas de colaboração: 5, não 8.** `COLLAB_DESIGN.md §8` tem
   Q1–Q5 (granularidade de átomos, seed, rota de undo, MSRV do loro, nome da
   feature). Corrigido em `arquitetura.md` §9 e `avaliacao.md` (documentação).
7. **CHANGELOG tem 11 releases (0.1.0→0.7.0), não 7.** Corrigido na matriz
   (item 13).
8. **Clone shallow tem 1 commit, não 4.** Corrigido no cabeçalho do `README.md`
   e na matriz.
9. **JSON do exemplo estava truncado e fora de ordem.** O `exemplos/README.md`
   anunciava "capturada integralmente", mas o bloco começava em `"content"`
   omitindo o `heading`. Substituído pela saída real (com `"level": 1` nos attrs
   do heading e `"text_align": null` nos dois blocos) e a nota de procedência
   agora diz que foi reexecutada.
10. **Benchmark tratado como constante.** "27 µs" e "825 µs" são uma amostra:
    a reexecução deu 39 µs (100 blocos) e 565–862 µs (20 000). `README.md` §5,
    §3.9 e `avaliacao.md` agora mostram faixas e dizem que os extremos variam.
    O harness original, aliás, hoje só roda 10k/20k — por isso a varredura
    completa foi feita numa cópia (`/tmp/taino-bench2`).
11. **Citações de linha pontuais.** `view.rs:103-106` → `102-106`;
    `table.rs:28-31` → `22-27`; `bold.rs:1-34` → `1-36`;
    `table-view/src/lib.rs:32` → `33`; `ROADMAP.md:305-315` → `310` para WCAG;
    `leptos/src/lib.rs:229-420` → `229-448` para a lista de listeners.
12. **Exemplo "em ~30 linhas"** → o `examples/headless-core` tem 121 linhas e o
    exemplo do dossiê, 129. Corrigido.
13. **Contagem de evidências da matriz.** "6 com execução direta e 7 por
    código/teste/doc" não fechava: com `E` aparecem 8 itens (1, 2, 3, 5, 6, 7,
    11, 13). Corrigido.

### Achados novos (o explorador não viu) — registrados, sem mudar nota

- **Doc do `read_dom_changes` está desatualizado no upstream**: o rustdoc diz
  que os adapters ligam a leitura "behind a `MutationObserver`
  (`dom/src/view.rs:456-457`), mas ambos usam o evento `input`
  (`leptos/src/lib.rs:277-287`). Não existe `MutationObserver` no wiring. É
  evidência de maturidade, não de arquitetura — anotado em `arquitetura.md` §7.2.
- **`ssr-leptos` estiliza `.taino-editor` num `<style>`**
  (`examples/ssr-leptos/src/lib.rs:161-164`): prova que a classe fixa é o único
  gancho de tema, reforçando a ressalva do "zero CSS".
- **Classe fixa confirmada por API**: `TainoEditor` expõe só `state`, `plugins`
  e `keymap` (`leptos/src/lib.rs:71-91`) — não há prop de classe, `aria-*` ou
  `spellcheck`.
- **Nenhum listener de `copy`/`cut`/`beforeinput`/`dragstart`/`dragover`/`drop`
  nos dois adapters** — confirmado por grep; a lista real é `input`, `keydown`,
  `compositionstart/end`, `paste`, `mousedown/mousemove/mouseup`,
  `selectionchange`, `scroll`, `resize`.
- **Tabelas não têm keymap default**: `table.rs:22-27` diz explicitamente "No
  default keymap", consistente com o dossiê.

## 3. Pontos que resistiram à auditoria

Estes foram checados e estão certos (mantidos como estavam, salvo linha):

- **Enter/Backspace/Delete mortos sem keymap**: `leptos/src/lib.rs:336-339`
  confere; `base_keymap` tem 8 bindings (`keymap.rs:218-242`).
- **Sem stored marks**: `commands.rs:99-101` (`return false; // v0.1: no
  stored-marks on an empty caret`).
- **`read_dom_changes` só vê o primeiro text run**: doc literal em
  `view.rs:455-456`; implementação `464-519`.
- **Copy/cut/DnD não ligados**; primitivas existem (`view.rs:589-605`) e há
  testes de browser (`tests/drag_drop.rs`, 4 casos), não executados.
- **A11y mínima**: `contenteditable` + `tabindex` e nada mais; zero `aria-`/
  `role` nos crates (grep vazio); toolbars dos exemplos têm `role="toolbar"`
  sem `aria-pressed` (`examples/basic-leptos/src/main.rs:300-340`).
- **Zero CSS com ressalva**: nenhum `.css`; mas `style="text-align: …"`
  (`align.rs:50-60`) e `style="width: …px"` (`table.rs:87`) vão para o
  documento, e as classes `taino-editor`/`taino-cell-selected`/
  `taino-deco-layer` são fixas. A nota de neutralidade **3** é justa.
- **Histórico clonado por transação**: `state.rs:190` literal.
- **`node_size` O(n)**: `node.rs:189-197`, `chars().count()` sem cache.
- **Marks por nó declaradas e não aplicadas**: `schema.rs:29-40`; `inclusive`
  também não interpretado (`schema.rs:56-63`).
- **`map_against` documentado e ausente**: só menção em `step.rs:33`.
- **`panic!` em modificador desconhecido**: `keymap.rs:124`.
- **Licença MIT OR Apache-2.0**, MSRV 1.80, `rust-toolchain.toml` com wasm32,
  CI fmt/clippy/test/doc + job SSR — todos conferem.
- **`codigo/INDEX.md`**: aponta 6 arquivos que existem, com contagem de linha
  correta e origem exata; reconfirmado por `diff -q`.

## 4. Notas da rubrica

Auditadas uma a uma; **nenhuma foi alterada**, porque todas sobrevivem à
evidência: usabilidade **3**, API **4**, neutralidade de estilo **3**, modelo
**5**, extensibilidade **4**, performance **2**, acessibilidade **1**,
documentação **5**, manutenção **4**, adequação **3**. Média 3,4 (conferida).

Único reparo de justificativa: a nota 5 de documentação continua defensável
(`DESIGN_NOTES`, `ROADMAP`, `COLLAB_DESIGN`, doc-tests), mas o README do projeto
tem números errados (214 testes, 12/13 extensões) — o próprio dossiê já
registrava isso; agora registra com precisão.

## 5. Cobertura da matriz de 85%

13/13 itens com evidência, sendo 8 com execução direta. O número **não** deve
ser lido como "experiência de browser verificada": os itens 4 (seleção/IME/
clipboard) e 8–9 (CSS/a11y) são leitura de código + testes não executados.
Cobertura realista: **~85%** com essa ressalva explícita; sem ela, seria
inflação.

## 6. Veredito (separado, como pedido)

- **Core Rust (`taino-edit-core`): próximo do nosso caso.** Modelo tipado com
  `Arc`, schema com DFA, steps invertíveis com `Mapping` mirror/recover,
  comandos `probe`/`dispatch`, keymap encadeado, serialização tripla e contrato
  SSR. Roda sem DOM — reexecutado. Vale **copiar conceitualmente** e adaptar
  quatro pontos: cache de comprimento de texto, histórico sem clone por
  transação, inversão preguiçosa, e aplicação real da expressão de marks.
- **Adapter Leptos/Dioxus + a11y: imaturo.** Enter/Backspace/Delete dependem de
  keymap e são `prevent_default` sempre; leitura de DOM só de texto divergente;
  copy/cut/DnD sem listener; zero `role`/`aria-*`; `style` inline no documento;
  classe fixa sem prop; wiring duplicado (496 vs 571 linhas). Não copiar como
  está: o custo de a11y e de reconciliação é nosso.

## 7. Confiança

| Eixo | Confiança | Por quê |
| --- | --- | --- |
| Modelo/schema/steps/estado | **92%** | executado, código copiado e conferido, citações amostradas |
| Serialização (JSON/HTML/Markdown) | **88%** | exemplos executados; HTML/Markdown cobertos por testes host; parser não testado com HTML real da web |
| Adapter/contenteditable/IME/clipboard | **78%** | só leitura de código e testes não executados |
| A11y | **85%** | ausência verificável por grep e por API; comportamento real de leitor de tela não testado |
| Performance | **80%** | benchmark reexecutado, mas com variação alta e sem profiling |
| **Geral** | **85%** | média ponderada; o eixo browser é o que segura |

## 8. Lacunas e o que não foi executado

- **Nenhum teste de browser** (`wasm_bindgen_test`): 78 (dom) + 10 (leptos) +
  4 (dioxus) + 4 (table-view) = 96 casos. Exigem Chrome-for-Testing 148 +
  chromedriver + `wasm-bindgen-cli` 0.2.121 patchado; o runner instalado é
  0.2.128. Isso inclui os testes de IME (`composition.rs`, 3 casos), clipboard
  (6), drag & drop (4) e o contrato SSR (`view_html_contract.rs`, 3).
- **Demos Leptos/Dioxus não compilados**: exigem `trunk`/`dx` e a árvore wasm
  de `leptos 0.8.3`/`dioxus 0.7`.
- **Não verificado em runtime**: Safari/Firefox, IME CJK real, colagem de HTML
  da web, arrasto nativo, se `input` dispara em `cut`.
- **Não reexecutado**: tempos de serialização (844 KB/~11 ms para 5 000
  parágrafos) — medidos uma vez, sem harness preservado.
- **Não perfilado**: a atribuição do custo por tecla (clone de histórico,
  `node_size` O(n), `Fragment::from_nodes`) é `[INFERENCE]`; o benchmark mostra
  o efeito, não a causa.
