# taino-edit — dossiê de referência

> Editor de texto rico (WYSIWYG) escrito em Rust, com modelo **tipado** ao estilo
> ProseMirror/TipTap, uma ponte para `contenteditable` e adapters finos para
> Leptos e Dioxus. **Biblioteca**, não produto: não existe app publicado, só
> crates, exemplos e demos.
>
> Código lido: `ref/taino-edit/` (workspace v0.7.0, commit
> `90ba5ed82097b2c7b5fc8158531e090ef89f3b4a`, 2026-09-05, clone **shallow de
> 1 commit** — `git rev-list --count HEAD` = 1, `.git/shallow` contém só esse
> SHA; `git status` está limpo). Nada em `ref/` foi modificado; builds usaram
> `CARGO_TARGET_DIR=/tmp/taino-target`.

## 1. O que é, exatamente

Sete crates num workspace Cargo (`ref/taino-edit/Cargo.toml:1-30`):

| Crate | Papel | Fonte |
| --- | --- | --- |
| `taino-edit-core` | modelo, schema, steps, estado, histórico, comandos, keymap, input rules, JSON/HTML/Markdown. **Zero** `web-sys`, Leptos ou Dioxus | `ref/taino-edit/crates/taino-edit-core/src/lib.rs:1-25` |
| `taino-edit-dom` | ponte `contenteditable`: mount, diff/patch, seleção, IME, clipboard, decorations | `ref/taino-edit/crates/taino-edit-dom/src/view.rs:1-70` |
| `taino-edit-extensions` | **14** extensões embutidas — 14 `impl Extension for` em `src/`; o demo compõe 13 (todas menos `TransformCase`) | `ref/taino-edit/crates/taino-edit-extensions/src/lib.rs:1-70`; `examples/basic-leptos/src/main.rs:98-112` |
| `taino-edit-leptos` | `<TainoEditor>` sobre `RwSignal<EditorState>` | `ref/taino-edit/crates/taino-edit-leptos/src/lib.rs:1-30` |
| `taino-edit-dioxus` | `<TainoEditor>` sobre `Signal<EditorState>` | `ref/taino-edit/crates/taino-edit-dioxus/src/lib.rs:1-14` |
| `taino-edit-table-view` | interação de ponteiro em tabelas como `ViewPlugin` | `ref/taino-edit/crates/taino-edit-table-view/src/lib.rs:1-40` |
| `taino-edit` | umbrella com features `dom`/`extensions`/`leptos`/`dioxus`/`table-view` | `ref/taino-edit/crates/taino-edit/src/lib.rs:1-45` |

Licença dupla **MIT OR Apache-2.0** (`ref/taino-edit/Cargo.toml:17`; arquivos
`LICENSE-MIT` e `LICENSE-APACHE` na raiz). MSRV declarado: **1.80**
(`ref/taino-edit/Cargo.toml:16`). Versão publicada: **0.7.0**, com histórico
Keep-a-Changelog desde 0.1.0 (`ref/taino-edit/CHANGELOG.md:9-30`).

O objetivo declarado é preencher a lacuna de um WYSIWYG **nativo em Rust** para
Leptos/Dioxus, em oposição a `leptos-tiptap`, que embrulha o bundle TypeScript
do TipTap (`ref/taino-edit/DESIGN_NOTES.md:6-22`). A arquitetura é explicitamente
uma cópia conceitual do par ProseMirror + TipTap (`DESIGN_NOTES.md:23-51`).

## 2. O que funciona (verificado por execução)

Executei, no próprio checkout:

```
cargo test -p taino-edit-core -p taino-edit-extensions      # 212 testes, 0 falhas
cargo test -p taino-edit-dom --lib                          #   6 testes, 0 falhas
cargo run   -p headless-core                                # exemplo headless
```

