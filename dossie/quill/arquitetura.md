# Arquitetura do Quill 2.0.3

Este documento descreve o caminho real do código em `ref/quill/packages/quill/src`: como o
documento é representado, como o DOM é mantido em sincronia, como seleção/IME/histórico/
clipboard funcionam, e onde a extensão e o CSS entram. Números de linha se referem ao commit
`539cbff`. Cópias integrais dos arquivos centrais estão em
[`codigo/`](codigo/INDEX.md).

---

## 1. Mapa do repositório e do produto

- Monorepo npm workspaces `packages/*`: `ref/quill/package.json:16-18`, com apenas dois
  pacotes: `quill` (biblioteca) e `website` (documentação/playground). Não há app de edição.
- O pacote `quill` declara `main: quill.js` e `type: module`
  (`ref/quill/packages/quill/package.json:5-6`), com build `tsc` (declarações) + Babel +
  webpack (`scripts/build:1-17`).
- **Delta não está no monorepo**: `quill-delta@^5.1.0`
  (`ref/quill/packages/quill/package.json:13`), resolvido para 5.1.0
  (`ref/quill/package-lock.json:15819-15821`). O código de `Delta` usado neste dossiê foi
  copiado do tarball npm para [`codigo/05-quill-delta.ts`](codigo/05-quill-delta.ts).
- O núcleo também depende de `parchment@^3` (árvore de blots/registry), `lodash-es`
  (`merge`, `cloneDeep`, `isEqual`) e `eventemitter3`.

Distribuição de código: 64 `.ts`, 8.795 linhas; 8 blots, 11 módulos, 21 formatos; 729 linhas de
Stylus (`find src -name '*.ts'`; `ls src/{blots,modules,formats}`; `cat src/assets/**/*.styl`).

---

## 2. Inicialização: a ordem importa

`new Quill(container, options)` executa uma sequência rígida
(`src/core/quill.ts:198-233`):

1. `expandConfig` resolve seletor de container, tema, módulos com defaults, `bounds` e
   registry (`:199`; função em `:793-870`).
2. O container recebe a classe `ql-container`; o HTML inicial é lido e **apagado** do DOM
   (`:208-211`). Isso significa que o conteúdo inicial passa por `clipboard.convert` antes de
   virar Delta (`:265-272`), não é adotado como DOM.
3. Cria-se `div.ql-editor` como raiz (`:212`), que será o `Scroll` blot.
4. `Scroll` (subclasse de `ScrollBlot`) é criado sobre essa raiz com o `Emitter`
   (`:217-224`; `src/blots/scroll.ts:31-52`).
5. `Editor`, `Selection` e `Composition` são instanciados nessa ordem (`:225-227`); o
   `Editor` já calcula `this.delta = this.getDelta()` no construtor
   (`src/core/editor.ts:19-26`).
6. O tema é instanciado e injeta módulos; `keyboard`, `clipboard`, `history`, `uploader`,
   `input` e `uiNode` são sempre adicionados (`:228-234`). O tema pode registrar handlers de
   toolbar (`src/themes/base.ts:63-110`) e a classe `ql-snow`/`ql-bubble`
   (`src/themes/snow.ts:103`).
7. Listeners de `SCROLL_UPDATE` e `SCROLL_EMBED_UPDATE` ligam mutações do DOM a `modify` e a
   `editor.update` (`:236-264`).
8. Conteúdo inicial é aplicado com `setContents`, o histórico é limpo e `readOnly`/placeholder
   são configurados (`:265-282`).

Defaults: `theme: 'default'`, módulos `clipboard/keyboard/history/uploader: true`,
`readOnly:false`, `placeholder:''`, registry global (`:80-91`). Um tema `default` não injeta
toolbar nem tooltip; `snow` e `bubble` são registrados no entrypoint público
(`src/quill.ts:100-120`).

---

## 3. Modelo de documento

### 3.1 O Delta

Um Delta é `Op[]` onde cada op tem exatamente uma de `insert`, `delete` ou `retain`
(eventualmente com `attributes`). Não há schema, IDs estáveis nem hierarquia: blocos são
**atributos no caractere `"\n"`**. `Block.delta()` delega a `blockDelta`, que materializa
isso inserindo `'\n'` com os formatos do blot (`src/blots/block.ts:19-23`, `:180-193`).

Invariantes verificáveis:

