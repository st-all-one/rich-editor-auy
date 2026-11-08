# Arquitetura — MarkText / Muya

Todas as citações apontam para `ref/marktext/`. Commit `a520bef`, `@muyajs/core` 0.2.0, MIT.
Não houve execução de build/testes: o clone não tem `node_modules/`.

---

## 0. Mapa mental

```
                      ┌──────────────── Markdown (arquivo em disco) ───────────────┐
                      │                                                           │
                  MarkdownToState                                            StateToMarkdown
                      │                                                           │
                      ▼                                                           ▲
        ┌─────────────────────────── TState[] (árvore de blocos) ────────────────┐
        │  JSONState (fonte da verdade em memória, sobre ot-json1)               │
        └───────────────┬────────────────────────────────────────────────────────┘
                        │ operações ot-json1 / ot-text-unicode
                        ▼
            EditTransactions (batch, regras, histórico, commit)
                        │
                        ▼
        ScrollPage → Parent → Content/Format   (árvore de blocos viva)
                        │
                        ▼
         InlineEngine.patch() → tokens inline → snabbdom → innerHTML
                        │
                        ▼
                  DOM contenteditable
```

Duas árvores coexistem: a **árvore de estado** (`TState[]`, serializável) e a **árvore de blocos
viva** (`ScrollPage`/`Parent`/`Content`, com `domNode`). A ponte é por **caminho JSON**
(`TBlockPath`), e é por caminho que toda edição é endereçada.

---

## 1. Modelo de dados

### 1.1 União discriminada, não schema

O documento é `TState[]`, a união de **21 formas** — 11 folhas (`TLeafState`) e 10 contêineres
(`TContainerState`), `src/state/types.ts:208-233`. Cada nó é uma interface com `name` literal,
declaradas em `types.ts:4-206`:

```ts
export interface IParagraphState {           // :4
    name: 'paragraph';
    text: string;
}
export interface IAtxHeadingState {          // :9
    name: 'atx-heading';
    meta: { level: number };
    // The complete editable line, including opening/closing markers and whitespace.
    text: string;
}
export interface ICodeBlockState {           // :32
    name: 'code-block';
    meta: { type: string; lang: string; fenceLength?: number; fenceChar?: '`' | '~' };
    text: string;
}
```

Características que importam para nós:

- **Folhas guardam Markdown cru.** Um heading ATX guarda a linha inteira, com `#` e espaços
  (`types.ts:9-14`); um Setext guarda também a linha de sublinhado (`types.ts:16-23`); um bloco de
  código guarda o fence char e o comprimento (`types.ts:32-43`). Isso compra round-trip fiel e
  vende estrutura: o nível de heading convive com o marcador no mesmo nó.
- **Contêineres têm `children: TState[]`** e metadados de sintaxe: `block-quote`,
  `github-alert`, `order-list` (com `start`, `loose`, `delimiter`, `style`, `sourceMarkers`),
  `bullet-list` (com `marker` e `loose`), `task-list`, `table`/`table.row`/`table.cell`,
  `footnote` (`types.ts:120-206`).
- **A discriminação é por string comparada em runtime**, com predicados gerados à mão
  (`isParagraphState`, `isAtxHeadingState`, … `types.ts:244-265`) e `isStateOfName()` genérico
  (`types.ts:237-242`). Não há validação de esquema: um `TState` malformado só falha no render.
- **Definições de referência não são nó de primeira classe.** Existem como `paragraph` cujo texto é
  a linha `[label]: url "title"`; a interface `ILinkReferenceDefinitionState` é explicitamente
  marcada `@deprecated` e descrita como não usada (`types.ts:51-62`).

### 1.2 Dois níveis de bloco: `Parent` e `Content`, e o caso `Format`

- `TreeNode` é a base: ponteiros `prev`/`next`/`parent`, `domNode`, `blockName` estático, e a
  marcação `domNode[BLOCK_DOM_PROPERTY] = this` para recuperar o bloco a partir de um nó DOM
  (`ref/marktext/packages/muya/src/block/base/treeNode.ts:1-100`).
