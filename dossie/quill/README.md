# Quill — estudo de caso (Delta + blots + DOM/contenteditable)

> `ref/quill/` é o monorepo do **Quill 2.0.3** (commit `539cbff`, 2025-07-25). O produto
> publicado é a **biblioteca** `quill` em `packages/quill/`; `packages/website/` é o site de
> documentação/playground, não um editor de produto. Não há app de edição neste repositório.
> A fonte de verdade do documento é um **Delta**, não o HTML. O DOM é uma projeção editável
> desse Delta, gerada e reconciliada por **blots** (Parchment), com um `MutationObserver` como
> canal de retorno.

Este dossiê responde, com `caminho:linha` e execução observada: o modelo de dados, o que é um
blot, como o DOM representa o documento, onde o CSS entra, e quão fácil é integrar Quill sem
brigar com uma folha de estilo própria. O exemplo mínimo está em
[`exemplos/minimal.html`](exemplos/minimal.html) e a saida observada em
[`exemplos/saida-minimal-dom.html`](exemplos/saida-minimal-dom.html).

---

## 1. Ficha

| Item | Valor | Evidência |
|---|---|---|
| Versão do monorepo | `quill-monorepo` 2.0.3 | `ref/quill/package.json:2-3` |
| Pacote publicado | `quill` 2.0.3, `"type": "module"` | `ref/quill/packages/quill/package.json:2-6` |
| Dependências de runtime | `eventemitter3`, `lodash-es`, `parchment@^3`, `quill-delta@^5.1.0` | `ref/quill/packages/quill/package.json:9-14` |
| Delta | **externo**, resolvido em `quill-delta@5.1.0` | `ref/quill/package-lock.json:15819-15821` |
| Licença | BSD-3-Clause (Slab/2017-2024, Jason Chen/2014, salesforce/2013) | `ref/quill/LICENSE:1-12`, `ref/quill/package.json:20` |
| Tamanho do código-fonte | 64 arquivos `.ts`, 8.795 linhas; 8 blots, 11 módulos, 21 formatos | `find src -name '*.ts'`, `ls src/{blots,modules,formats}` |
| CSS | 729 linhas de Stylus em `src/assets` | `cat src/assets/*.styl src/assets/{snow,bubble}/*.styl` |
| Testes | ~521 ocorrências de `test(` em `test/unit`; 23 em `test/e2e`; fuzz em `test/fuzz` | `grep -rho "test(" test/unit \| wc -l`; `grep -rho "test(" test/e2e \| wc -l` |
| Runner | Vitest em **browser real** (Playwright/Chromium), não jsdom | `ref/quill/packages/quill/test/unit/vitest.config.ts:22-27` |
| Maturidade | 2.0 GA em 2024, releases 2.0.1–2.0.3 em 2024; correções contínuas de IME/clipboard | `ref/quill/CHANGELOG.md:1-45` |

---

## 2. O modelo: Delta é o documento

Um documento Quill é uma lista plana de operações Delta (`insert`, `delete`, `retain`), onde
texto e formatação são separados: cada `insert` carrega texto (ou um embed) e um mapa de
atributos; o final de cada linha é um `"\n"` com os atributos de bloco. O Delta não tem árvore
nem schema — a hierarquia (parágrafo, lista, header) emerge dos atributos aplicados ao `"\n"`.

```text
new Delta()
  .insert('Olá ')
  .insert('mundo', { bold: true })
  .insert('\n', { header: 1 })          // <h1>Olá <strong>mundo</strong></h1>
```

Evidência de que o Delta é a fonte de verdade, e não o HTML:

- `Editor.getDelta()` percorre as linhas e concatena `line.delta()`:
  `ref/quill/packages/quill/src/core/editor.ts:162-166`. O `Block.delta()` delega à função
  `blockDelta`, que agrega os leaves e injeta um `'\n'` com os formatos do bloco:
  `src/blots/block.ts:19-23`, `:180-193`.
- Toda edição programática produz **e retorna** um Delta: `deleteText`
  `src/core/quill.ts:304-322`, `format`/`formatLine`/`formatText` `:347-455`, `insertText`
  `:585-613`, `setContents` `:702-720`, `updateContents` `:755-767`.
- O estado anterior é mantido em `Editor.delta` e o novo é comparado por diferença quando o DOM
  muda por fora: `src/core/editor.ts:273-315` (em especial `oldDelta.diff(this.delta)` na
  linha 312).
- O Delta é serializável diretamente (`JSON.stringify(quill.getContents())`), como no exemplo
  mínimo; `getSemanticHTML()` existe, mas é uma **projeção para colagem/exportação**, não a
  persistência: `src/core/quill.ts:543-552`.