- `Delta.length()` soma o comprimento de todas as ops, inclusive `delete` (o comprimento é o
  do documento base, não o resultante) — `05-quill-delta.ts:213-217`.
- `changeLength()` é a diferença líquida — `:202-211`.
- `push` funde ops adjacentes compatíveis e prioriza `insert` antes de `delete` no mesmo
  índice — `:113-163`; `chop` remove `retain` final — `:165-171`.
- `compose(other)` aplica `other` **sobre** `this` (documento ou mudança); `transform` rebaseia
  mudanças concorrentes; `invert(base)` produz o inverso de uma mudança em relação ao base
  (`:236-337`, `:434-545`).
- `diff` só funciona se ambos os lados forem documentos (todas as ops de `insert`);
  caso contrário lança `diff() called with non-document` — `:339-352`.

O construtor **não valida** as ops (`:59`: "Assume we are given a well formed ops"). A
validação fica no app. Atributos são objetos livres; `AttributeMap.compose/diff/invert/
transform` definem a semântica de merge e `null` para remover
(`05-quill-delta.ts` → `AttributeMap.ts:9-101`).

### 3.2 Linhas, blocos e embeds

`Scroll.length()` e `Block.length()` incluem o newline (`NEWLINE_LENGTH = 1`,
`src/blots/block.ts:14`, `:85-90`). Uma linha vazia tem comprimento 1 (o `"\n"`), mas o DOM
correspondente é `<p><br></p>`; `Break` tem `length() === 0` e é removido quando há irmãos
(`src/blots/break.ts:9-20`).

Embeds inline (imagem, fórmula, vídeo) são ops `insert: { image: "..." }`; embeds de bloco
(tabela, por exemplo) ocupam a linha inteira. `Scroll.deltaToRenderBlocks` separa os dois ao
renderizar (`src/blots/scroll.ts:322-377`), e `applyDelta` insere newlines implícitos quando o
embed inline cai no fim/início de linha (`src/core/editor.ts:51-92`).

---

## 4. Blots: a camada que liga Delta a DOM

Blot é um adaptador declarativo de Parchment. O contrato usado pelo Quill:

| Membro | Papel | Exemplo |
|---|---|---|
| `blotName`, `tagName`, `className` | identidade e tag(s) DOM | `Block.tagName='P'`; `CodeBlock.className='ql-code-block'` (`src/blots/block.ts:126-129`; `src/formats/code.ts:30-55`) |
| `scope` | inline/block/block-embed; governa `formatLine` vs `formatText` | `BlockEmbed.scope = Scope.BLOCK_BLOT` (`src/blots/block.ts:180`) |
| `allowedChildren`, `defaultChild`, `requiredContainer` | forma da árvore | `Scroll.allowedChildren = [Block, BlockEmbed, Container]` (`src/blots/scroll.ts:35-36`); `ListItem.requiredContainer = ListContainer` (`src/formats/list.ts:56-57`) |
| `create/formats/value` | Delta/atributo ↔ DOM | `Header.formats` converte tag em número (`src/formats/header.ts:6-9`) |
| `optimize` | normaliza a árvore após mutações | `Break.optimize` remove `<br>` redundante (`src/blots/break.ts:9-13`); `Inline.optimize` reordena aninhamento (`src/blots/inline.ts:44-58`) |
| `attachUI` / `uiClass` | nós de UI fora do modelo | checkbox/lista em `.ql-ui` (`src/core/quill.ts:34`; `src/formats/list.ts:22-42`) |

A árvore é mantida por Parchment (externo); o Quill adiciona semântica de documento
arredondando comprimentos (`Block.length`), cacheando Delta por bloco (`Block.cache`,
`src/blots/block.ts:17-26`) e filtrando formatos derivados (`code-token`) em `bubbleFormats`
(`:195-222`).

Ordem de aninhamento inline é **normativa**, não estética: `Inline.order` define que `<a>` fica
por dentro de `strong`/`em`, etc.; `compare` é usado por `formatAt` para decidir se isola e
envolve o trecho (`src/blots/inline.ts:8-42`). Isso evita HTML inválido e mantém o DOM
determinístico para o diff.

---

## 5. Fluxo de edição: duas rotas

### 5.1 Entrada do usuário → Delta