- `Parent` mantém `children` e `attachments` em listas ligadas próprias; `attachments` existe para
  ícones/checkbox que **não** fazem parte do estado serializado
  (`src/block/base/parent.ts:15-19`).
- `Content` é a folha: um `span.mu-content` com `contenteditable` e um único `string`
  (`src/block/base/content.ts:356-433`).
- `Format extends Content` (`src/block/base/format.ts:328`) e é onde vive quase toda a edição
  inline: pareamento automático, Enter/Backspace, formatação, imagens, links.

O `path` da folha é derivado do pai + `'text'`, com uma exceção: `language-input` escreve em
`meta.lang` (`content.ts:380-391,395-398`). Ou seja, **o caminho no JSON é a identidade de edição**.

### 1.3 Blocos registrados

39 registros em `ScrollPage.register(...)` (`src/block/index.ts:41-81`), cobrindo CommonMark
(parágrafo, ATX/Setext, thematic break, code fence, HTML), GFM (tabela, task list) e extras
(math, frontmatter, diagramas, footnote, github alert). O registro é um `Map<string, Constructor>`
global por classe (`src/block/scrollPage/index.ts:25-40`); nome desconhecido → `warn` e `undefined`.

---

## 2. Parser: Markdown → estado

`MarkdownToState` (`src/state/markdownToState.ts:56`) delega o **lexing de blocos** a `marked`,
configurado com extensões por opção (`src/utils/marked/lexBlock.ts:17-80`): math (`$`, GFM,
backslash), footnote, multiline blockquote, fancy lists, frontmatter, GitHub alerts.

Depois converte o fluxo de tokens numa árvore com uma **pilha de pais**
(`markdownToState.ts:90-101`):

```ts
const parentList: TState[][] = [states];
while ((token = tokens.shift())) {
    if (CONTAINER_TOKEN_TYPES.has(token.type))
        this._handleContainerToken(token, parentList, tokens);
    else
        this._handleLeafToken(token, parentList, tokens, trimUnnecessaryCodeBlockEmptyLines);
}
```

O truque é a **sentinela sintética `block-end`**: ao abrir um contêiner, o parser empurra um
`block-end` e os tokens filhos no topo da fila; ao encontrá-lo, faz `parentList.shift()`
(`markdownToState.ts:112-131`). Contêiner vazio é corrigido para um parágrafo vazio (fix #1735,
`markdownToState.ts:114-130`). O handler inteiro é `_handleContainerToken` (`:105-281`), com os
demais tipos de contêiner empilhando o próprio `block-end`.

Consequências:
- O parser **não é** o mesmo da renderização inline: blocos vêm de `marked`; inline vem de um lexer
  próprio (`src/inline/parser/lexer.ts`).
- Toda a forma do bloco é decidida na importação; não há "nó genérico" que virará bloco depois.
- Documento vazio vira `[{ name: 'paragraph', text: '' }]` (`markdownToState.ts:101`).

## 3. Serializador: estado → Markdown

`ExportMarkdown` (`src/state/stateToMarkdown.ts:67`) percorre o estado por tipo de bloco, com
**estado interno de listas abertas** (`_listType`, `_isLooseParentList`) e um resolvedor de
separação entre blocos (`BlockBoundaryResolver`). O laço principal
(`stateToMarkdown.ts:113-167`) decide, para cada par (anterior, atual), se usa linha em branco,
adjacência ou "rewrite-required" — e neste último caso um bloco de código indentado é reescrito
como fence **apenas na saída**, sem mutar o estado (`stateToMarkdown.ts:170-180`, a decisão na
linha 174).

Ponto de projeto: **o serializador é autorizado a normalizar sintaxe** (fence vs indentado,
marcador de lista, separação). Isso é o oposto de um AST byte-preserving — e é a razão pela qual
"abrir e salvar" pode mudar o arquivo.

## 4. Estado e transações

### 4.1 `JSONState` — a fonte da verdade em memória

`ref/marktext/packages/muya/src/state/index.ts:20`:

- Guarda `_state: TState[]` e um `_revision`.
- `preview(op)` aplica uma operação json1 sobre um **clone** e devolve um candidato imutável
  (`:50-52`). É o que permite renderizar a edição antes do commit.