Os 212 são **208 testes `#[test]` + 4 doc-tests** (recontei: `grep -rc '#\[test\]'`
nos `tests/` = 208; `Doc-tests taino_edit_core … 4 passed`). Rodar com
`--all-features` dá o mesmo 212 — a feature `collab` está vazia e não liga
tests. O README do próprio projeto anuncia **214** (`README.md:39`), número que
não consegui reproduzir de nenhuma forma; é mais uma afirmação desatualizada.

Saída real de `headless-core`:

```
after edit  → HiHello world
json bytes  → 221
html        → <h1>Hi</h1><p>Hello world</p>
after bold  → <h1>Hi</h1><p><strong>Hello</strong> world</p>
after undo  → <h1>Hi</h1><p>Hello world</p>
```

O exemplo do dossiê (`exemplos/`, escrito e executado por mim) confirma o mesmo
caminho fim-a-fim:

```
inicial     → <h1>Título</h1><p>Hello world</p>
após editar → <h1>Título</h1><p>Hello world!</p>
após negrito→ <h1>Título</h1><p><strong>Hello</strong> world!</p>
após undo   → <h1>Título</h1><p>Hello world!</p>
markdown    → "# Título\n\nHello world!\n"
parágrafo com bloco dentro é rejeitado? true
```

Funciona de verdade, no core:

- **Modelo tipado imutável** com compartilhamento estrutural via `Arc`
  (`codigo/node.rs:102`, `codigo/node.rs` doc de posições ~linha 95).
- **Schema com autômato de conteúdo** (`paragraph+`, `(text|image)*`, `+ * ? |`)
  compilado por NFA→DFA, portado do ProseMirror
  (`ref/taino-edit/crates/taino-edit-core/src/content.rs:1-12`).
- **Cinco tipos de step** (`Replace`, `ReplaceAround`, `AddMark`, `RemoveMark`,
  `Attr`), todos invertíveis, mapeáveis e serializáveis em JSON
  (`codigo/step.rs:35-55`).
- **`Mapping`/`StepMap` com mirror/recover**, que sobrevive a ida-e-volta de
  undo (`ref/taino-edit/crates/taino-edit-core/src/map.rs:1-10`).
- **Algoritmo de replace** (`replace_two_way`/`replace_three_way`) — o pedaço
  mais difícil, portado de `replace.ts` (`ref/taino-edit/crates/taino-edit-core/src/replace.rs:1-10`).
- **Histórico linear com grupos e limite** (default 100)
  (`codigo/state.rs:17`, `codigo/state.rs:39-102`).
- **Comandos + keymap + input rules**
  (`ref/taino-edit/crates/taino-edit-core/src/commands.rs:1-40`,
  `.../keymap.rs:218-242`, `.../inputrules.rs:1-40`).
- **Serialização JSON validada pelo schema**, HTML escapado com parser
  estrito e limitado em profundidade, e Markdown round-trip via
  `pulldown-cmark` (`.../json.rs:1-10`, `.../html.rs:1-31`, `.../markdown.rs:1-20`).
- **Ponte DOM funcional**: mount, render recursivo, patch incremental,
  seleção bidirecional, leitura de digitação, IME, paste sanitizado,
  decorations de nó e de range (`codigo/view.rs`; `position_map.rs:1-20`).
- **SSR**: o documento inicial é serializado como HTML exatamente igual ao que
  o editor monta, com contrato fixado por teste de browser
  (`ref/taino-edit/crates/taino-edit-dom/src/view_html.rs:1-30`).
- **Tabelas span-aware** com colspan/rowspan/header/colwidth e comandos que não
  deixam `rowspan` órfão (`.../extensions/src/table.rs:1-16`).

## 3. O que NÃO funciona (ou funciona com ressalvas)

Estas são as descobertas que mais importam para nós. Todas têm caminho:linha.