- `Keyboard.listen` intercepta `keydown` no `ql-editor`, monta um `Context` (collapsed, empty,
  offset, prefix, suffix, format, line) e casa bindings por key/modificadores/contexto; se um
  handler devolve diferente de `true`, chama `preventDefault`
  (`src/modules/keyboard.ts:173-260`).
- Backspace/Delete são tratados manualmente para preservar formatos ao juntar linhas:
  `AttributeMap.diff` entre a linha atual e a vizinha e `retain` com atributos
  (`:268-327`). `handleEnter` insere `'\n'` com os formatos de bloco herdados (`:334-352`).
- `Input` complementa com `beforeinput` para `insertText`/`insertReplacementText` e
  `deleteContentBackward/Forward`, evitando depender do comportamento do browser
  (`src/modules/input.ts:8-47`).
- Digitação normal **não** passa por essas rotas: o browser muta o texto, o
  `MutationObserver` dispara e `Editor.update` faz o diff.

### 5.2 API → DOM (`modify` + `applyDelta`)

Toda mutação programática (`insertText`, `deleteText`, `format*`, `setContents`,
`updateContents`) passa por `modify` (`src/core/quill.ts:881-919`):

1. Bloqueia fontes `USER` quando `readOnly` e não é `editReadOnly`.
2. Guarda a seleção atual (`getSelection`) e o Delta antigo (`editor.delta`).
3. Executa o modificador, que retorna um Delta.
4. Reposiciona a seleção com `shiftRange` — usa `transformPosition` quando há Delta e
   `priority = source !== USER` (`:1009-1040`).
5. Emite `EDITOR_CHANGE` e, se a fonte não é `SILENT`, `text-change`.

`updateContents` chama `editor.applyDelta` (`:755-765`), que:

- normaliza `\r\n`/`\r` para `\n` (`normalizeDelta`, `src/core/editor.ts:451-463`);
- divide strings em linhas (`splitOpLines`, `:465-482`);
- percorre as ops inserindo no `Scroll` e formatando (`scroll.formatAt`) por atributo;
- acumula as remoções num `deleteDelta` e só as aplica ao final, para não deslocar índices
  (`:33`, `:95`, `:113-120`);
- aplica `scroll.optimize()` e retorna `update(normalizedDelta)` (`:121-122`).

`Editor.applyDelta` é o ponto mais delicado: ele precisa lidar com newline implícito antes/
depois de embeds e com atributos implícitos herdados da linha (`AttributeMap.diff`), porque
um `insert` de texto sem atributos **herda** a formatação do ponto de inserção
(`:48-92`).

### 5.3 DOM → Delta (`MutationObserver` + `Editor.update`)

`Scroll.update` drena `observer.takeRecords()`, ignora mutações dentro de blots "updatable"
(embeds que gerenciam seu próprio conteúdo, como vídeo/fórmula) e emite
`SCROLL_BEFORE_UPDATE`/`SCROLL_UPDATE` (`src/blots/scroll.ts:281-303`). O Quill reage com
`modify(() => editor.update(null, mutations, selectionInfo))` (`src/core/quill.ts:241-250`).

`Editor.update` (`src/core/editor.ts:273-315`) tem dois caminhos:

- **Atalho**: exatamente uma mutação `characterData`, alvo ASCII e dentro de um blot →
  diff apenas entre o texto antigo e o novo daquele leaf, com os formatos do blot
  (`:279-309`). É o caminho da digitação comum; evita reconstruir o documento.
- **Geral**: `this.delta = this.getDelta()` (percorre todas as linhas) e, se o Delta antigo
  composto com a mudança não é igual ao novo, deriva `oldDelta.diff(this.delta,
  selectionInfo)` (`:310-315`). Ou seja, o custo é O(documento) fora do atalho.

Depois disso, o `Quill`/`Scroll` atualizam a seleção e emitem eventos; a ordem é escolhida
para que a seleção seja corrigida antes de `selection.update` (`src/core/quill.ts:748-753`).

---

## 6. Seleção, cursor e IME

`Range` é `{index, length}` no espaço do documento (`src/core/selection.ts:31-36`); não há
âncora em nós DOM. A tradução é feita por:

- `normalizedToRange`: encontra o blot de cada nó, soma `offset` e `blot.index(node,
  offset)`; o fim é limitado a `scroll.length()-1` (`:272-292`).
- `rangeToNative`: para cada extremidade, `scroll.leaf(index)` e `leaf.position(offset)`
  (`:341-353`).