As cinco operações que importam (implementação em
[`codigo/05-quill-delta.ts`](codigo/05-quill-delta.ts)):

| Operação | Semântica | Linha |
|---|---|---|
| `insert(arg, attributes?)` | insere texto/embed; `''` é no-op; `attributes` vazio não é gravado | `05-quill-delta.ts:69-87` |
| `delete(length)` | remove `length` caracteres; `<= 0` é no-op | `:88-94` |
| `retain(length, attributes?)` | avança; com atributos, formata o trecho | `:95-112` |
| `compose(other)` | aplica uma mudança sobre um documento/mudança | `:236-337` |
| `transform(other, priority?)` | rebase de mudanças concorrentes (OT) | `:477-545` |
| `invert(base)` | produz a mudança inversa — base do undo | `:434-476` |
| `diff(other)` | deriva a mudança mínima entre dois documentos | `:339-399` |

Normalizações importantes: `push` funde operações adjacentes, coloca `insert` antes de `delete`
no mesmo índice e mantém `retain`+atributos separado (`:113-163`); `chop` remove `retain`
final sem atributos (`:165-171`). O construtor **assume ops bem formados** (`:57-60`, comentário
em `:59`) — não há validação de schema no Delta.

Comportamento observado (`exemplos/saida-delta.json`, `node delta-steps.cjs`):

```json
{"doc":"Olá + ' mundo'{bold} + '\\n'{header:1}", "length": 10,
 "edit":"retain 4, insert '!', delete 1, retain 5 {italic}",
 "compose":"Olá  + '!' + 'undo'{italic,bold} + '\\n'{italic,header:1}",
 "invert":"retain 4, insert 'm'{bold}, delete 1, retain 5 {italic:null}",
 "redo":"compose(compose(doc,edit), invert) == doc → true"}
```

Isso mostra duas propriedades centrais: atributos ausentes são herdados/compostos, e
`null` remove atributo (`AttributeMap.compose`, `05-quill-delta.ts` do pacote, e
`src/modules/keyboard.ts:268-302` para o uso em backspace). O `invert` precisa do documento
base; o `transform` é o que permite colaboração/OT — Quill **não** traz transporte nem
servidor, só as primitivas.

---

## 3. O que é um blot

Blot é a unidade de ligação entre o Delta e o DOM. Cada blot tem um `blotName`, um ou mais
`tagName`, uma `scope` (inline, block, block-embed), filhos permitidos e métodos
`create/formats/value/optimize`. A árvore de blots é paralela à árvore DOM; a raiz é o
`Scroll` (o `div.ql-editor`):

- `Scroll` é `ScrollBlot` de Parchment, com filhos permitidos `Block`, `BlockEmbed`,
  `Container` e filho padrão `Block`: `ref/quill/packages/quill/src/blots/scroll.ts:31-36`.
- `Block` é um `<p>` com filhos `Break`, `Inline`, `EmbedBlot`, `Text` e um `"\n"` implícito
  de comprimento 1: `src/blots/block.ts:14`, `:126-129`; `length()` soma
  `NEWLINE_LENGTH` (`:85-90`).
- `Inline` é o invólucro de formatação (`<strong>`, `<em>`, `<a>`, `<span class=...>`) com uma
  **ordem canônica de aninhamento** (`cursor < inline < link < underline < strike < italic <
  bold < script < code`): `src/blots/inline.ts:8-20`; `optimize` reescreve a árvore para
  respeitar essa ordem (`:44-58`).
- `Break` é o `<br>` de linha vazia, com `length() === 0` e remoção automática se houver
  irmãos: `src/blots/break.ts:1-25`.
- `Text` é o `TextBlot` de Parchment; o único código local é `escapeText` para HTML:
  `src/blots/text.ts:1-18`.
- Formatos concretos são subclasses: `Bold` (`<strong>` ou `<b>`, normaliza para o primeiro),
  `Header` (`h1..h6`), `Link` (`<a>` com sanitização), `List` (`<li data-list=...>` dentro de
  `<ol>`), `CodeBlock` (`div.ql-code-block` dentro de container):
  `src/formats/bold.ts:1-21`, `src/formats/header.ts:1-12`, `src/formats/link.ts:1-46`,
  `src/formats/list.ts:1-63`, `src/formats/code.ts:1-55`.