1. **Sem `keymap`, Enter/Backspace/Delete não fazem nada.** O adapter sempre
   registra `keydown` e **sempre** chama `prevent_default()` para essas três
   teclas, mesmo quando nenhum comando casou:
   `let structural = matches!(key.key.as_str(), "Enter" | "Backspace" | "Delete");
   if handled || structural { kev.prevent_default(); }`
   (`ref/taino-edit/crates/taino-edit-leptos/src/lib.rs:336-339`).
   Ou seja: `<TainoEditor state=signal />` puro digita texto, mas não cria
   parágrafo. Todo exemplo real passa `keymap=` (`examples/basic-leptos/src/main.rs:158,365`;
   `examples/ssr-leptos/src/lib.rs:102,165`).
2. **Não há "stored marks".** `Mod-b` com caret vazio retorna `false` e não faz
   nada — não existe o conceito "próximo caractere sai em negrito"
   (`ref/taino-edit/crates/taino-edit-core/src/commands.rs:99-101`).
3. **`read_dom_changes` só enxerga o primeiro text run divergente** e não
   reconcilia mudanças estruturais do DOM:
   `v0.1 reports the first divergent text run`
   (`codigo/view.rs:455`, implementação em `codigo/view.rs:464-518`).
   Por isso os adapters sequestram Enter/Backspace/Delete em vez de confiar no
   `contenteditable`.
4. **Copy/cut/drag-and-drop não estão ligados nos adapters.** Existem
   primitivas (`extract_slice`, `drop_slice`, `paste_html`, `paste_markdown` em
   `codigo/view.rs:579-610`) e testes de browser para elas
   (`crates/taino-edit-dom/tests/drag_drop.rs`), mas a lista de listeners dos
   adapters é `input`, `keydown`, `compositionstart/end`, `paste`,
   `mousedown/mousemove/mouseup`, `selectionchange`, `scroll`, `resize`
   (`ref/taino-edit/crates/taino-edit-leptos/src/lib.rs:229-420`). Não há
   `copy`, `cut`, `beforeinput`, `dragstart`, `dragover`, `drop`.
   Copiar funciona por comportamento nativo do browser; recortar/arrastar não
   foram verificados e não têm caminho de reconciliação.
5. **Colaboração não existe.** Só o documento de projeto
   (`ref/taino-edit/COLLAB_DESIGN.md`), nenhuma linha do crate
   `taino-edit-collab`, e `core` só tem uma feature vazia
   `collab` (`ref/taino-edit/crates/taino-edit-core/Cargo.toml:22-24`,
   `DESIGN_NOTES.md:110-121` decisão 6).