- `setNativeRange`: cria `document.createRange`, trata `<br>` como índice entre irmãos,
  remove todas as ranges e adiciona a nova (`:355-418`).

Detalhes que denunciam o custo de `contenteditable`:

- `savedRange` guarda a última seleção não nula; `focus()` restaura essa range
  (`:44-61`, `:143-148`).
- `selectionchange` global é escutado no `document`, com `mouseDown`/`composing` como guardas
  e `setTimeout(..., 1)` (`:62-67`). O `Emitter` global escuta `selectionchange`, `mousedown`,
  `mouseup` e `click` e despacha para todas as instâncias `.ql-container`
  (`src/core/emitter.ts:6-18`).
- Antes de mutações do `Scroll`, a seleção nativa é salva; após `SCROLL_UPDATE`, é restaurada
  se os nós ainda pertencem à raiz (`:69-110`).
- **Seleção colapsada com formato**: como não há texto selecionado, um blot `cursor` (span
  com `\uFEFF`, `contenteditable=false`) é inserido no ponto, formatado, e depois removido
  quando o usuário digita (`src/blots/cursor.ts:1-30`; `src/core/selection.ts:150-176`).
- **IME**: `Composition` escuta `compositionstart/end` na raiz; no início emite
  `COMPOSITION_BEFORE_START`, `scroll.batchStart()` e `COMPOSITION_START`; no fim fecha o
  batch em `queueMicrotask` por causa de um bug de WebKit (`src/core/composition.ts:15-45`).
  `Selection` marca `composing`, e `Syntax` evita highlight durante composição
  (`src/modules/syntax.ts:273-275`). Observado: durante a composição o Delta permanece
  inalterado; ao commitar, o texto entra inteiro (`exemplos/saida-interact.json`).

O `uiNode` mitiga um caso específico de navegação de caret em torno de `.ql-ui` (checkbox,
contador de lista) com um TTL de 100 ms (`src/modules/uiNode.ts:8`, deadline em `:83-100`), evidência de que
nós de UI dentro de `contenteditable` são uma fonte recorrente de bugs.

---

## 7. History

`History` é um módulo, não core (`src/modules/history.ts:24-33`). Mantém `undo`/`redo` como
pilhas de `{delta, range}` e se alimenta de `EDITOR_CHANGE` (`:36-62`).

- `record(changeDelta, oldDelta)` calcula `undoDelta = changeDelta.invert(oldDelta)` e
  **funde** com o item anterior quando `now < lastRecorded + delay` (default 1000 ms), o que
  transforma digitação contínua num único undo (`:111-137`; defaults `:26-31`).
- `change('undo','redo')` faz `item.delta.invert(quill.getContents())` para empilhar o inverso,
  aplica `updateContents` com `ignoreChange=true` e restaura a seleção (`:85-109`).
- Atalhos: `Mod+Z`, `Mod+Shift+Z`, `Mod+Y` no Windows (só quando `navigator.platform` casa
  `/Win/i`) e `beforeinput` `historyUndo/Redo` (`:59-82`).
- `transform(delta)` rebaseia pilhas quando chega mudança remota; `transformStack` usa
  `remoteDelta.transform(oldItem.delta, true)` e remove itens que ficaram vazios (`:139-159`,
  `:161-180`). Isso é a base para colaboração, mas **não há** transporte nem merge de
  presença.
- `userOnly` (default `false`, `:28`) ignora mudanças de fonte `API` no undo quando ligado;
  o E2E liga a opção em runtime e verifica o efeito (`test/e2e/history.spec.ts:17-44`).
- O range é restaurado quando existe; senão, `getLastChangeIndex` posiciona o caret no fim da
  mudança (`:149-158`, `:182-208`).

Limite conhecido: a pilha é linear e o inverso é sempre relativo ao documento atual. Nada
persiste entre sessões; recarregar a página perde o histórico.

---

## 8. Clipboard, colagem e upload

### 8.1 Cópia

`onCaptureCopy` intercepta `copy`/`cut`, impede o comportamento nativo e escreve
`text/plain` (`getText`) + `text/html` (`getSemanticHTML`) no `clipboardData`
(`src/modules/clipboard.ts:171-186`). Em `cut`, chama `deleteRange`
(`:176-179`; implementação em `src/modules/keyboard.ts:802-816`).