O blot também carrega dados que **não** estão no Delta: `List` cria um `<span class="ql-ui">`
para o marcador/checkbox e escuta `mousedown` para alternar `checked`:
`src/formats/list.ts:22-42`. O `ParentBlot.uiClass = 'ql-ui'` é global
(`src/core/quill.ts:34`); esse span é `contenteditable="false"` e posicionado por CSS.
Em code block com syntax, `code-token` é **excluído** dos Deltas por `bubbleFormats`
(`src/blots/block.ts:195-222`), isto é, highlight é dado derivado, não documento.

---

## 4. Como o DOM representa o documento (e onde ele escapa)

Fluxo de uma edição do usuário:

1. `keydown`/`beforeinput`/`composition` chegam ao `div.ql-editor`.
2. `Keyboard` casa bindings e, quando age, chama a API (`quill.format`, `updateContents`,
   `deleteText`): `src/modules/keyboard.ts:173-260`, `:334-352`; `Input` trata
   `beforeinput`/`replaceText`: `src/modules/input.ts:8-47`.
3. A API embrulha tudo em `modify`: captura a seleção, executa o modificador que devolve um
   `Delta`, reajusta a seleção via `shiftRange`/`transformPosition` e emite
   `EDITOR_CHANGE`/`text-change`: `src/core/quill.ts:881-919`, `:1009-1040`.
4. `Quill.updateContents` chama `editor.applyDelta` (`src/core/quill.ts:755-765`), que:
   normaliza CRLF, divide strings em linhas (`splitOpLines`), insere/remove no DOM, aplica
   formatos por `formatAt` e só no fim aplica os `delete` acumulados num `deleteDelta`
   (`src/core/editor.ts:28-123`).
5. O `MutationObserver` do Parchment dispara `scroll.update`; `Scroll.update` filtra mutações
   de blots "updatable" (embeds), emite `SCROLL_UPDATE` e o `Quill` reconstrói o Delta e emite
   `text-change` (`src/blots/scroll.ts:281-303`; `src/core/quill.ts:241-250`).
6. `Editor.update` tem um atalho para mutação de `characterData` ASCII (digitação): diff de
   texto dentro de um único blot, sem reconstruir o documento
   (`src/core/editor.ts:279-309`). Fora desse atalho, `getDelta()` inteiro + `diff`
   (`:310-315`).

Ou seja: **o DOM é a representação editável, o Delta é a representação canônica**. Essa
dualidade é a maior fonte de complexidade e de bugs de borda do Quill — cursor, IME, paste e
embeds existem para manter as duas árvores sincronizadas.

Onde o DOM não corresponde 1:1 ao Delta:

- Linha vazia é `<p><br></p>`, mas no Delta é apenas `"\n"`.
- Marcadores de lista e checkboxes moram em `.ql-ui` (DOM), não no Delta; o Delta guarda
  `{list:'ordered'}` no `"\n"` (`src/formats/list.ts:8-16`, `src/blots/scroll.ts:379-401`).
- `ql-cursor` (span com `\uFEFF`) existe só para formatar texto com seleção colapsada
  (`src/blots/cursor.ts:1-30`; `src/core/selection.ts:150-176`).
- `ql-tooltip` e `ql-picker` são chrome do tema, injetados dentro do container
  (`src/themes/snow.ts:106-125`; observado em `exemplos/saida-minimal-dom.html`).
- Syntax highlight (`ql-token`/`hljs`) é derivado e filtrado dos Deltas
  (`src/blots/block.ts:195-222`).

---

## 5. Onde o CSS entra (e por que isso importa para nós)

Quill **não é zero-CSS**. Há três folhas e uma hierarquia clara:

| Artefato | Conteúdo | Evidência |
|---|---|---|
| `quill.core.css` | o **mínimo funcional**: `.ql-container`, `.ql-editor` (`white-space: pre-wrap`, `outline: none`, `padding`, `overflow-y`), listas com `list-style: none` e marcadores via `.ql-ui:before` + counters, placeholder `.ql-blank::before` | `src/assets/core.styl:12-18`, `:40-52`, `:71-110`, `:208-215`; entrada de build em `webpack.common.cjs:41` |
| `quill.snow.css` | `@import './core'` + `./base` + toolbar/tooltip/pickers do tema snow; `.ql-container.ql-snow { border: 1px solid }` | `src/assets/snow.styl:1-18`; `webpack.common.cjs:43` |
| `quill.bubble.css` | o mesmo core/base + tooltip flutuante e preview de link | `src/assets/bubble.styl:1-40`; `webpack.common.cjs:42` |