- `apply(op)` adota o preview (`:55-58`).
- `setContent` / `markdownToState` (`:62-79`).
- `buildReplaceOp` constrói uma operação invertível de substituição comparando por
  `JSON.stringify` bloco a bloco (`:87-127`) — usado quando a árvore inteira precisa ser trocada.
- `getState()` devolve **clone profundo** (`:135-136`); `rawState` é a leitura interna não copiada
  (`:131-133`).
- `getMarkdown()` serializa sob demanda (`:139-148`).

Statics delegando a `ot-json1`: `invert`, `compose`, `transform` (`:21-36`). **Não há colaboração
implementada** — não há transporte, presença ou persistência de revisões. O README da engine é
explícito: "A collaborative host must supply transport, operation transformation and
synchronization" (`packages/muya/README.md:204`).

### 4.2 `EditTransactions` — o único caminho de escrita

`ref/marktext/packages/muya/src/editor/transactions/index.ts:38`.

| Peça | Linha | Papel |
|---|---|---|
| `enqueue(op, source)` | `:112-127` | Compõe na operação pendente; agenda `requestAnimationFrame` quando não há escopo |
| `insertOperation` / `removeOperation` / `editOperation` / `replaceOperation` | `:128-146` | Açúcar: `editOperation` é `json1.editOp(path, 'text-unicode', diff)` |
| `run(cmd, isolated)` | `:75-107` | Escopo síncrono aninhável; exceção → `cancel()` + `_rollback()` + rethrow |
| `apply(op, options)` | `:149-165` | Dentro de escopo: preview + `enqueue` + aplica parcial; fora: `flush` e commit |
| `flush()` | `:167-190` | Completa a transação, roda regras de edição, publica |
| `_complete()` | `:206-231` | `EDITING_RULES` (`:31`) compõem correções; devolve `view`, `history`, `blockOperation` |
| `assertStandalone()` | `:201-204` | Histórico e troca de documento são proibidos dentro de escopo |

O `run()` é o que sustenta `muya.transact()` (`src/muya.ts:295-297`). Rollback: qualquer exceção
descarta o pendente e chama o callback de reconstrução que o `Editor` injeta nas transações —
`() => runDocumentUpdate(() => scrollPage.updateState(jsonState.getState()))`, passado em
`src/editor/index.ts:299` e acionado em `src/editor/transactions/index.ts:102,182` (o método
`_rollback` vive em `EditTransactions`, `:53`, não no `Editor`).

### 4.3 Commit

`Editor._commitTransaction` (`src/editor/index.ts:572-616`) faz, em ordem:

1. `jsonState.apply(operation)`
2. `_updateBlocks(blockOperation, view, selection, recover)`
3. `eventCenter.emit('json-change', createChangePayload(operation, source, before, after, …))`

`_updateBlocks` (`:617-638`) tem dois modos:
- **incremental** (padrão): walks `pick`/`drop` sobre a árvore viva, dirigidos pela operação json1;
  qualquer falha cai para rebuild (`:631-637`).
- **rebuild**: `scrollPage.updateState(estado)` recria o DOM do zero (`:619-623`), usado por
  undo/redo de operações que o walker não consegue aplicar.

Isso é a decisão arquitetural mais interessante do projeto: **a operação é a unidade de
reconciliação**, e existe um caminho de recuperação integral quando ela não é reconciliável.

---

## 5. Renderer

### 5.1 Bloco → DOM

Cada bloco instancia seu próprio `tagName`/`classList`/`attributes` e cria o DOM no construtor via
`createDomNode()` (`src/block/base/treeNode.ts:100-120`). Um `github-alert` é um
`blockquote.mu-block-quote.mu-github-alert` com `data-alert-type` e um título
`contenteditable="false"` (`src/block/extra/githubAlert/index.ts:6-38`).

Não há reconciliação genérica: mutações usam `append`, `insertBefore`, `insertAfter`, `remove`,
`replaceWith` da lista ligada (`src/block/base/parent.ts`), sempre com `ChangeSource`.

### 5.2 Inline → HTML por bloco

`InlineEngine.patch()` (`src/inline/index.ts:70-86`):