`getSemanticHTML` (`src/core/quill.ts:543-552`, que delega a `editor.getHTML` -- `src/core/editor.ts:198-206` -- e a `convertHTML` `:363-456`) reconstrói tags a
partir dos blots: texto escapado com `&nbsp;` para espaços, listas com `<ol>/<ul>` e
`data-list` para checklists, tabelas com `style="border: 1px solid #000;"` forçado. É uma
projeção **sem garantia de fidelidade** ao HTML original — observado:
`<p>Hello World!</p>` virou `<p>Hello&nbsp;World!</p>` (`exemplos/saida-probe.json`,
`afterEditsHTML`; `exemplos/saida-interact.json`, `pasteHTML`).

### 8.2 Colagem

`onCapturePaste` (`src/modules/clipboard.ts:197-226`) prefere `text/html`, cai para
`text/plain`, usa `text/uri-list` se não houver nenhum, e desvia arquivos para o
`Uploader` (quando não há HTML ou quando o HTML é só um `<img>`).

`convert({html,text})` (`:100-121`) e `convertHTML` (`:127-142`) fazem:

1. `DOMParser().parseFromString(html,'text/html')`.
2. `normalizeHTML(doc)` → normalizadores de **Word** e **Google Docs**
   (`src/modules/normalizeExternalHTML/index.ts:1-12`).
3. `prepareMatching` monta matchers por seletor (`:253-283`).
4. `traverse` pós-ordem: texto → matchers de texto; elementos → filhos, depois matchers de
   elemento (`:381-419`).

Os matchers built-in cobrem `br`, `li` (indent), `ol/ul` (list/checklist), `pre` (code
block), `tr` (tabela), `b/i/strike` (aliases), estilos inline
(`font-weight`, `font-style`, `text-decoration`, `text-indent`) e atributos/classes
reconhecidos por Parchment (`:32-44`, `:579-626`). Texto passa por normalização agressiva:
NBSP vira espaço, espaços consecutivos colapsam, espaços de borda de blocos são removidos, e
`<o:p>` do Word é tratado (`matchText`, `:627-669`).

Segurança: `Link.sanitize` usa whitelist `http/https/mailto/tel/sms` e devolve
`about:blank` caso contrário; `Image.sanitize` aceita `http/https/data` e devolve `//:0`
(`src/formats/link.ts:5-27`; `src/formats/image.ts:37-40`). Observado: colar
`<a href="javascript:alert(1)">` produziu `href="about:blank"` e `<img onerror>` foi
descartado; `src` inválido virou `//:0` (`exemplos/saida-probe.json`, `urlSanitizedHTML`).
Não há sanitização de atributos `style` arbitrários no HTML de entrada além dos matchers.

`dangerouslyPasteHTML` aplica `convert` e `setContents`/`updateContents` diretamente
(`:145-170`); o nome é explícito sobre o risco.

### 8.3 Uploader

`Uploader` aceita `drop` e arquivos do clipboard, filtra por `mimetypes` (default
`image/png`, `image/jpeg`) e, por default, converte para **data URL** via `FileReader`
(`src/modules/uploader.ts:1-83`). O handler é substituível; a API não implementa upload
remoto, progresso ou cancelamento.

---

## 9. Extensão

Quatro mecanismos, em ordem de custo:

1. **Attributor/format**: registrar uma instância de `ClassAttributor`/`StyleAttributor`
   (`Quill.register(attributor)`), como no exemplo observado: um `ClassAttributor('highlight',
   'ql-highlight')` gerou `class="ql-highlight-true"` no DOM e `highlight:"true"` no Delta
   (`exemplos/saida-probe.json`, `customFormat`). Para mudar tag/semântica, é preciso uma
   subclasse de `Inline`/`Block` com `blotName`, `tagName`, `formats`, `create`.
2. **Blot**: `Quill.register('formats/meu', MeuBlot)`; como `register` também registra no
   registry global (`src/core/quill.ts:127-180`). O `formats: [...]` do construtor restringe o
   registry e inclui os `requiredContainer` transitivamente, detectando ciclos
   (`src/core/utils/createRegistryWithFormats.ts:5-40`).
3. **Módulo**: registrar `modules/meu` e pedir no `modules` do construtor; `Theme.addModule`
   instancia via `Quill.import('modules/name')` e mescla defaults
   (`src/core/theme.ts:30-52`). Módulos recebem `(quill, options)` e podem escutar
   `Emitter.events`/eventos DOM.