O CSS "de tema" é separável do "estrutural": `base.styl` estiliza `.ql-editor h1..h6`,
`blockquote`, `code`, `img { max-width:100% }` e chrome de toolbar/pickers:
`src/assets/base.styl:117-152`, `:154-173`. O `core.styl` é o que o documento **precisa** para
ser editável de forma sã (espaços preservados, listas numeradas, placeholder). Sem ele, o
`contenteditable` continua funcionando, mas listas perdem marcador/indentação e o espaçamento
não é preservado — ver a ressalva de medição em
[`avaliacao.md#4-css-neutralidade-resposta-direta`](avaliacao.md#4-css-neutralidade-resposta-direta).

Custo em número de linhas (contado por `wc -l`, não é o CSS minificado): `core.styl` 215,
`base.styl` 309, `snow.styl` 18 + `snow/*` 79, `bubble.styl` 45 + `bubble/*` 63 — total 729.
Ou seja, `quill.core.css` é ~30% do CSS e os temas carregam core+base (~524 linhas de fonte)
antes de qualquer regra própria. Medido no probe: `.ql-editor` fica com
`white-space: pre-wrap`, `font-family: Helvetica, Arial, sans-serif`, `font-size: 13px`,
`p { margin: 0 }`; listas ficam com `list-style-type: none`, `ol { padding-left: 19.5px }` e
marcador por `.ql-ui:before` (`exemplos/saida-probe.json`, chaves `css.editor`/`css.list`).

Classes e contrato de estilo observáveis (não são API pública, mas são o que o CSS do app
precisa conhecer): `.ql-container`, `.ql-editor`, `.ql-editor.ql-blank`, `.ql-toolbar`,
`.ql-snow`/`.ql-bubble`, `.ql-ui`, `li[data-list]`, `.ql-indent-N`, `.ql-align-*`,
`.ql-direction-rtl`, `.ql-code-block-container`, `.ql-size-*`, `.ql-font-*`, `.ql-color-*`,
`.ql-bg-*`. O `theme` só adiciona classes; o app pode substituir as folhas por CSS próprio se
reimplementar os seletores estruturais. Isso é viável, mas é reimplementação, não "neutralidade
de estilo".

---

## 6. Exemplo mínimo real

[`exemplos/minimal.html`](exemplos/minimal.html) usa CDN 2.0.3 (mesma versão do `ref/`),
toolbar declarada em HTML e persistência em Delta:

```html
<link href="https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css" rel="stylesheet" />
<div id="toolbar">
  <button class="ql-bold" aria-label="Negrito"></button>
  <button class="ql-italic" aria-label="Itálico"></button>
  <button class="ql-link" aria-label="Link"></button>
</div>
<div id="editor"><p>Hello World!</p></div>
<script src="https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js"></script>
<script>
  const quill = new Quill('#editor', { theme: 'snow', modules: { toolbar: '#toolbar' } });
  const json = JSON.stringify(quill.getContents());   // persistência
  quill.setContents(JSON.parse(json));                // carga
</script>
```

Saída observada (headless Chromium 156, `--dump-dom`,
[`exemplos/saida-minimal-dom.html`](exemplos/saida-minimal-dom.html)): o container vira
`class="ql-container ql-snow"`, o editor vira `<div class="ql-editor" contenteditable="true">`
com os `<p>` originais preservados, a toolbar recebe `ql-toolbar ql-snow` + ícones SVG, e um
`<div class="ql-tooltip ql-hidden">` é injetado no container. O Delta inicial do HTML de
exemplo foi observado como:

```json
[{"insert":"Hello World!\nSome initial "},
 {"insert":"bold","attributes":{"bold":true}},
 {"insert":" text\n\n"}]
```

(`exemplos/saida-probe.json`, chave `loadDeltaOfInitialHTML`.)

---

## 7. O que o usuário experiencia

- **Init**: `new Quill(selector, {theme, modules})`; defaults ligam clipboard, keyboard,
  history e uploader (`src/core/quill.ts:80-91`), tema `default` sem chrome. A ordem de
  inicialização é rígida e instrutiva: container → `.ql-editor` → `Scroll` → `Editor` →
  `Selection` → `Composition` → tema → módulos (`:198-233`).
- **Primeira edição**: funciona como um contenteditable normal, mas o `text-change` entrega
  `Delta`s; digitação ASCII usa o atalho de `characterData` e não recalcula o documento.
- **Formatação**: `format(name,value)` escolhe `formatLine` ou `formatText` conforme o escopo
  do blot; seleção colapsada usa o `cursor` blot (`src/core/quill.ts:347-374`).
- **Seleção/IME**: modelo `Range {index,length}`; composição abre `batch` no Scroll para
  coalescer mutações e fecha em microtask (bug de WebKit documentado) —
  `src/core/composition.ts:15-45`, `src/core/selection.ts:113-132`. Observado: durante a
  composição o Delta **não** muda (o cenário do script parte do editor vazio, então o Delta
  fica `[{"insert":"\n"}]`); ao commitar, o texto entra de uma vez
  (`exemplos/saida-interact.json`, `imeDuringComposition` → vazio, `imeCommitted` →
  `[{"insert":"日本\n"}]`).
- **Undo/redo**: `History` guarda Deltas inversos + ranges, agrupa por `delay: 1000` e
  `maxStack: 100`; atalhos `Mod+Z`, `Mod+Shift+Z` e `Mod+Y` no Windows, além de
  `beforeinput` `historyUndo/historyRedo` (`src/modules/history.ts:26-82`). Observado:
  digitar → `Ctrl+B` → `Ctrl+Z` restaura o conteúdo anterior, `Ctrl+Shift+Z` refaz
  (`saida-interact.json`).
- **Clipboard**: cópia usa `getSemanticHTML`, não `innerHTML` (`src/modules/clipboard.ts:228-233`);
  colagem faz `DOMParser` + normalizadores Word/Google Docs + matchers, e sanitiza URLs
  (`javascript:` → `about:blank`) e imagens (`//:0`) — `src/formats/link.ts:5-27`,
  `src/formats/image.ts:37-40`. Observado: colar `<b>Bold</b>` + lista produz Delta com
  `bold`, `list: bullet` e `<ul><li>` no HTML semântico (`saida-interact.json`, `pasteHTML`).
- **Extensão**: `Quill.register()` aceita blot, attributor, módulo, tema; `formats: [...]`
  restringe o registry (`src/core/quill.ts:127-180`, `src/core/utils/createRegistryWithFormats.ts:5-40`).
  Observado: um `ClassAttributor` novo aplicado a texto vira `class="ql-highlight-true"` no DOM
  e `highlight:"true"` no Delta (`saida-probe.json`, `customFormat`).
- **readOnly**: desliga `contenteditable` e faz `modify` descartar fontes `USER`
  (`src/core/quill.ts:881-890`; `src/blots/scroll.ts:98-100`). Observado: teclado não altera;
  API altera (`saida-interact.json`, `readOnlyTyping`/`readOnlyApiStillWorks`).

---

## 8. Prós e contras para o `rich-editor-auy`

**A favor**

- Modelo de documento simples, plano, serializável e com operações formais (Delta), com
  `compose/transform/invert` já resolvidos — bom precedente para o nosso modelo em Rust.
- Extensão por registro em vez de fork; formatos, módulos e temas são substituíveis
  (`src/core/quill.ts:127-180`).
- Separação explícita entre core CSS e tema; `quill.core.css` é um artefato pequeno e
  identificável (`webpack.common.cjs:41`).
- Comportamento de borda (IME, paste Word/Docs, seleção após mutação) foi endurecido por anos
  e há testes de browser reais + fuzz.

**Contra**

- A fonte de verdade é mantida **em paralelo** ao DOM; toda a complexidade de `Selection`,
  `Composition`, `Cursor`, `MutationObserver` existe para reconciliar os dois.
- O CSS estrutural é obrigatório: listas, indentação, placeholder e `pre-wrap` vêm de
  `core.styl`; adotar Quill significa herdar classes e seletores.
- Alguns dados vivem fora do Delta (UI nodes, tooltip, syntax tokens, HTML semântico com
  `&nbsp;`), então "exportar o Delta" não é suficiente para fidelidade visual.
- Não é headless: o núcleo depende de `document`, `MutationObserver`, `Selection` e layout.
- Sem colaboração embutida: só as primitivas de OT; rede, presença e persistência ficam por
  conta do app.

**Copiar** — o formato de operações e a semântica de `compose/transform/invert`; a ideia de
blot como adaptador declarativo (nome, tag, escopo, filhos, `formats`, `optimize`); a separação
core-CSS/tema.

**Adaptar** — a noção de "documento plano + projeção" para um renderer headless: o Delta vira
uma sequência de ops no core Rust; o front-end recebe a projeção e devolve intenções, não HTML.
A sanitização de URL/imagem (`link.ts`, `image.ts`) e a normalização de colagem são boas listas
de casos.

**Evitar** — depender de `contenteditable` como fonte de verdade; replicar a dualidade
DOM/modelo sem necessidade; herdar o CSS inteiro; e assumir que o Delta cobre todo o estado
(UI nodes e dados derivados ficam fora).