```ts
const tokens = this._tokenizer(block, highlights);
const renderCursor = !this.muya.isReadOnly && cursor?.block === block ? cursor : {};
const html = this.renderer.output(tokens, block, renderCursor);
domNode!.innerHTML = html;
```

Ou seja: **o bloco inteiro é re-renderizado**, por `innerHTML`, a cada atualização. O renderer é
uma tabela de funções por tipo de token (`src/inline/renderer/index.ts:43-100`) produzindo VNodes
de `snabbdom`, serializados para string. Isso explica tanto as classes `mu-*` no DOM quanto o
comportamento de seleção (o cursor é restaurado por offsets depois da troca de HTML).

`checkNeedRender()` evita re-render quando nada mudou de token (chamado em
`src/block/base/format.ts:695-699`), mas qualquer mudança de token reescreve o bloco.

### 5.3 Tipografia

`applyAppearance` escreve variáveis CSS no nó raiz (`src/muya.ts:1602-1616`): `--mu-font-size`,
`--mu-line-height`, `--mu-font-family`, `--mu-code-font-size`, `--mu-code-font-family`, e a classe
`mu-code-wrap`. O contêiner original é **substituído** por um novo `div`
(`src/muya.ts:1618-1646`), copiando os atributos do elemento hospedeiro.

---

## 6. Seleção, teclado, undo, clipboard, IME

### 6.1 Seleção

`Selection` (`src/selection/index.ts:36`) mantém três especializações: `TextSelection`,
`ImageSelection`, `TableRectSelection`. A união serializável é `TSelectionState`
(`src/selection/types.ts:61-88`):

- texto: `{ path, offset }` em **unidades UTF-16**;
- imagem: `{ path, range: [start, end), raw }`;
- tabela: `{ tablePath, anchor, focus }` com coordenadas **zero-based incluindo o cabeçalho**.

A seleção viva é sincronizada por `selectionchange`/`mouseup` (`selection/index.ts:55-56`) e só é
publicada no commit, evitando tempestade de eventos durante digitação.

### 6.2 Teclado

Eventos DOM são mesclados com RxJS e roteados para o bloco ativo
(`src/editor/index.ts:353-445`): `click`, `input`, `keydown`, `keyup`, `compositionend`. O
`beforeinput` é interceptado primeiro para capturar a seleção **antes** de o navegador mover o
caret (`editor/index.ts:353-366`) — detalhe que evita a maior fonte de bugs em contenteditable.

`Format.inputHandler` (`src/block/base/format.ts:635-700`) lê `textContent`, aplica autopareamento,
compara tokens, grava `this.text` (que enfileira a operação) e restaura o caret.

### 6.3 Undo/redo

`History` (`src/history/index.ts:80`) guarda **operações json1**, não snapshots, com `maxStack: 100`
e `delay: 1000` (`:52-53`). Agrupamento é quebrado por fronteira de palavra e troca de
inserção/remoção (`shouldBreakUndoGroup`, `:66-77`) e por comandos explícitos
(`markInputBoundary`, `:191`). `recordRebuild` marca operações que exigem reconstrução total
(`:237`). O histórico é serializável (`ISerializedHistory`, `:26-38`) e o desktop o persiste por
aba separadamente do conteúdo (`editor.vue:1920-1922`).

### 6.4 Clipboard

`Clipboard` (`src/clipboard/index.ts:49`) trata copy/cut/paste com formatos `text/html` e
`text/plain`, além de `copyAsMarkdown`, `copyAsHtml`, `copyAsRich` (`:195-208`). O paste usa
`MarkdownToState` para Markdown e `HtmlToMarkdown` (turndown + plugin GFM) para HTML
(`src/clipboard/paste.ts:1-25`). Existe um conjunto explícito de `inputType`s "estrangeiros" que
não devem ser tratados como edição cruzada (`clipboard/index.ts:24-40`).

### 6.5 IME

IME tem tratamento dedicado porque o DOM e o modelo divergem durante composição:

- `isComposed` bloqueia o `inputHandler` durante a composição
  (`src/block/base/format.ts:637-645`).
- `compositionend` chama `inputHandler` manualmente, porque o navegador não emite `input`
  (`src/block/base/content.ts:693-707`).