4. **Tema**: registrar `themes/meu`; `expandConfig` resolve o tema antes de instanciar
   (`src/core/quill.ts:802-811`). O tema controla chrome (toolbar, tooltip, pickers) e pode
   sobrescrever handlers (`src/themes/base.ts:63-110`, `:193-230`).

Exemplos reais de extensão no próprio core:

- `Table`/`tableEmbed` implementam tabela como container + embeds com atributos próprios e
  registram handlers de embed no Delta (`src/modules/tableEmbed.ts:259`).
- `Syntax` estende `CodeBlock` e adiciona `code-token` como format inline, **filtrando** o
  token dos Deltas (`src/modules/syntax.ts:14-70`, `:117-143`; filtro em
  `src/blots/block.ts:195-222`). É o modelo de "dado derivado que não é documento".
- `List` usa `uiNode` (span) para checkbox/marcador e alterna `data-list`
  (`src/formats/list.ts:22-42`).

O que **não** existe como extensão de primeira classe: schema/validação de atributos,
serialização alternativa, colaboração, virtualização, ou um sistema de plugins com ciclo de
vida declarado (mount/unmount só via eventos `SCROLL_BLOT_MOUNT/UNMOUNT`).

---

## 10. Renderer, DOM e CSS

O renderer é o browser: `contenteditable` + HTML. O Quill não desenha; ele mantém a árvore de
blots em sincronia com o Delta e delega layout/estilo ao CSS.

Classes e estrutura observadas:

- Container: `ql-container` + `ql-snow`/`ql-bubble` (`src/core/quill.ts:209`;
  `src/themes/snow.ts:103`).
- Editor: `ql-editor` + `ql-blank` quando vazio (toggle no listener de `EDITOR_CHANGE`,
  `src/core/quill.ts:236-240`; `Editor.isBlank` em `src/core/editor.ts:245-253`).
- Listas: `ol > li[data-list]`, com `.ql-ui` para marcador/checkbox; numeração por
  `counter-reset/increment` e `.ql-ui:before` (`src/assets/core.styl:71-110`).
- Indentação/align/direction: `ql-indent-N`, `ql-align-*`, `ql-direction-rtl` via class
  attributors (`src/formats/indent.ts:29-33`; `src/formats/align.ts:7-13`).
- Code: `div.ql-code-block-container > div.ql-code-block` (`src/formats/code.ts:45-50`).
- Placeholder: `.ql-editor.ql-blank::before { content: attr(data-placeholder) }`
  (`src/assets/core.styl:208-215`).

Três folhas com papéis distintos: `core.css` (estrutural, exigido pelo documento),
`base.css` (estética compartilhada de chrome e tipografia do editor) e `snow`/`bubble`
(chrome específico). `snow.styl` e `bubble.styl` **importam** core e base
(`src/assets/snow.styl:8-10`; `src/assets/bubble.styl:8-10`), então usar `snow.css` significa
levar tudo. O artefato `quill.core.css` é um entry separado do webpack
(`webpack.common.cjs:41-43`) — é o único caminho para "só o mínimo" sem reescrever CSS.

O CSS não é apenas cosmético: `white-space: pre-wrap` preserva espaços; `outline:none` remove
o foco padrão (o foco visível passa a depender do CSS do app); `li { list-style-type: none }`
+ counters produzem a numeração; `.ql-ui` é `position:absolute` para tirar o checkbox do fluxo
do texto (`src/assets/core.styl:40-110`, `:205-206`). Sem `core.css`, o DOM continua editável,
mas listas/placeholder/espaçamento degradam.

Tema `bubble` não tem toolbar fixa: o tooltip é posicionado sobre a seleção e vira toolbar
contextual (`src/themes/bubble.ts:17-55`), o que muda o modelo de interação, não o documento.

---

## 11. Acessibilidade

O que existe:

- Toolbar: `role="toolbar"` e botões com `aria-label`/`aria-pressed`, atualizados a cada
  mudança de seleção (`src/modules/toolbar.ts:34`, `:166-200`; teste
  `test/unit/modules/toolbar.spec.ts:180-193`).
- Picker: `role="button"`, `aria-expanded`, `aria-hidden`, `aria-controls`
  (`src/ui/picker.ts:45-108`).
