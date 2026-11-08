# 01 — Referências: o que absorver, o que recusar

> **Método.** Leitura direta dos repositórios em [`ref/`](../ref/) (código, README, guias de
> agentes, decisões) e da documentação pública do ProseMirror e do CodeMirror 6. Onde a afirmação
> é sobre código, o caminho do arquivo está citado; onde é sobre arquitetura pública, o link está
> no texto. Não há benchmark reproduzido aqui: números de terceiros são citados como motivação,
> não como prova.

---

## 1. ProseMirror — o desenho que queremos herdar

O ProseMirror é a referência mais importante porque resolveu, em 2015–2016, os problemas que
ainda aparecem em editores de 2026. Do [guia oficial](https://prosemirror.net/docs/guide/):

- **Documento = árvore de blocos + conteúdo inline plano.** Um parágrafo com
  `<strong>forte com <em>ênfase</em></strong>` não vira uma árvore de marcas; vira uma sequência de
  runs de texto com um conjunto de marcas em cada run. Isso permite posições por offset dentro do
  parágrafo e edição de marca por divisão de runs.
- **Uma única representação válida.** Runs adjacentes com o mesmo conjunto de marcas são
  combinados; runs vazios não existem; a ordem das marcas é definida pelo schema.
- **Schema com expressões de conteúdo.** `"paragraph+"`, `"(paragraph | blockquote)+"`,
  `"heading paragraph+"`, grupos (`block`), cardinalidades `{2}`, `{1,5}`, `{2,}`.
- **Steps pequenos, invertíveis e mapeáveis.** `ReplaceStep`, `AddMarkStep`, `StepMap`,
  `Mapping` com viés; `apply` devolve um resultado que pode falhar; `invert` produz o step de
  desfazer; `map` reposiciona um step sobre outro.
- **Transações.** Toda edição passa por uma transação; a transação vira um novo estado; o estado
  inclui doc, seleção e `storedMarks`.
- **Rebase explícito.** O guia documenta o pipeline de rebase (`invert(mapB1), mapA1, mapA2,
  mapB1'`) e avisa que um step rebaseado ainda pode falhar ao aplicar.
- **Autofill determinístico.** Quando um step deixa conteúdo inválido, o schema fornece o
  primeiro tipo válido para preencher; a ordem da or-expression importa e uma escolha recursiva
  ingênua pode causar stack overflow (o guia alerta explicitamente).

**Absorver:** o modelo de dois níveis, a representação única, a álgebra de steps, o viés de
mapeamento, a noção de slice com profundidades abertas, o autofill determinístico, a separação
entre `prosemirror-model`/`state`/`transform`.

**Recusar:** o runtime JS e a dependência de `contenteditable`; a superfície de plugins como
cidadã de primeira classe (é poderosa, mas o custo de compatibilidade é alto); a ideia de que o
core pode "resolver tudo"; o uso de `char`/UTF-16 como unidade interna (o guia usa offsets de
texto; nós usaremos bytes UTF-8 e conversão explícita).

**O que o guia não resolve para nós:** bytes canônicos como contrato de biblioteca, `no_std`,
orçamento de alocação, schema validado em compilação, import/export com relatório de perdas,
`Send + Sync` com snapshots baratos. Esses são nossos problemas.

---

## 2. CodeMirror 6 — a fronteira estado/view

O CodeMirror 6 não está em `ref/`, mas é a melhor referência pública de **separação estado/view**
para editores:

- `@codemirror/state` é headless: `EditorState` imutável, transações, seleção, facets.
- `@codemirror/view` é a camada de DOM/eventos; o estado não conhece o DOM.
- Um estado pode ser compartilhado por múltiplas views; cada view mantém seu próprio estado de
  renderização.
- Transações são valores; extensões declaram comportamento por *facets* em vez de mutar o core.

Fontes: [README do `@codemirror/state`](https://github.com/codemirror/state/blob/main/src/README.md)
e [design doc do CodeMirror 6](https://codemirror.net/).

**Absorver:** a separação radical; a ideia de que a view é descartável e o estado é o produto; a
noção de que múltiplas views podem observar o mesmo estado.

**Recusar:** o sistema de *facets*/extensões do CodeMirror é uma resposta ao problema de
composição dinâmica em JS; em Rust, um registry genérico de plugins heterogêneos custa `Any`,
`Box<dyn>`, ordem de inicialização e compatibilidade. Nosso MVP usa pontos de extensão
declarativos e deixa o registry para depois de evidência.

**Lição específica:** o CodeMirror não tenta ser um framework de UI; ele é um engine de texto com
uma view. O `rich-editor-auy` é o equivalente para rich text, sem view.

---

## 3. Lexical — identidade de nó e double buffering

Do [`ref/lexical/AGENTS.md`](../ref/lexical/AGENTS.md):

- `EditorState` imutável, serializável em JSON, com árvore de nós e seleção.
- *Double buffering*: `editor.update()` clona o estado atual como work-in-progress; mutações
  acontecem nesse clone; updates síncronos são agrupados; o reconciliador de DOM aplica o diff; o
  novo estado imutável vira o corrente.
- **Node keys**: cada nó lógico tem uma `__key` de runtime; clones compartilham a key; `getWritable`
  cria a cópia mutável. Isso permite que métodos de nó sempre achem a versão mais recente no estado
  ativo.
- As funções `$` só podem ser chamadas dentro de `editor.update`/`editor.read`/handlers; é um
  "contexto síncrono" imposto em runtime.
- `LexicalSchema.ts` agora expõe um schema de serialização estrutural (tipos, domínios, defaults,
  aliases) que também serve para gerar JSON Schema e arbitraries de teste.
- `@lexical/headless` existe: o produto reconhece o caso de uso headless.

**Absorver:** a ideia de identidade estável de nó para a UI (cache de layout, decorações,
seleção) e a separação entre "estado lógico" e "work-in-progress" mutável.

**Recusar:** o contexto global mutável (`editor.update`) e a key de runtime como parte do modelo.
Em Rust, um `Transaction` explícito e um `Doc` imutável já dão o double buffering sem estado
global. IDs de nó de runtime são **efêmeros** (para a UI); a única exceção no formato canônico é a
âncora de publicação (`NodeId`) opcional e explícita, que existe para referência entre documentos,
não para cache de renderização.

**Lição específica:** o schema de serialização do Lexical é uma boa ideia (metadados que servem
para parse, teste e documentação). O nosso schema já é a fonte de verdade; podemos expor
metadados sem duplicar a declaração.

---

## 4. Quill — Delta como álgebra de mudança, Parchment como árvore

Do [`ref/quill/packages/quill/src/core/quill.ts`](../ref/quill/packages/quill/src/core/quill.ts):

- `quill-delta` é a representação de mudanças e de conteúdo: `retain`/`insert`/`delete` com
  atributos.
- `parchment` é a árvore de *blots* que espelha o DOM; `Block`, `Inline`, `Text`, `Embed` etc.
- `quill.getContents()` devolve um Delta; `updateContents(delta)` aplica.
- O modelo de editor é uma árvore de blots, mas a álgebra pública é flat (Delta) e os índices são
  sobre o conteúdo linearizado.
- Modules (clipboard, keyboard, history, uploader) são plugins de UI/comportamento.

**Absorver:** a álgebra de change set compacta (retain/insert/delete) como inspiração para o
`ChangeSet` que a UI recebe; a ideia de que a UI pode aplicar mudanças incrementais.

**Recusar:** a árvore de blots como fonte da verdade e o acoplamento ao DOM. O Delta é ótimo para
mudanças lineares, mas não expressa conteúdo estruturado aninhado (listas, tabelas) sem extensões;
nosso modelo é uma árvore de blocos com inline plano, e o change set é derivado dos steps, não a
fonte da verdade.

**Lição específica:** um editor pode ter uma API flat excelente e ainda assim precisar de uma
árvore interna. A fronteira entre "o que a UI vê" e "o que o modelo é" pode ser assimétrica.

---

## 5. TipTap — a prova de mercado do "headless toolkit"

Do [`ref/tiptap/AGENTS.md`](../ref/tiptap/AGENTS.md): "Headless rich text editor toolkit built on
ProseMirror. A monorepo of small packages: a framework-agnostic core and extensions, plus React
and Vue bindings."

- Pacotes pequenos e composáveis: `core`, `extension-bold`, `extension-list`, `extension-table`,
  `html`, `markdown`, `static-renderer`, `react`, `vue`, `suggestion`, `pm`.
- Extensões são declarativas e se compõem; o consumidor escolhe o conjunto.
- O core é framework-agnóstico; os bindings são finos.
- A documentação é voltada ao consumidor, não ao implementador.

**Absorver:** a granularidade de extensões e a ideia de que o consumidor compõe o vocabulário;
o `static-renderer` como prova de que renderizar para HTML/SSR é um adaptador, não o core.

**Recusar:** depender do ProseMirror (e, portanto, de um runtime JS); a superfície de extensões
como cidadã de primeira classe no core (o custo de compatibilidade é permanente).

**Lição específica:** existe demanda real por "headless + extensões"; o nosso diferencial é ser
Rust nativo, canônico, `no_std` e com orçamento de performance.

---

## 6. Summernote — o contraexemplo necessário

Do [`ref/summernote/src/js/module/Editor.js`](../ref/summernote/src/js/module/Editor.js):

- jQuery + `document.execCommand` (`bold`, `italic`, `formatBlock`, `insertOrderedList`…).
- O "modelo" é o DOM editável; `snapshot` é o histórico; `range`/`dom` são utilitários.
- Não há schema, não há representação canônica, não há steps invertíveis.
- As operações são comandos do navegador; o resultado depende do navegador.

**Absorver:** nada do modelo. Talvez a lição de produto: um editor "simples" é atraente, e a
complexidade precisa se pagar em correção e performance.

**Recusar:** tudo que depende do DOM como verdade. É exatamente o defeito que o projeto existe
para não repetir.

**Lição específica:** sem modelo canônico, não há como garantir undo correto, serialização
determinística, colaboração ou testes diferenciais. O custo de um modelo é alto, mas o custo de
não ter modelo é maior.

---

## 7. taino-edit — o porte Rust mais próximo (e o mapa das pedras)

Do [`ref/taino-edit/DESIGN_NOTES.md`](../ref/taino-edit/DESIGN_NOTES.md) e do código:

- Arquitetura em camadas: `taino-edit-core` (modelo, transforms, state, history, commands, keymap,
  input rules), `taino-edit-dom` (contenteditable), adaptadores Leptos/Dioxus, extensões.
- Core de 6.667 linhas; 16 arquivos de teste; dependências `pulldown-cmark`, `regex`, `serde`,
  `serde_json` (core), `web-sys`/`wasm-bindgen` (dom).
- Modelo: `Node(Arc<NodeInner>)`, `Fragment`, `Slice`, `Schema`/`SchemaBuilder`, `AttrValue`/
  `Attrs`, `NodeType`/`MarkType`, `ContentMatch` (NFA→DFA para expressões de conteúdo).
- Steps: trait `Step` + `Box<dyn Step>` (`ReplaceStep`, `ReplaceAroundStep`, `AddMarkStep`,
  `RemoveMarkStep`, `AttrStep`), `StepMap`, `Mapping` com mirror/recover.
- Posições: `usize` em **escalares Unicode** (`chars().count()`), com nota explícita de que difere
  do ProseMirror (UTF-16).
- Schema: `HashMap` para nós/marcas/atributos; `content.rs` compila expressões.
- Estado: `EditorState` com doc, seleção, schema, histórico, plugins; `History` linear com
  `HistoryIntent`; `Plugin` trait com estado associado.
- Serialização: `serde`/`serde_json`, schema-checked; HTML parser/serializer próprio no core;
  Markdown via `pulldown-cmark`.
- `#![deny(unsafe_code)]`, `#![forbid(unstable_features)]`, `missing_docs` warn.

**Absorver:** a divisão core/adaptador; o `ContentMatch` (NFA/DFA) como referência de
implementação; a ideia de `Step::map` para rebase; a experiência de que `History` não se encaixa
bem no trait `Plugin` (eles decidiram mantê-lo como campo de primeira classe — ver
`CHANGELOG.md`).

**Recusar/evitar:** `Arc<Node>` por nó sem orçamento; `Box<dyn Step>`; posições em `char`;
`HashMap` no schema; `serde` no core; `regex` para input rules; ausência de testes de alocação e
de determinismo; `usize`/`as` em aritmética de posição.

**Lição específica:** o taino-edit prova que o escopo é factível em Rust e que o ProseMirror é um
bom mapa. Também prova que "porte fiel" não é o mesmo que "desenho para os objetivos": as escolhas
acima são razoáveis para um produto JS-portado, não para um engine com orçamento e `no_std`.

---

## 8. knudge — o sistema operacional de rigor e planejamento

O `ref/knudge/` é a referência de **processo**, não de domínio. O que absorvemos:

- **Plano em camadas:** `plan/00_panorama.md` (visão), `plan/02_decisoes.md` (abertas, com
  bloqueio 🔴/🟠/🟡), `plan/03_decisoes-fechadas.md` (contrato), `plan/implementation/` (épicos e
  tarefas com aceite), `plan/brainstorm/` (registro cru), `plan/proposals/` (mudanças futuras).
- **Políticas de engenharia** (`R01–R45`) separadas de decisões de produto (`Dxx`): uma política é
  uma regra transversal (memória, logs, erros, build), uma decisão é uma escolha de domínio.
- **Gate único local:** `make check` = `fmt --check` + `clippy --all-targets -D warnings` + `test`
  + gate de tamanho de arquivo (`scripts/check_file_length.sh`, 300 linhas de produção).
- **Rigor de lints:** `clippy.toml` com valores (limiares, allowlists, `disallowed-types`,
  `disallowed-methods`, `disallowed-macros`) e `[workspace.lints]` com níveis; `unwrap`/`expect`/
  `panic` proibidos **inclusive em testes**; `arithmetic_side_effects` warn; `indexing_slicing`
  deny; `Rc`/`RefCell`/`HashMap`/`HashSet` proibidos no core.
- **Determinismo como requisito:** portas `Clock`/`Rng`/`Env`/`Fs`, fakes nos testes, proibição de
  `SystemTime::now`, `env::var`, iteração de `HashMap`; ordem canônica de bytes.
- **`DIVERGENCES.md`:** cada borda onde a implementação poderia divergir de si mesma (Unicode,
  ordem, hash, lock, atomicidade) com mitigação e **o teste que a trava**. É o melhor artefato de
  rigor do repositório.
- **Matriz de aceite por tool:** formato de saída, erro, exit code, estado persistido, teste que
  cobre. Para nós, o equivalente é uma matriz por feature (schema, doc, steps, seleção, JSON…).
- **`AGENTS.md` operacional:** idioma (docs PT, identificadores EN), `make check`, mapa do
  repositório, padrões, erros, logs, testes, checklist.
- **Decisões com consequência:** cada `Dxx` tem contexto → decisão → consequência; mudar exige
  nova entrada. O `plan/` é registro histórico, não documentação viva.

**Adaptar:** o knudge é um binário CLI com I/O; nós somos uma biblioteca pura. Portanto:
`no_std + alloc` em vez de `std`; sem portas de I/O (não há I/O); sem `tokio`; sem logs (o core
não escreve); erros tipados, sem `anyhow`; a matriz de aceite é por feature de biblioteca, não por
verbo de CLI.

**Não copiar:** o limite de 300 linhas é uma boa disciplina, mas pode ser apertado para tabelas
geradas; a regra será 300 linhas para `src/` com exceções explícitas para tabelas de schema
geradas (se houver) e testes. O `arithmetic_side_effects` do knudge é `warn`; nós queremos `deny`
com um módulo `math` auditado. O knudge usa `thiserror`; nós usamos `Display`/`core::error::Error`
à mão para evitar essa dependência. `blake3` é a exceção normal auditada (F30).

---

## 9. html-auy — a filosofia de biblioteca

Do [`ref/html-auy/docs/22-rigor.md`](../ref/html-auy/docs/22-rigor.md),
[`docs/00-principios.md`](../ref/html-auy/docs/00-principios.md) e
[`AGENTS.md`](../ref/html-auy/AGENTS.md):

- **Duas camadas de rigor:** níveis de lint no `Cargo.toml`, valores no `clippy.toml`. Uma sem a
  outra é ficção.
- **Dois números de toolchain:** pin de desenvolvimento (`rust-toolchain.toml`, 1.99) ≠ MSRV
  (`rust-version`, 1.92); `resolver = "3"` para respeitar MSRV; job de CI na MSRV real.
- **Uma fonte de verdade:** a tabela de elementos é gerada de `spec/elements.toml`; as cópias Rust
  são verificadas por paridade; nunca editar a tabela à mão.
- **Sem camada advisory:** ou é erro de compilação, ou é teste que falha, ou é gate do consumidor.
  Aviso que não bloqueia é documentação de pior qualidade.
- **Sem `unsafe`, sem `unwrap`/`expect`/`panic`, sem `HashMap` em caminho de emissão, sem relógio/
  ambiente, sem `as` silencioso, sem indexação.**
- **Bytes canônicos:** ordem de atributos/classes/assets é estável; ETag é confiável; golden
  byte-for-byte; regeneração explícita.
- **Erro de compilação com span:** `compile_error!` + `trybuild`; mensagem que ensina.
- **Corte de escopo:** o crate não é framework, não é VDOM, não é sanitizador, não é HTTP. O
  consumidor é dono de I/O, rotas, sessão, CSS e gates de navegador.
- **`AGENTS.md` como guia de evolução:** missão, fontes de verdade, regras duras, validação
  obrigatória, checklist de aceitação.

**Absorver:** a filosofia de biblioteca, a separação níveis/valores, os dois números de toolchain,
a fonte única de schema, a proibição de advisory, a noção de bytes canônicos, o `AGENTS.md` com
regras duras, o checklist de aceitação.

**Adaptar:** o html-auy valida HTML em compilação com uma tabela enorme; nós validamos um schema de
editor. O análogo do `html!` é o `schema!` (macro opcional) e os comandos tipados. O análogo do
`Page`/`audit` é o `wire`/`ImportReport`. O análogo do `Markup` é o `Doc` canônico.

**Não copiar:** o html-auy é `no_std` sem `alloc`? Não: é `no_std + alloc`; nós também. O
html-auy não tem estados mutáveis quentes; nós temos um caminho de edição por caractere, e por
isso o orçamento de alocação é mais importante do que lá. O html-auy usa `trybuild` para mensagens
de compilação; nós usaremos `trybuild` para a macro `schema!` e para erros de tipo.

---

## 10. Tabela comparativa

| Projeto | Modelo | Verdade | Imutável? | Posições | Serialização | Extensão | Headless? | Rigor | Rust/no_std |
|---|---|---|---|---|---|---|---|---|---|
| ProseMirror | árvore de blocos + inline plano | doc + schema | sim (steps) | offset de texto | JSON | plugins | sim | alto (JS) | não |
| CodeMirror 6 | árvore de linhas + estado | estado | sim | offset de texto | não é o foco | facets | sim | alto (TS) | não |
| Lexical | árvore de nós + keys | estado | sim (double buffer) | offset | JSON + schema | extensões | sim (`headless`) | alto (Flow/TS) | não |
| Quill | árvore de blots + Delta | DOM/blots | parcial | índice flat | Delta | modules | não (DOM) | médio | não |
| TipTap | ProseMirror | doc | sim | offset | JSON/HTML | extensões | sim | alto (TS) | não |
| Summernote | DOM | DOM | não | range DOM | HTML | modules | não | baixo | não |
| taino-edit | árvore PM-like | doc | sim (Arc) | `char` | JSON (serde) | extensions/plugins | core sim, produto não | médio | sim, `std` |
| **rich-editor-auy** | **árvore de blocos + inline plano** | **doc canônico** | **sim (COW)** | **byte UTF-8** | **JSON canônico + envelope** | **schema/comandos/hooks** | **sim** | **máximo** | **sim, no_std + alloc** |

### 10.1 Referências complementares

Quatro dossiês não entram na tabela acima porque não são editores de texto, mas fecham lições do
nosso escopo:

- **MarkText/Muya**: Markdown como verdade, engine de blocos e app Electron; a engine não é
  publicada como biblioteca. Serve de baseline de compatibilidade Markdown, não de core Rust.
- **Zettlr**: CodeMirror + `LinkProvider` + citação; é o modelo mais próximo de referências entre
  publicações, mas é app GPL, reindexa tudo e descarta link quebrado. Serve de UX, não de código.
- **akribie/akribiechart**: pipeline determinístico, `Doc::diff` semântico, schema derivado,
  `Diagnostic` estável; não é editor. Ver [`../dossie/IDEIAS-akribie-calc.md`](../dossie/IDEIAS-akribie-calc.md).
- **calc-auy**: hash de conteúdo × trace, typestate `Draft`→`Committed`→`Verified`, limites na
  leitura e erro estável; MPL-2.0, sem reutilização de código. Ver o mesmo documento de ideias.

## 11. Síntese: doze lições que o desenho deve carregar

1. **Inline é plano.** Marcas são metadados de runs, não nós. Isso simplifica posições, edição de
   marca e renderização.
2. **Uma representação válida.** Normalização não é opcional; é o que torna o formato canônico.
3. **Schema é contrato.** Validar na construção, não no primeiro documento; fingerprint para
   detectar divergência.
4. **Steps pequenos e invertíveis.** `apply`/`invert`/`map` são a base de undo, colaboração e
   inspeção.
5. **Mapping tem viés.** Uma posição na borda de uma inserção/deleção não tem resposta única;
   `assoc` explícito é obrigatório.
6. **Transação é o único ponto de entrada.** Nenhuma mutação fora de `state.apply(tx)`.
7. **A UI é cliente.** O core entrega dados e change sets; não desenha, não mede, não lê I/O.
8. **Identidade de nó é da UI, não do formato.** IDs efêmeros podem existir; nunca no JSON
   canônico.
9. **Determinismo é um contrato de bytes.** Ordem, normalização, números e hashes são decididos
   antes do código.
10. **Performance é orçamento.** Contagem de alocação e instruções, não "parece rápido".
11. **Rigor é configuração + CI.** Níveis e valores de lint; MSRV real; supply chain; fuzz; Miri.
12. **O plano é registro.** Decisões com contexto e consequência; mudança exige nova entrada;
    brainstorming é histórico, não documentação viva.