6. **Tabelas não têm keymap default**: os comandos são exportados, mas o host
   precisa ligá-los, porque `Tab` colide com indentação de listas
   (`.../extensions/src/table.rs:22-27`: "No default keymap — the structural
   commands are exported for a host toolbar to wire").
7. **Plugins são observadores, não editores.** `Plugin::apply` não pode mudar o
   documento; histórico fica de fora como campo de `EditorState`
   (`ref/taino-edit/crates/taino-edit-core/src/plugin.rs:1-25`).
8. **Acessibilidade é mínima**: `contenteditable="true"` + `tabindex="0"` no
   root (`codigo/view.rs:102-106`), e nada mais. Não há `role`, `aria-*`,
   `aria-multiline`, `aria-label`, `spellcheck` em nenhum crate
   (grep em `crates/` não acha nenhum `role=`/`aria-`; os exemplos têm só
   `role="toolbar"`/`role="search"`).
9. **Custo por tecla cresce com o documento**: ~27–39 µs com 100 blocos,
   ~94–119 µs com 5 000, ~565–862 µs com 20 000 (medido; a reexecução da
   revisão está em §5). O ROADMAP registra o risco
   "documentos >10k nós" (`ref/taino-edit/ROADMAP.md:323`).
10. **`EditorState::apply` clona o histórico inteiro e inverte todos os steps a
    cada transação com mudança** (`codigo/state.rs:190-198`).
11. **Atributos default vazam para a serialização**: o parágrafo sai no JSON com
    `"attrs": {"text_align": null}` (observado na saída do exemplo), porque a
    extensão `Align` declara o default
    (`.../extensions/src/align.rs:20-27`).
12. **`Fragment::replace_child`/`cut` etc. reconstroem o vetor de filhos** —
    copy-on-write por caminho, não persistente de verdade
    (`ref/taino-edit/crates/taino-edit-core/src/fragment.rs:16-45,100-165`).

## 4. Instalação e primeira edição

```toml
[dependencies]
taino-edit = { version = "0.7", features = ["leptos"] }  # ou "dioxus"
```

(`ref/taino-edit/README.md:163-176`.) Nenhum adapter vem ligado por padrão;
`table-view` é feature separada. O adapter Leptos é **neutro quanto ao modo de
render** — não liga `csr`/`hydrate`/`ssr` (README `### Server-side rendering`,
linhas 207-220).

O mínimo real (compilado do README, `README.md:178-206`) é: declarar `doc` e
`text` à mão, empilhar extensões com `build_schema_with`, montar o documento com
`schema.node(...)` (que **valida** contra o autômato) e passar
`RwSignal::new(EditorState::new(doc, schema))` para `<TainoEditor>`. Na prática
você também precisa de `build_keymap_with(...)` no prop `keymap` — senão o
editor não quebra linha (ver §3.1).

Exemplo mínimo executável em Rust puro (sem browser), com saída observada, em
[`exemplos/README.md`](exemplos/README.md). O componente Leptos mínimo está no
mesmo arquivo, não executado (falta toolchain `trunk`/wasm no ambiente).

## 5. Números medidos

Testes (executados por mim):

| Suíte | Casos | Resultado |
| --- | --- | --- |
| `taino-edit-core` + `taino-edit-extensions` (host) | 212 | ok |
| `taino-edit-dom --lib` (host, contrato SSR) | 6 | ok |

Testes declarados no repo que **não** executei: 78 casos `wasm_bindgen_test` em
`taino-edit-dom`, 10 em `taino-edit-leptos`, 4 em `taino-edit-dioxus`, 4 em
`taino-edit-table-view` (contagem por `grep -c '#\[wasm_bindgen_test\]'`).
Motivo: exigem `chromedriver`/Chrome-for-Testing 148 + runner `wasm-bindgen-cli`
patchado (`ref/taino-edit/scripts/wasm-test.sh:1-20`; o runner instalado aqui é
0.2.128 e o lock fixa 0.2.121).

Benchmark próprio (`/tmp/taino-bench`, `--release`, melhor de 3, histórico
profundidade 100), inserindo caracteres no fim da última sentença. Números da
primeira medição e, em parênteses, a **reexecução da revisão** (mesmo harness,
máquina do revisor; o harness original hoje só roda 10k/20k, então criei uma
cópia com a varredura completa em `/tmp/taino-bench2`):

| Blocos (~90 chars cada) | Tempo/tecla (1ª medição / reexecução) |
| --- | --- |
| 100 | ~27 µs / 39 µs |
| 1 000 | ~29–65 µs / 29 µs |
| 5 000 | ~94–119 µs / 94 µs |
| 10 000 | ~189 µs / 191–198 µs |
| 20 000 | ~825 µs / **565–862 µs** |

Leitura honesta: a curva é reprodutível na forma (patamar até ~5k, ~2x de 5k
para 10k, salto de 3–4x de 10k para 20k), mas os valores absolutos variam
bastante entre execuções — sobretudo nas pontas (100 e 20 000). Trate "27 µs" e
"825 µs" como uma amostra, não como constantes do projeto.

Serialização do mesmo documento: 5 000 parágrafos → JSON 844 KB em ~11 ms,
HTML 444 KB em ~3,2 ms.

## 6. Prós e contras como editor rico

**Prós**

- Modelo de dados sério: tipado, imutável, validado, com posições à la
  ProseMirror e steps invertíveis. É a fundação certa, e é a mesma que escolhemos
  conceitualmente.
- Separação real `core` ↔ `dom` ↔ adapter: o core roda em servidor/CLI
  (`examples/headless-core`) sem nenhuma dependência web.
- Serialização tripla (JSON schema-checked, HTML escapado + parser estrito com
  `MAX_DEPTH=100`, Markdown) — e o teste de contrato SSR ↔ DOM montado é uma
  ideia excelente.
- Comandos no contrato ProseMirror (`probe` sem dispatch, `chain`) e keymap com
  *encadeamento* em vez de sobrescrita (`keymap.rs:179-215`), o que resolve
  colisões entre extensões.
- Tabelas realmente span-aware, com compactação de grid — bem acima do que a
  maioria dos editores embutidos entrega.
- Licença permissiva e CI com fmt/clippy/test/doc + job separado de SSR.

**Contras**

- O "editor" só existe com keymap explícito; o estado inicial "copia o README"
  é uma armadilha (§3.1).
- `contenteditable` usado como *input device* frágil: só texto divergente é
  lido de volta; estrutura é toda interceptada. Qualquer tecla/gesto não mapeado
  (IME exótico, autocorrect móvel, ditado, arrastar) não tem caminho.
- Sem stored marks, sem copy/cut próprios, sem DnD ligado, sem a11y real: a
  experiência fica abaixo de TipTap/Quill em polimento, apesar do modelo melhor.
- Performance degrada de forma superlinear depois de ~10k blocos e cada tecla
  paga clone de histórico.
- Duas cópias quase idênticas de wiring de eventos (Leptos e Dioxus), 496 e 571
  linhas, sem camada comum.
- Adapters fixam `class="taino-editor"` no container
  (`ref/taino-edit/crates/taino-edit-leptos/src/lib.rs:189`,
  `.../taino-edit-dioxus/src/lib.rs:228`) sem prop para trocar.
- Documentação de arquitetura excelente, mas o README do projeto ainda repete
  números antigos: diz "Twelve built-in extensions" (`README.md:118`) e
  "13 built-in extensions" (`README.md:39,144`) no mesmo arquivo, e anuncia
  **214** testes de host (`README.md:39`) quando o comando do próprio README
  produz 212. O código tem **14** `impl Extension for`; o demo compõe 13
  (`examples/basic-leptos/src/main.rs:98-112`). Nada disso muda a arquitetura,
  mas é sinal de doc não regenerada.

## 7. Quão "headless" e "zero CSS" ele é de verdade

**Headless: sim, com um porém.** `taino-edit-core` não depende de `web-sys`,
Leptos ou Dioxus (`crates/taino-edit-core/Cargo.toml:14-18` só tem
`pulldown-cmark`, `regex`, `serde`, `serde_json`) e o exemplo `headless-core`
edita, serializa e desfaz sem DOM — rodei. Mas "headless" aqui significa
*modelo + transações*, não *renderer alternativo*: a única forma de renderizar é
`taino-edit-dom` + `contenteditable` (`web-sys`/`wasm-bindgen`/`js-sys`). Não há
trait de renderer; `blitz`/`freya` são explicitamente futurologia
(`DESIGN_NOTES.md:47-50`, `ROADMAP.md:305-312`).

**Zero CSS: quase.** Não existe nenhum `.css`/`.scss` no repositório
(`find . -name '*.css' -o -name '*.scss'` volta vazio). O visual é 100% do host
— `examples/basic-leptos/index.html` traz o CSS do demo num `<style>`, e o
`ssr-leptos` estiliza `.taino-editor` em `<style>` no próprio componente
(`examples/ssr-leptos/src/lib.rs:161-164`), o que só funciona porque a classe é
fixa. Mas o editor **emite marcação com semântica de estilo**:

- classes fixas no DOM vivo: `taino-editor` no container
  (`leptos/src/lib.rs:189`, `dioxus/src/lib.rs:228`), `taino-cell-selected` na
  célula (`table-view/src/lib.rs:33`) e `taino-deco-layer` na camada de
  decorações (`dom/src/view.rs:315`);
- **atributos `style` inline dentro do documento serializado**:
  `style="text-align: …"` nos blocos alinhados
  (`extensions/src/align.rs:50-60`) e `style="width: …px"` em células
  (`extensions/src/table.rs:87`) — este último vai para o HTML e o JSON só
  carrega `colwidth`, ou seja, o estilo sai *dentro* do conteúdo.

Ou seja: sem stylesheet embutido, porém com `style` inline no documento, o que é
pior para "zero CSS" do que classes — o estilo viaja com o dado. A classe
`taino-editor` também não é configurável: `TainoEditor` expõe só `state`,
`plugins` e `keymap` (`leptos/src/lib.rs:71-91`), sem prop de classe.

## 8. Veredito rápido para o `rich-editor-auy`

O veredito tem **duas metades que não podem ser somadas**: o core Rust está
próximo do que queremos; o adapter Leptos + a camada de acessibilidade ainda são
immaturos. Comprar o "modelo" sem herdar o "adapter" é a decisão certa.

**A. Core Rust (`taino-edit-core`) — próximo, vale copiar conceitualmente.**

- Modelo tipado imutável, schema com autômato, steps invertíveis, `Mapping`
  com mirror/recover, comandos `probe`/`dispatch`, keymap encadeado e o contrato
  "serializador SSR = markup do renderer": isso é exatamente a fundação do
  `rich-editor-auy`, e roda sem DOM (`examples/headless-core`).
- Ressalvas do core: `node_size` O(n) sem comprimento de texto cacheado,
  histórico clonado/invertido por transação, expressão de marks por nó
  declarada e não aplicada, `map_against` documentado e ausente.

**B. Adapter Leptos/Dioxus + a11y — imaturo, adaptar ou reescrever.**

- Enter/Backspace/Delete só funcionam com `keymap` explícito e são
  `prevent_default` sempre; sem keymap o editor não quebra linha.
- `read_dom_changes` reconcilia **só o primeiro text run divergente**; copy/cut/
  DnD não têm listener nos adapters; não há stored marks.
- `contenteditable` + `tabindex` e nada mais: zero `role`/`aria-*`, nenhum
  caminho de leitor de tela, foco por teclado é o mínimo do browser.
- `style` inline no documento e `class="taino-editor"` sem prop; wiring de
  eventos duplicado entre os dois adapters (496 vs 571 linhas).
- Consequência prática: usar o core como referência de modelo, **não** copiar o
  adapter como está; o custo de a11y e de reconciliação de DOM é nosso.

**O que fazer, em ordem:**

- **Copiar conceitualmente**: o modelo tipado + schema com autômato, steps
  invertíveis com `Mapping` mirror, contrato de comando `probe/dispatch`,
  keymap encadeado, e a ideia de *um* serializador que é o mesmo markup do
  renderer (contrato SSR).
- **Adaptar**: histórico (não clonar por transação), leitura do DOM (não aceitar
  só o primeiro text run), tabelas (o modelo de grid funciona), extensões como
  três eixos explícitos (schema/keymap, plugin de estado, view plugin).
- **Evitar**: depender de keymap para o básico, sequestrar `preventDefault` de
  teclas estruturais sem fallback, `style` inline no documento, adapter com
  wiring duplicado, e a ideia de que `contenteditable` pode ser reconciliado
  lendo só texto.

Detalhes de arquitetura em [`arquitetura.md`](arquitetura.md); rubrica,
cobertura e evidência em [`avaliacao.md`](avaliacao.md); código copiado com
licença em [`codigo/INDEX.md`](codigo/INDEX.md).