- Checkbox de lista: `.ql-ui` com `contenteditable="false"` e `cursor:pointer`, alternável por
  mouse/touch (`src/formats/list.ts:22-42`; `src/assets/core.styl:24-28`).
- Código: `spellcheck="false"` no container (`src/formats/code.ts:8-13`).
- Teclado: atalhos de formatação, navegação em listas/tabelas/embeds, Tab/Shift+Tab para
  indentar (`src/modules/keyboard.ts:355-700`), e `deleteRange` preservando formatos
  (`:802-816`).
- Foco: tooltip restaura o foco no editor ao cancelar/salvar (`src/themes/base.ts:273-275`).

O que **não** existe (observado e verificado no código):

- A raiz editável **não tem `role` nem `aria-label`**: `rootRole` e `rootAriaLabel` são `null`
  no probe (`exemplos/saida-probe.json`). O `div[contenteditable]` fica dependente de o
  leitor de tela inferir "textbox"; não há `aria-multiline`, `aria-labelledby` ou associação
  com o container.
- Não há regiões `aria-live` para anúncio de mudanças; `text-change` é evento JS.
- A toolbar não é ligada ao editor por `aria-controls`; é apenas um `role=toolbar` irmão.
- A ordem de tabulação inclui o `div.ql-editor` e os controles da toolbar, mas não há
  gerenciamento de foco por setas.
- Listas/checklists são `span.ql-ui` com `contenteditable=false`; a semântica de checkbox não
  é exposta (`role=checkbox`/`aria-checked` ausentes).
- O placeholder é CSS `::before`, não texto acessível.

Testes de a11y existem, mas cobrem só o chrome: `aria-pressed` da toolbar
(`test/unit/modules/toolbar.spec.ts:180-193`) e `aria-expanded`/`aria-hidden`/`aria-controls`
do picker (`test/unit/ui/picker.spec.ts:44-129`); o resto dos matches de `aria|role` em `test/`
é HTML de fixture. Não há teste do nó editável nem teste com leitor de tela.

---

## 12. Performance

Mecanismos:

- `Block.cache` evita recalcular `delta()`/`length()` enquanto o bloco não muda
  (`src/blots/block.ts:17-26`, `:28-90`).
- `Editor.update` tem o atalho de `characterData` para digitação (`src/core/editor.ts:279-309`).
- `Scroll.update` evita `document.getSelection()` quando a raiz não está conectada e filtra
  mutações de embeds (`src/core/selection.ts:251-264`; `src/blots/scroll.ts:294-301`).
- Batching: `batchStart/batchEnd` coalesce mutações durante `applyDelta`, `insertContents` e
  composição (`src/blots/scroll.ts:54-66`; `src/core/editor.ts:30-121`).

Números observados (Chromium 156 headless, CDN 2.0.3, sem tuning; não é benchmark oficial):

| Cenário | Medida |
|---|---|
| `setContents` com 1.000 parágrafos (48.890 caracteres) | ~37 ms |
| `getContents` do mesmo documento | ~0,1 ms |
| 200 `insertText` de API no início do documento | ~335 ms (~1,7 ms/insert) |
| Digitação real de 120 caracteres (`keyboard.type`, delay 0) | ~247 ms |
| Digitação de "Olá mundo" (10 caracteres) | ~27 ms (medição armazenada; varia entre execuções) |

Fonte: [`exemplos/saida-probe.json`](exemplos/saida-probe.json) (`bench`) e
[`exemplos/saida-interact.json`](exemplos/saida-interact.json) (`typing`, `typing120`).
Limites: não há virtualização; cada parágrafo é um nó DOM; `applyDelta` de um documento inteiro
insere nó a nó e `getDelta` percorre todas as linhas. Documentos muito grandes (dezenas de
milhares de blocos) tendem a sofrer com layout e com o custo O(n) de `update` fora do atalho.
Não executei teste de estresse; o diretório `test/fuzz` existe e é a referência do projeto
para robustez (`ref/quill/packages/quill/test/fuzz/editor.spec.ts`).

---

## 13. Serialização e persistência

- **Formato canônico**: `quill.getContents()` → `Delta` (JSON `{ops:[...]}`). É o que deve ser
  persistido.
- **Carga**: `quill.setContents(delta)` — apaga tudo, insere o novo e remove o `\n` extra do
  editor vazio; retorna a mudança composta (`src/core/quill.ts:702-720`).
- **Atualização incremental**: `updateContents(delta)` aplica um Delta e retorna a mudança
  efetiva (`:755-765`).