- Existe uma **âncora de largura zero** inserida no fim de linha para o Chromium não engolir a
  primeira tecla depois de um `\n` (fix #3468), removida no `compositionend`
  (`content.ts:711-780`).

Testes: `e2e/tests/typing/ime.spec.ts` e `ime-soft-line-break.spec.ts`. O próprio teste documenta
a limitação do Playwright e simula `compositionstart`/`input`/`compositionend` manualmente
(`ime.spec.ts:5-27`) — ou seja, **IME real de SO não é coberto por automação**.

---

## 7. Temas e CSS

- A engine importa **6 folhas** no próprio entry (`src/muya.ts:43-48`): `blockSyntax`,
  `index`, `inlineSyntax`, `githubAlerts`, `dragDropText`, `prismjs/light.theme` — **2.555 linhas**.
  O diretório tem **7** arquivos (2.934 linhas): o sétimo, `exportStyle.css` (379 linhas), **não** é
  importado pelo entry, e sim inlinado no HTML de export (`src/state/markdownToHtml.ts:6`).
- Inclui **16 `@font-face`** em `src/assets/styles/index.css:68-203`: 8 faces de Open Sans (4 pesos
  × normal/itálico), 6 de DejaVu Sans Mono, Noto Music e Noto Sans Math. (`exportStyle.css` tem
  mais 2, não importados pelo entry.)
- Tema = **contrato de variáveis CSS**. A engine define kebab-case (`--mu-*`, `--editor-color-*`,
  `--theme-color`, `--link-color`; `src/assets/styles/index.css:1-66`); os **temas do desktop**
  ainda usam camelCase (`--editorColor`, `--themeColor`, `--linkColor`), e o comentário no CSS
  registra que a migração está pendente (`src/assets/styles/index.css:44-47`).
- O app desktop tem **32 arquivos de tema** em
  `packages/desktop/src/renderer/src/assets/themes/*.theme.css` (contagem observada), cada um só
  redefinindo variáveis (`dark.theme.css:1-60`).

Para nós: MarkText/Muya **não é neutro em estilo**. Estrutura HTML, nomes de classe e folhas de
estilo fazem parte do contrato público; um consumidor precisa aceitar `mu-*` classes e, na prática,
as variáveis CSS.

---

## 8. Persistência e exportação

### 8.1 Arquivo

O app nunca escreve JSON de estado em disco: o formato persistido é **Markdown**. No renderer, o
pipeline é `json-change` → `getMarkdown()` → store (`editor.vue:1907-1926`). No processo principal,
`writeMarkdownFile` converte line endings, codifica com `iconv` e **rejeita salvar** quando a
codificação não representa algum caractere (`EncodingLossError`,
`packages/desktop/src/main/filesystem/markdown.ts:134-163`). A escrita é atômica
(`writeFileAtomic`, `packages/desktop/src/main/filesystem/index.ts:126`).

Isso resolve, no nível de app, um problema que o formato não resolve: BOM, CRLF/LF, encoding
legado, newline final.

### 8.2 Exportação

Duas APIs públicas, ambas fora do editor vivo:

- `renderToStaticHTML(md, opts)` — **síncrona**, sem diagramas, sem wrapper, sanitizada por padrão
  (`src/state/renderToStaticHTML.ts:42-75`).
- `MarkdownToHtml` — **assíncrona**, gera documento completo com KaTeX/Prism/Mermaid/Vega/PlantUML
  renderizados e CSS embutido (`src/state/markdownToHtml.ts:1-70`).

O desktop usa um `Printer` próprio (`editor.vue:1689`), e a suíte de conformidade mede o caminho
`renderToStaticHTML(..., { sanitize: false })`
(`packages/muya/test/spec/conformance.md:11-13`).

---

## 9. Extensibilidade

### 9.1 Adicionar um bloco (fork obrigatório)

Não existe API de schema. A sequência observada é:

1. declarar a interface em `src/state/types.ts` e incluí-la em `TLeafState`/`TContainerState`;
2. emitir o estado no parser (`src/state/markdownToState.ts`, `_handleLeafToken`/`_handleContainerToken`);
3. serializar em `src/state/stateToMarkdown.ts` (`_serializeSimpleBlock`);
4. criar a classe em `src/block/{commonMark,gfm,extra}/…` com `static blockName`, `create()` e `getState()`;
5. registrar em `src/block/index.ts`;
6. se houver render inline novo, adicionar token no model e função no renderer.

Nenhum desses passos é opcional; `loadBlock` avisa e devolve `undefined` para nome não registrado
(`src/block/scrollPage/index.ts:33-40`).

### 9.2 Plugins de UI (extensão de verdade)

`Muya.use(Plugin, options)` é global na classe, com deduplicação por `pluginName`
(`src/muya.ts:142-152`). São **17 classes de plugin** em 22 diretórios de `src/ui/` (os demais são
`baseFloat`/`baseScrollFloat`, `utils` e `__tests__`), cada uma com `static pluginName` e exportada
por `src/index.ts` (ex.: `formatPicker` não existe; os reais são `inlineFormatToolbar`, `linkTools`,
`paragraphQuickInsertMenu`, `tableChessboard`…). Todos são flutuantes, posicionados
por `@floating-ui/dom`. Instanciados em `init()` (`src/muya.ts:246-253`).

Isso estende **a interface**, não o documento. Um host pode desligar atalhos
(`keyboardShortcuts: false`) e injetar seus próprios (`editor.vue:1695-1714`).

### 9.3 Opções como eixo de comportamento

`IMuyaOptions` (`src/types.ts:9-126`) tem **57 chaves** e inclui os hooks de host que substituem
integração com o SO: `clipboardFilePath`, `clipboardText`, `clipboardWriteText`, `imageAction`,
`getPathForFile`, `fetchLinkHtml`, `documentId` (`types.ts:75-121`). Defaults em
`MUYA_DEFAULT_OPTIONS` (`src/config/index.ts:317`). Oito opções mudam a **estrutura** e forçam
reparse (`PARSE_AFFECTING_OPTIONS`, `src/muya.ts:121-130`).

---

## 10. Acessibilidade

O que existe:

- Raiz com `tabindex="0"`, `role="textbox"` (edição) ou `document` (leitura) e `aria-label`
  traduzido (`src/muya.ts:218-222`).
- `aria-expanded`/`aria-label` no toggle de dobra de heading
  (`src/block/commonMark/headingFoldToggle/index.ts:31-95`).
- `aria-label` em botão de copiar código e feedback de cópia
  (`src/block/commonMark/codeBlock/code.ts:39`, `src/utils/copyFeedback.ts:10-19`).
- Modo leitura promove links a `role="link"` + `tabindex="0"`
  (`src/editor/readOnly.ts:5-15`) e navegação por Enter.
- Diagramas recebem nome acessível via `accessibleName`/`applyFinalizations`
  (`src/utils/diagram/render.ts:20-56`).

O que não existe / é frágil:

- Contagem total de ocorrências de `aria-`/`role=` em `src/**/*.ts` (excluindo testes): **19**
  (comando `grep -rn` observado, 18 linhas `aria-*` + 1 `role=`). Para 39 blocos e 17 plugins, é
  cobertura marginal.
- A suíte axe filtra por `impact === 'critical'` (`e2e/tests/a11y/host-scan.spec.ts:19`) e apenas
  loga os demais (`logNonCritical`, `:21-27`) — passa com violações `serious`.
- O conteúdo é um único `contenteditable` com `role="textbox"`; leitor de tela enxerga um campo
  editável gigante, não uma árvore de documento navegável.
- Não há gerenciamento de foco entre blocos anunciado, nem live regions.

---

## 11. Performance

Fatos observáveis:

- **Sem virtualização.** `setContent` chama `scrollPage.updateState(state)`, que faz `empty()` e
  recria **todos** os blocos (`src/block/scrollPage/index.ts:87-96`).
- **Re-render por bloco, por edição**: `innerHTML` completo do bloco
  (`src/inline/index.ts:70-86`), mais re-tokenização inline com cache apenas de *referências*.
- O cache de definições é por **identidade do snapshot** de estado
  (`src/inline/referenceDefinitions.ts:41-71`) — barato e correto, mas só cobre labels.
- Existe um único teste de performance: 10.000 parágrafos devem carregar em **< 60 s** no Vite dev
  (`e2e/tests/stability/perf.spec.ts:17-42`, asserção `< 60_000` na linha 35, timeout de 120 s na
  linha 13), com verificação de contagem de DOM e scroll até o fim. É um *smoke*, não um benchmark.
- Há um teste explícito de vazamento de listeners (`e2e/tests/stability/listener-leak.spec.ts`) e
  `destroy()` desanexa eventos (`src/muya.ts`), o que indica preocupação real com ciclo de vida.
- Histórico limita-se a 100 operações (`src/history/index.ts:52-53`) — memória previsível, undo
  curto em sessões longas.

Leitura para o nosso caso: o custo cresce com o tamanho do documento porque (a) toda a árvore é
DOM real e (b) cada tecla reescreve um bloco. Para documentos de 10 mil parágrafos isso é aceitável
apenas porque o orçamento é frouxo. **Nenhuma medição de digitação em documento grande foi
encontrada** — a lacuna é real.

---

## 12. Links, referências e arquivos

- **Links inline** são tokens (`link`, `referenceLink`, `autoLink`) com `range` UTF-16 e `raw`
  preservado; o DOM guarda `data-raw`, `data-start`, `data-end` para o popover
  (`src/utils/getLinkInfo.ts:5-53`).
- **Referências** `[ref]: url "title"` são escaneadas de parágrafos do estado, com precedência da
  primeira definição e normalização de label (`src/inline/referenceDefinitions.ts:6-39`).
- **Arquivos**: o caminho de imagem é resolvido pelo host via `imageAction`, `getPathForFile`,
  `beforeImageDrop` (`src/types.ts:100-130`). O app mapeia `getPathForFile` para
  `webUtils.getPathForFile` (`editor.vue:1780-1782`).
- **Links internos** (`#anchor`) são resolvidos **no app**, não na engine: `FORMAT_LINK_CLICK`
  rola para o `githubSlug` do TOC ou para um elemento com `id`
  (`packages/desktop/src/renderer/src/store/editor.ts:425-449`). Links de arquivo vão por IPC
  (`mt::format-link-click`).
- **Não há resolução de referência entre documentos.** Referência é só rótulo → URL dentro do
  mesmo documento. Para "referências entre publicações" teríamos de desenhar isso do zero.

---

## 13. Erros e diagnóstico

- `logger` com namespaces por módulo (`src/utils/logger.ts`, usado como `logger('editor:')`).
- Falha de aplicação incremental não derruba o documento: cai para rebuild e loga
  (`src/editor/index.ts:631-637`).
- `_rollback` reconstrói a árvore a partir do estado commitado em exceção de transação
  (`src/editor/index.ts:291-300`).
- Marcação de falha de expectativa do parser é **versionada em arquivo**, não em tolerância:
  `test/spec/expected-failures.json` falha se um exemplo listado começar a passar
  (`packages/muya/AGENTS.md:22`).
- Não há relatório de erro tipado para o host: erros de transação são exceções JS.

---

## 14. Resumo dos tradeoffs

| Decisão | Ganho | Custo |
|---|---|---|
| Texto Markdown cru dentro de cada bloco | Round-trip fiel da sintaxe | Metadados duplicam a informação; edição precisa reparsear |
| `TState[]` discriminado por `name` | Simples, serializável, tipado | Sem schema, sem validação, extensão = fork |
| Operações `ot-json1` como unidade de reconciliação | Undo/transação/rollback coerentes; pronto para OT | Complexidade alta (`pick`/`drop`, rebuild de fallback) |
| `contenteditable` + `innerHTML` por bloco | Funciona em qualquer navegador; DOM simples | IME/caret frágeis, re-render caro, a11y limitada |
| Plugins só de UI | Host customiza sem tocar no core | Documento permanece fechado |
| Tema por variáveis CSS + 6 folhas próprias (2.555 linhas) | Belo out-of-the-box | Não é neutro; acopla consumidor a classes e vars |
| Markdown como único formato em disco | Portável, diffável, interoperável | Perde histórico de estrutura; mudanças de serialização alteram arquivos |