- **HTML**: `getSemanticHTML()` é para exportação/colagem; não preserva o HTML de entrada
  (normaliza espaços para `&nbsp;`, escolhe tags canônicas, força borda em tabela, remove
  atributos desconhecidos). Observado em `exemplos/saida-probe.json`.
- **Estado não serializado**: stacking de UI, tooltip, pickers, posição de scroll, seleção,
  highlight de sintaxe, histórico. O Delta não contém IDs de bloco nem metadados de aplicação.

---

## 14. Limitações e "dados fora do Delta"

1. **Nós de UI** (`.ql-ui`, checkbox de lista) e chrome (tooltip/pickers) não estão no Delta;
   a posição/estado vem do DOM e do blot. Ao serializar, eles desaparecem e são recriados.
2. **Dados derivados**: `code-token` (syntax) é explicitamente filtrado do Delta; qualquer
   transformação visual precisa ser recalculável a partir do texto.
3. **Atributos não validados**: o Delta aceita qualquer atributo; `scroll.query` decide o que
   vira DOM e o que é ignorado. Não há schema declarativo como no ProseMirror.
4. **HTML não é fiel**: conversão de ida e volta (`getSemanticHTML` → `clipboard.convert`)
   altera espaços, tags e atributos; não use HTML como storage.
5. **Colaboração**: só primitivas (`transform`, `invert`, `transformStack`); sem protocolo,
   sem resolução de autoria, sem cursores remotos.
6. **IME/selection**: dependem de APIs do browser e de hacks (microtask no WebKit, cursor
   blot, TTL de seleção em `uiNode`); comportamento varia por navegador.
7. **readOnly é de fachada**: bloqueia `USER` em `modify` e desliga `contenteditable`, mas a
   API continua editando; não é sandbox.
8. **Sem schema/validação de entrada**: `dangerouslyPasteHTML` e `clipboard.convert` aceitam
   HTML arbitrário e dependem dos matchers para descartar o que não conhecem; a sanitização
   cobre URL de link/imagem, não CSS/atributos arbitrários.
9. **Sem virtualização/particionamento**: um documento = uma árvore DOM.
10. **CSS obrigatório**: a apresentação correta de listas, placeholder e espaços exige
    `core.css` ou reimplementação dos seletores.

---

## 15. Build, testes e manutenção

- Build: `tsc` para `.d.ts`, Babel para JS, webpack para bundles e CSS
  (`ref/quill/packages/quill/scripts/build:1-17`; `webpack.common.cjs:38-44`).
- Testes unitários em **browser real** via Vitest + Playwright, Chromium por default
  (`test/unit/vitest.config.ts:22-27`); E2E em `test/e2e` com fixtures de teclado, clipboard e
  composição (`test/e2e/fixtures/Composition.ts:1-50`); fuzz em `test/fuzz`.
- Cobertura de comportamento: 107 `test(` em `editor.spec.ts`, 45 em `selection.spec.ts`,
  48 em `clipboard.spec.ts`, 21 em `history.spec.ts` (contagem por `grep -c`).
- Manutenção: 2.0 GA em 2024, releases 2.0.1–2.0.3 com correções de IME/Safari/clipboard
  (`ref/quill/CHANGELOG.md:1-45`). O projeto é mantido pela Slab e tem CI
  (`.github/workflows`). Não há LTS.

---

## 16. O que isso implica para o `rich-editor-auy`

- O par **Delta canônico + projeção** é o desenho certo para um core headless em Rust; o Quill
  mostra o custo de manter duas árvores quando a projeção é mutável por fora (DOM). No nosso
  caso, a projeção no front-end pode ser somente leitura e as intenções voltam como ops.
- A semântica de `compose/transform/invert` e a regra "`\n` carrega atributos de bloco" são
  diretamente aproveitáveis; já a ausência de schema/validação é uma lacuna que devemos
  fechar (tipos de atributo, whitelist, validação de op).
- A separação `core.css` vs tema é o precedente mais útil: existe um conjunto mínimo
  estrutural que o renderer precisa garantir; o resto é opinião de produto.
- O que **não** copiar: depender de `contenteditable` como fonte de verdade, o modelo de
  seleção baseado em índices sem âncoras (funciona, mas exige o cursor blot e hacks de IME),
  e a ausência de acessibilidade no nó editável.
