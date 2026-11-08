# Arquitetura do Lexical

Tudo abaixo se refere a `ref/lexical/` (MIT, commit `5326e06`, versão `0.53.0`).
Símbolos citados com `arquivo:linha`.

---

## 1. Mapa de pacotes

O monorepo tem 40 pacotes (`ls ref/lexical/packages | wc -l`). O grafo relevante para nós:

```
lexical                      <- núcleo: modelo, update, reconciler, comandos, extensão-core
  @lexical/internal          <- invariants/version; única dependência de runtime do core
  @lexical/extension         <- LexicalBuilder + extensões de plataforma (IME, HMR, NodeStructure)
  @lexical/rich-text         <- MODO rico (registerRichText, HeadingNode, QuoteNode)
  @lexical/plain-text        <- MODO texto puro; conflita com rich-text
  @lexical/history           <- undo/redo
  @lexical/clipboard         <- DnD + paste; depende de html
  @lexical/html              <- export/import DOM
  @lexical/markdown          <- import/export/shortcuts markdown
  @lexical/selection         <- helpers de RangeSelection ($wrapNodes, $patchStyleText, …)
  @lexical/utils             <- $dfs, mergeRegister, positioning, etc.
  @lexical/yjs               <- binding Yjs (V1 + V2__EXPERIMENTAL)
  @lexical/a11y              <- aria-live, announce de undo, focus trap, roving tabindex
  @lexical/react             <- bindings React (Composer, ContentEditable, ~50 plugins)
  @lexical/headless          <- createHeadlessEditor sem DOM
  @lexical/list|link|table|code|mark|hashtag|overflow|dragon|file|text|offset|...
```

Regra de dependência: `lexical` **não** importa nenhum `@lexical/*` de UI. Suas dependências
declaradas são só `@lexical/internal` (`ref/lexical/packages/lexical/package.json`). Isso é o que
torna o modelo independente de React e de DOM-de-aplicação.

`@lexical/rich-text` importa `@lexical/a11y`, `@lexical/clipboard`, `@lexical/dragon`,
`@lexical/extension`, `@lexical/html`, `@lexical/selection`, `@lexical/utils` e `lexical` — ou seja,
"modo rico" é uma composição de 8 pacotes, não uma flag.

## 2. Árvore de nós

### 2.1 Campos e ligação

`LexicalNode` carrega só ponteiros e identidade
(`ref/lexical/packages/lexical/src/LexicalNode.ts:1072-1085`):

```ts
readonly __type: string;      // identidade de classe/serialização
__key: string;                // identidade de instância
__parent: null | NodeKey;
__prev: null | NodeKey;
__next: null | NodeKey;
__state?: NodeState<this>;    // NodeState declarativo (novo)
```

Filhos **não** ficam num array no nó: `ElementNode` navega por `__prev`/`__next` e mantém
`getChildren()` derivado. `LexicalNode` expõe `getParent/getPreviousSibling/getNextSibling/
getIndexWithinParent` (`LexicalNode.ts:1386-1520`).

A árvore é uma **lista duplamente ligada por pai**, não uma árvore de arrays. Vantagem: mover/splitar
custa O(1) por ponteiro. Desvantagem: toda leitura de filhos é uma caminhada.

### 2.2 Hierarquia de classes

```
LexicalNode
├── TextNode           (inline, format bitmask, mode normal|token|segmented)
├── LineBreakNode      (<br>)
├── TabNode
├── ElementNode        (contêiner; children + indent + format + direction)
│   ├── RootNode       (tipo 'root', único por editor)
│   ├── ParagraphNode  ($config children:'inline')
│   ├── HeadingNode / QuoteNode / ListNode / TableNode / … (pacotes)
├── DecoratorNode<T>   (nó folha cujo DOM é renderizado por fora — React, outro editor)
└── ArtificialNode__DO_NOT_USE
```

`ParagraphNode.$config()` declara a estrutura:
`children: 'inline'` e a regra de import de `<p>`
(`packages/lexical/src/nodes/LexicalParagraphNode.ts:56-68`).

`NodeChildrenConfig` é `'inline' | 'block' | readonly ('inline' | Klass)[]`
(`packages/lexical/src/LexicalNode.ts:137-140`). Esse é o único "schema estrutural" pronto do
core; é **experimental** (`@experimental node-structure`, `LexicalStructure.ts:52-77`).

### 2.3 Texto como *runs*, não como string

Negrito não é atributo do parágrafo: é `format: number` de cada `TextNode`. O bitmask está em
`packages/lexical/src/LexicalConstants.ts:38-56`:

```
IS_BOLD=1, IS_ITALIC=2, IS_STRIKETHROUGH=4, IS_UNDERLINE=8, IS_CODE=16,
IS_SUBSCRIPT=32, IS_SUPERSCRIPT=64, IS_HIGHLIGHT=128,
IS_LOWERCASE=256, IS_UPPERCASE=512, IS_CAPITALIZE=1024
```

Consequência observada no probe: formatar "hi bold link" em negrito **quebra** os nós de texto nas
fronteiras de formato:

```
5) after formatText("bold"): 4 TextNodes -> "hi"(1) " "(0) "bold"(1) " link"(0)
```

Isso torna o diff trivial (compare `format`) e a serialização compacta (`"format": 1`).

`TextNode` tem ainda `__detail` (directionless/unmergeable),
`__mode` (`normal` | `token` | `segmented` — `LexicalConstants.ts:29-33`) e `__style` (CSS inline
por nó). `token`/`segmented` são o que permite "menções"/entidades não editáveis por caractere.

## 3. EditorState e o ciclo de transação

### 3.1 EditorState

`EditorState` = `{_nodeMap, _selection, _readOnly, _parsed, _movedNodes, _splitNodes, _textEdits,
_slotsUsed}` (`packages/lexical/src/LexicalEditorState.ts:193-261`).

Pontos que importam para nós:

- **Imutável por convenção**: o `Map` é substituído por nó em `getWritable`, e o estado só vira
  `_readOnly = true` ao ser commitado/clonado (`:279-294`).
- `_movedNodes`/`_splitNodes`/`_textEdits`: metadados *internos* que só o binding Yjs usa para
  distinguir mover de deslocar e para aplicar edições de texto com offsets exatos (`:203-233`).
  É a solução do Lexical para "diff de árvore não sabe dizer onde você digitou".
- `toJSON(compact)` tem **duas formas**: a legada (todo campo) e a compacta, que omite o que o
  parse restaura do schema (`:295-330`). `JSON.stringify(editorState)` sempre usa a legada,
  porque `toJSON` é chamado com o nome da propriedade, não com `true` (`:316-322`).

### 3.2 `editor.update(fn)` passo a passo

`updateEditor` só enfileira se já estiver num update; caso contrário chama `$beginUpdate`
(`packages/lexical/src/LexicalUpdates.ts:1561-1571`).

`$beginUpdate` (`:1308-1400`):

1. aplica `options.tag` ao conjunto de tags do update;
2. **clona** o `EditorState` pendente ou atual e o torna o estado ativo (`:1332-1340`);
3. captura a seleção do DOM (`$internalCreateSelection`) se o editor tem raiz (`:1364-1375`);
4. roda `fn()`;
5. `$processNestedUpdates` (updates disparados dentro do update são enfileirados, não recursados);
6. `applySelectionTransforms`;
7. se algo ficou dirty: `$applyAllTransforms` (ou só normalização de texto quando
   `skipTransforms`);
8. `$garbageCollectDetachedNodes` (remove do `NodeMap` o que desanexou);
9. valida contenção de `RangeSelection` em relação a *slots*.

O commit é **assíncrono por padrão** (microtask); `{discrete: true}` força síncrono. Verificado em
teste do próprio repo: `editor.getEditorState().read(...)` ainda devolve `''` logo após um
`editor.update` não-discreto, e `editor.read('pending', ...)` devolve `'foo'`
(`packages/lexical/src/__tests__/unit/LexicalEditorState.test.ts:45-60`).

### 3.3 Transforms

`$applyAllTransforms` (`LexicalUpdates.ts:333-428`) implementa um **fixpoint** com uma heurística
explícita, documentada no próprio código:

> 1. transformamos folhas primeiro; se gerarem dirty, repetimos;
> 2. depois transformamos elementos; se gerarem folhas dirty, voltamos ao passo 1.

`$applyTransforms` (`:251-275`) roda a lista de transforms registrada para o `__type` do nó e
**aborta** a lista se o nó deixou de estar anexado.

Há duas formas de registrar:
- clássica: `editor.registerNodeTransform(Klass, fn)` (`LexicalEditor.ts:1627`);
- declarativa: `static transform()` na classe do nó (`LexicalNode.ts:1907`), recolhida por
  `getTransformSetFromKlass` (`LexicalEditor.ts:933`) e registrada em `createEditor`
  (`:1102`).

Medido no probe (nó `LoudTextNode` com `static transform()` que uppercasa o texto):

```
execuções do transform: 2
texto final: "HELLO TRANSFORM"
JSON do nó: {"detail":0,"format":0,"mode":"normal","style":"","text":"HELLO TRANSFORM","type":"loud-text","version":1}
```

Duas execuções porque o próprio transform marca o nó dirty; o fixpoint converge porque a segunda
passada não muda mais nada.

### 3.4 Seleção dentro do estado

`_selection` é parte do `EditorState` (`BaseSelection | null`). Ou seja: **seleção é dado**, não
estado global do DOM. `editorStateHasDirtySelection` compara seleção pendente com a atual
(`LexicalEditorState.ts:66-84`). Isso é o que permite colaboração com cursores remotos sem tocar
no DOM local.

## 4. Reconciliação de DOM

`$reconcileNode` (`packages/lexical/src/LexicalReconciler.ts:1793`) é o coração:

1. se `prevNode === nextNode` e não está dirty: reusa o DOM e o texto em cache
   (`__lexicalTextContent`) — caminho rápido, sem tocar no DOM (`:1816-1844`);
2. se mudou e está dirty: registra mutação (`'updated'`) para os listeners;
3. chama `updateDOM(prev, dom, config)`; **se retornar `true`**, o DOM é recriado via
   `$createNode` e o antigo destruído (`:1863-1876`);
4. se for ElementNode, ajusta `indent` e `format` comparando com o anterior.

`$createNode` (`:707`) faz `createDOM`, associa o DOM à chave (`storeDOMWithKey`,
`LexicalReconciler.ts:2294`, que delega a `setNodeKeyOnDOMNode` e grava a propriedade
`__lexicalKey_<editorKey>`, não atributo — `LexicalUtils.ts:689-707`) e marca
`data-lexical-text="true"` para nós de texto (`:771-773`).

Reconciliação de filhos é por **chave**, não por índice: `$reconcileChildren` (`:1438`) casa
`next` com `prev` por `NodeKey` e move DOM existente. Isso permite reuso de DOM em movimento entre
pais — inclusive para decorators React, evitando remount (`:714-730`).

`MutationObserver` é inicializado em `setRootElement` (`LexicalEditor.ts:1755`) e
**desconectado antes de cada reconcile** (`LexicalUpdates.ts:686`) para não reagir às próprias
mutações; mutações não reconhecidas pelo modelo são tratadas em `LexicalMutations.ts`
(`flushRootMutations`, `initMutationObserver`).

### 4.1 Custo e limites

- Cada update percorre apenas `_dirtyElements`/`_dirtyLeaves`, não a árvore toda. Um
  `FULL_RECONCILE` só em `setRootElement`/`setEditorState` estrutural (`LexicalConstants.ts:24-27`).
- Texto é acumulado por subárvore em `__lexicalTextContent` no DOM, para `getTextContent` O(1) em
  subárvores limpas.
- Não há virtual DOM nem batching de estilo: `updateDOM` roda síncrono dentro do commit.

## 5. Seleção, teclado, IME, clipboard

### 5.1 Duas seleções

- **Modelo**: `RangeSelection` = `{anchor: Point, focus: Point, format, style}`
  (`LexicalSelection.ts:838-864`) ou `NodeSelection` = `Set<NodeKey>`
  (`:614-628`). `Point` = `{key, offset, type: 'element'|'text'}` (`:177`).
- **DOM**: `window.getSelection()`, lida em `$internalCreateSelection`
  (`LexicalSelection.ts:3568-3585`) e escrita em `$updateDOMSelection` (`:4019`).

A sincronização é nos dois sentidos e é a maior fonte de bugs históricos (a suíte tem
`LexicalSelectionResolveLeafPosition.test.ts`, `LexicalRootSelectionPhantomParagraph.test.ts`,
`MouseClickWithoutLayout.test.ts`, `NormalizeTripleClickSelectionExtension`…).

### 5.2 Evento → comando

O core registra cinco handlers (`LexicalEvents.ts:1153-1175`). Eles *traduzem*:

| Evento DOM | Comando |
|---|---|
| `keydown` Enter | `KEY_ENTER_COMMAND` (`LexicalEvents.ts:1995,2133`) |
| `keydown` Backspace | `KEY_BACKSPACE_COMMAND` (`:2175`) |
| `keydown` setas | `KEY_ARROW_*_COMMAND` (`:2163`) |
| `paste` | `PASTE_COMMAND` (`:1574`) |
| `copy`/`cut` | `COPY_COMMAND`/`CUT_COMMAND` (`:2240,2517-2526`) |
| `beforeinput` | `BEFORE_INPUT_COMMAND`, depois `INPUT_COMMAND` |
| `compositionstart/end` | `COMPOSITION_START/END_COMMAND` |

Quem **executa** são os pacotes de modo. Exemplo: `DELETE_CHARACTER_COMMAND` →
`selection.deleteCharacter(isBackward)` em `lexical-rich-text/src/index.ts:1330-1344`.

Prioridades: `COMMAND_PRIORITY_BEFORE_CRITICAL` … `CRITICAL`, `EDITOR`, `NORMAL`, `LOW`
(`LexicalEditor.ts:674-719`). Um listener que retorna `true` **consome** o comando; a cadeia para.
Foi assim que `FORMAT_TEXT_COMMAND` funcionou no probe sem UI.

O núcleo também traz um **keymap default** rígido: setas, Enter, Tab, `Delete`/`Backspace` de
palavra, Ctrl/Cmd+B/I/U e Ctrl/Cmd+Z/Shift+Z são convertidos em comandos antes de qualquer plugin
(`LexicalEvents.ts:2163-2240`). Isso não contradiz "o core não edita" — é tradução, não ação — mas
é comportamento que o host não escolheu e não consegue desligar sem reescrever o handler.

### 5.3 IME / composição

O core mantém `_compositionKey` (nó em composição) e não roda transforms nesse nó
(`LexicalUpdates.ts:276-285`: `node.__key !== compositionKey`). Há sufixos de composição
dependentes de plataforma em `LexicalConstants.ts:100-112`
(`COMPOSITION_SUFFIX`, `COMPOSITION_START_CHAR`), com `NON_BREAKING_SPACE` para iOS/Safari e
Firefox.

O comportamento é plataforma-específico e testado: `LexicalIosKoreanIME.test.ts` documenta que o
teclado 10-key do iOS **não** emite `compositionstart/end` e manda
`beforeinput deleteContentBackward` com `targetRange` não-colapsado
(`packages/lexical/src/__tests__/unit/LexicalIosKoreanIME.test.ts:9-23`). Existem ainda
`LexicalAndroidChromeComposition.test.ts`, `LexicalFirefoxCompositionEndTag.test.ts`,
`LexicalNormalizationComposition.test.ts` e `Composition.spec.mjs` (e2e).

`@lexical/dragon` existe só para resiliência de composição. E há `IMEExtension` em
`@lexical/extension` (`lexical-extension/src/IMEExtension.ts`, 144 linhas).

Conclusão para nós: **IME é o custo escondido de qualquer editor contenteditable**. Lexical o
enfrenta com código por plataforma, não com uma abstração.

### 5.4 Clipboard

`@lexical/clipboard` grava três MIME types no `DataTransfer`: HTML (`$getHtmlContent`, `:89`),
`application/x-lexical-editor` (`$getLexicalContent`, `:118`, JSON serializado) e texto puro. Na
colagem dá preferência ao formato Lexical quando o namespace bate, senão HTML, senão texto
(`$insertDataTransferForRichText`, `:173`).

Observado no probe sem `@lexical/link` instalado: colar
`<p>hi <strong>bold</strong> <a href="…">link</a></p>` produziu três `TextNode`s e **descartou o
link** — porque não há `LinkNode` registrado com regra `importDOM` para `<a>`. Isto é a prova de
que "suporte a uma feature" = "nó registrado + regra de import/export", não uma flag.

### 5.5 Undo/redo

`@lexical/history` guarda `undoStack`/`redoStack` de `{editorState, ...}`
(`lexical-history/src/index.ts:74`). Merge de edições dentro de uma janela `delay` (padrão 300 ms,
`:854`) para não criar um undo por caractere. `HISTORY_MERGE_TAG` marca updates que devem entrar no
mesmo passo.

Verificado no probe:

```
B: abc
after undo: a
after redo: abc
1) CAN_UNDO/CAN_REDO observed by a listener: {"undo":true,"redo":null}
```

`CAN_UNDO_COMMAND` é **notificação**: o history a dispara para quem escutar; chamá-la você mesmo
retorna `false` (nenhum listener tratou). É uma pegadinha real de API.

## 6. Extensões: o que é modelo e o que é plugin

### 6.1 `defineExtension`

```ts
defineExtension({
  name, namespace,
  nodes: () => [HeadingNode, QuoteNode],
  dependencies: [ ...outras extensões... ],
  conflictsWith: ['@lexical/plain-text'],
  config: safeCast<C>(defaults),
  mergeConfig, build, init, register,
  theme,
});
```

Campos e semântica observados em `RichTextExtension`
(`lexical-rich-text/src/LexicalRichTextExtension.ts:111-141`) e no núcleo
(`packages/lexical/src/extension-core`, reexportado em `src/index.ts:560-591`).

`buildEditorFromExtensions` (`lexical-extension/src/LexicalBuilder.ts:81`) cria um `LexicalBuilder`,
que:
- injeta `InitialStateExtension` como raiz (`:142`);
- resolve o grafo de dependências (`addEdge`, `:251`), mescla configs na ordem em que foram vistas
  e detecta conflitos por nome (`:300-321`);
- chama `createEditor` com `nodes` + `theme` + `onError/onWarn` e depois `build(editor)` de cada
  extensão em ordem topológica (`constructEditor`, `:177-208`);
- `register(editor)` roda em seguida e o retorno é composto num `dispose()`
  (`buildEditor`, `:210-231`) — daí `editor.dispose()` existir.

### 6.2 O que é o quê

| Camada | Mora em | Exemplo |
|---|---|---|
| Modelo puro (nós, update, reconciler, comandos) | `lexical` | `ParagraphNode`, `RangeSelection` |
| Modo de edição | `@lexical/rich-text`, `@lexical/plain-text` | `registerRichText` |
| Comportamento opcional | `@lexical/extension` | `IMEExtension`, `NodeStructureExtension`, `TabIndentationExtension` |
| Feature de documento | pacotes `@lexical/list`, `link`, `table`, `code`, `mark` | `ListNode` + comandos |
| Serialização | `@lexical/html`, `markdown`, `clipboard` | `$generateHtmlFromNodes` |
| Colaboração | `@lexical/yjs` | `createBinding` |
| A11y | `@lexical/a11y` | `AriaLiveRegionExtension` |
| UI | `@lexical/react` | `ToolbarPlugin` (no playground) |

Regra prática: **se muda o documento, é nó; se muda a interação, é extensão; se desenha pixels, é
host.**

### 6.3 Três sistemas de configuração de nó (dívida visível)

1. **Clássico** — `static getType()`, `static clone()`, `static importJSON`, `constructor(key)`.
   Ainda obrigatório em produção (`LexicalNode.ts:1100-1137`).
2. **`$config()`** — declarativo, com `extends`, `children`, `importDOM`, `generated` (JSON
   pré-gerado). `ParagraphNode.$config()` é o exemplo canônico
   (`LexicalParagraphNode.ts:56-68`). `getType()` do base agora **lê** desse config
   (`LexicalNode.ts:1100-1112`).
3. **Schema declarativo + `NodeState`** — `nodeSchema`/`stringValue`/`withField`/`createState`
   (`LexicalSchema.ts` 3.386 linhas; `LexicalNodeState.ts` 1.374 linhas; exports em
   `src/index.ts:243-314`). Permite serialização tipada com defaults, `accepts`, e migração por
   versão.

Os três coexistem e se sobrepõem. Para o `rich-editor-auy`, escolher **um** (o 3º é o mais próximo
do que queremos) e não replicar a dívida.

## 7. Schema e restrições de conteúdo

- **Estrutural**: `NodeChildrenConfig` (`'inline'`/`'block'`/lista) validado em
  `validateNodeStructure` (`LexicalStructure.ts:1892`) chamado no `createEditor` em DEV
  (`LexicalEditor.ts:1132-1134`). Normalização em `INTERNAL_$normalizeNodeStructure`
  (`LexicalStructure.ts:1657`) e enforcement por `NodeStructureExtension`
  (`lexical-extension/src/NodeStructureExtension.ts`, 92 linhas).
- **Por nó**: `isInline()`, `canInsertTextBefore/After` (`TextNode.ts:1287-1308`),
  `isParentRequired`/`createParentElementNode` (`LexicalNode.ts:2063-2073`).
- **Serialização**: schema por campo, com `accepts` para tolerar JSON antigo
  (`LexicalSchema.ts:276-290`).
- **Mutações ilegais falham alto**: há invariants em vez de coerção silenciosa (ex.:
  `ElementNode` em slot, `cleanNodeStructure`, `$getTextNodeOffset: invalid offset 5 for size 4`
  observado no probe).

## 8. Serialização

### 8.1 JSON (canônico)

`editorState.toJSON()` → `{root: {...}}`, recursivo por `exportJSON`/`$exportNodeToJSON`
(`LexicalEditorState.ts:94-133`). Cada nó carrega `type`, `version`, e os campos próprios. Slots
vão em `$slots` (`:113-129`).

Import: `editor.parseEditorState(jsonOuString)` → `EditorState` (sem rodar transforms,
`_parsed = true`), aplicado por `editor.setEditorState(state)`.

### 8.2 HTML

- Export: `$generateHtmlFromNodes(editor, selection)` (`lexical-html/src/index.ts:255`). Usa
  `node.exportDOM()` (não `createDOM()`), e o pipeline respeita `$shouldInclude`/`$shouldExclude`
  do `DOMRenderConfig` (`:240-260`, `:77-96`).
- Import: `$generateNodesFromDOM(editor, dom)` (`:147`), usando `static importDOM()` de cada nó
  com `DOMConversionMap` e prioridade.

### 8.3 Markdown

`@lexical/markdown` expõe `$convertFromMarkdownString`, `$convertToMarkdownString`,
`registerMarkdownShortcuts` e conjuntos de *transformers*
(`lexical-markdown/src/index.ts` exports; `MarkdownTransformers.ts:1373-1423`:
`ELEMENT_TRANSFORMERS`, `MULTILINE_ELEMENT_TRANSFORMERS`, `TEXT_FORMAT_TRANSFORMERS`,
`TEXT_MATCH_TRANSFORMERS`).

O modelo é **transformers, não parser**: cada regra sabe importar e exportar um construto
(`HEADING`, `QUOTE`, `CODE`, `UNORDERED_LIST`, `CHECK_LIST`, `ORDERED_LIST`, `BOLD_STAR`,
`ITALIC_STAR`, `LINK`, `HIGHLIGHT`, `INLINE_CODE`, `STRIKETHROUGH`,
`MarkdownTransformers.ts:806-1400`). Shortcuts de digitação (`## `, `- `, `**x**`) reaproveitam as
mesmas regras via `registerMarkdownShortcuts`.

### 8.4 Clipboard

Formato triplo (ver §5.4). O formato Lexical inclui namespace para evitar colar entre editores
incompatíveis.

## 9. Colaboração (Yjs)

- `createBinding(editor, provider, id, doc, docMap, excludedProperties?)`
  (`lexical-yjs/src/Bindings.ts:175-188`).
- Mapeamento: `CollabElementNode` (um `Y.XmlElement` por elemento), `CollabTextNode` (um
  `Y.XmlText` por run), `CollabDecoratorNode`, `CollabLineBreakNode`
  (`lexical-yjs/src/Collab*.ts`).
- Sincronização nos dois sentidos: `syncLexicalUpdateToYjs` e `syncYjsChangesToLexical`
  (`SyncEditorStates.ts:359` e `:195`), com variantes `V2__EXPERIMENTAL` (`:576`, `:489`).
- Cursores remotos em `SyncCursors.ts`.
- A ponte usa `COLLABORATION_TAG`/`SKIP_COLLAB_TAG` (`LexicalUpdateTags.ts`) para não gerar loop
  update → yjs → update.
- `_movedNodes` e `_textEdits` no `EditorState` existem **para isto**: sem eles, um diff de árvore
  não distingue "nó movido" de "irmãos deslocados", nem sabe onde um insert aconteceu quando há
  texto igual dos dois lados (`LexicalEditorState.ts:195-233`).

Existe também `@lexical/y` (7,7 MB), experimental, com "layouts" novos — o CHANGELOG avisa que
layouts antigos podem deixar de ser lidos (`ref/lexical/CHANGELOG.md`, topo).

## 10. Acessibilidade

Estado do core: **quase nada**. O `role`/`aria-label` do exemplo são escritos à mão
(`examples/vanilla-js/index.html:14-17`). O que existe:

| Capacidade | Onde | Opt-in? |
|---|---|---|
| `data-lexical-editor`, `data-lexical-text`, `data-lexical-decorator` | reconciler | sempre |
| região `aria-live` única + `announce()` | `AriaLiveRegionExtension`, `lexical-a11y/src/index.ts:506` | sim |
| anúncio de undo/redo ("Undone"/"Redone") | `HistoryAnnounceExtension`, `:590` | sim |
| anúncio editable/read-only | `EditorModeAnnounceExtension`, `:649` | sim |
| focus trap | `FocusTrapExtension`, `:696` | sim |
| roving tabindex | `RovingTabIndexExtension`, `:709` | sim |
| focus manager | `FocusManagerExtension`, `:722` | sim |
| anúncio de heading ao navegar | `HeadingAnnounceExtension`, `lexical-rich-text/src/HeadingAnnounceExtension.ts` | via rich-text |

`announce` insere um ZWSP quando a mensagem se repete, para o leitor de tela registrar mudança
(`lexical-a11y/src/index.ts:508-517`). Detalhe pequeno e correto.

Não encontrado no core: `aria-multiline` automático, `aria-placeholder` automático (o React exige
que você passe `aria-placeholder` junto do `placeholder`, por tipo —
`lexical-react/src/LexicalContentEditable.tsx:29-41`), nem gestão de `aria-expanded` de menus
(fica nos plugins).

## 11. Estilo e tema

- **Zero CSS nos pacotes de biblioteca.** `find node_modules/lexical node_modules/@lexical -name
  '*.css'` no probe não retorna nada. No monorepo há 48 `.css`, todos em três lugares que **não são
  biblioteca**: `lexical-playground` (app), `lexical-devtools` (extensão de navegador) e
  `lexical-website` (site de docs). Nenhum pacote publicado arrasta CSS.
- Tema é um objeto `EditorThemeClasses` passado em `createEditor({theme})`, com papéis:
  `root`, `paragraph`, `text.{base,bold,italic,underline,strikethrough,code,highlight,...}`,
  `indent`, `list`, `table`, `link`, `heading`, `quote`, `code`, `hashtag`, `collaboration`,
  `embedBlock`, `mark`, `markOverlap`, `tab`, `ltr`/`rtl`
  (`packages/lexical/src/LexicalEditor.ts:189-260`).
- Cada nó lê o tema no seu `createDOM`: `getCachedClassNameArray(config.theme, 'paragraph')`
  (`LexicalParagraphNode.ts:72-80`), `setTextThemeClassNames`
  (`LexicalTextNode.ts:194-265`).
- Caso especial tratado explicitamente: **underline + strikethrough** precisam de uma classe
  combinada (`underlineStrikethrough`) porque ambas usam `text-decoration`
  (`LexicalTextNode.ts:207-232`; comentário: "there's no easy workaround for many atomic CSS
  systems today"). É uma cicatriz real de CSS atômico.
- Estilos **inline** que o core impõe: no root (`user-select`, `white-space`, `word-break`,
  `LexicalEditor.ts:1749-1751`); em `TextNode.__style` (CSS por nó, `setDOMStyleFromCSS`); no
  export HTML (`white-space: pre-wrap` em cada texto, `LexicalTextNode.ts:998`).
- `indent` é aplicado como atributo/estilo (`data-lexical-indent` lido em
  `setNodeIndentFromDOM`, `LexicalUtils.ts:3367-3380`; escrito no `exportDOM` do elemento) — não
  como classe.

## 12. Performance

Instrumentação versionada em `packages/lexical/src/__bench__/` com dois projetos vitest:
`bench` (node, estruturas) e `bench-dom` (jsdom, ciclo real). Arquivos:
`nodeMap.bench.ts`, `getWritable.bench.ts`, `serialization.bench.ts`,
`dom/editorCycle.bench.ts`, `dom/editorOperations.bench.ts`.

Desenho que sustenta o desempenho:
1. **imutabilidade estrutural barata** — `GenMap` (mapa persistent/copy-on-write) em vez de `Map`
   clonado (`LexicalGenMap.ts`, 266 linhas);
2. **dirty tracking** por folha e por elemento, com caminho rápido para subárvore limpa
   (`$reconcileNode`, `:1816-1844`);
3. **cache de texto por subárvore no DOM** (`__lexicalTextContent`);
4. **comandos com early-exit** por prioridade;
5. **transforms por tipo em cache** (`transformsCache` em `$applyTransforms`).

Medido por nós (não é o bench oficial), `lexical@0.53.0` publicado, Node 22, sem DOM. Cinco
execuções do mesmo script, com a máquina compartilhada — a faixa importa mais que o ponto:

| Cenário | Tempo (5 execuções) | Tamanho |
|---|---|---|
| construir 100 parágrafos | 11,6–27,0 ms | ~24 KB JSON |
| construir 1.000 parágrafos | 13,7–63,6 ms | 243 KB JSON |
| construir 5.000 parágrafos | 49,4–231,4 ms | 1,22 MB JSON |
| `toJSON()` de 1.000 / 5.000 | 3,9–14,3 / 10,4–83,0 ms | — |
| `JSON.stringify` de 1.000 / 5.000 | 1,4–5,6 / 7,1–13,4 ms | — |
| 200 updates discretos em 1.000 parágrafos | 190,9–699,1 ms total (0,95–3,50 ms cada) | — |

A dispersão chega a ~5× entre execuções: trate como ordem de grandeza. Os números menores são de
execuções em máquina ociosa; os maiores, de execuções concorrendo com outros processos.

Limites conhecidos:
- documento inteiro é re-serializado por `toJSON`; não há serialização incremental;
- `editorState` guarda **todas** as versões referenciadas — manter muitos snapshots é caro;
- updates discretos em documentos grandes custam O(subárvore suja), não O(1): o número acima
  (1 ms/edição em 1k parágrafos) é o que se deve esperar de digitação em documento grande sem
  otimização de UI.

## 13. Erros e diagnóstico

- `createEditor({onError, onWarn})`: `onError` padrão é `console.error`
  (`LexicalEditor.ts:1127-1130`). O `LexicalBuilder` embrulha para incluir o editor
  (`LexicalBuilder.ts:186-197`).
- Erros dentro de `update` são capturados, o estado pendente é descartado/revertido e o erro vai
  para `_onError` — a exceção **não** propaga para o chamador em todos os caminhos; por isso existe
  `LexicalErrorBoundary` no React para erros de render de decorators
  (`lexical-react/src/LexicalErrorBoundary.tsx:28-60`).
- DEV-only: invariants com mensagem longa e acionável. Exemplo real observado no probe:
  `$getTextNodeOffset: invalid offset 5 for size 4 at key 1` — aponta o offset, o tamanho e a chave.
- `@lexical/devtools` + `lexical-devtools-core` (812 KB) oferecem painel de árvore/estado no
  navegador; o playground embute um `TreeView`.
- `@lexical/eslint-plugin` valida regras de `$`-funções (chamadas só em contexto de editor).

## 14. O que isso significa para o `rich-editor-auy`

| Decisão do Lexical | Copiar / adaptar / evitar | Por quê |
|---|---|---|
| Core sem comportamento de edição | **copiar** | Modo é composição; permite Rust headless com N front-ends |
| `NodeMap` + `getWritable` + versões imutáveis | **copiar** | Snapshot barato, diff barato, colaboração viável |
| Lista duplamente ligada de filhos | **adaptar** | Ótimo para split/move; ruim para acesso aleatório — em Rust, `Vec` + índices estáveis pode ser melhor |
| Bitmask `format` por run de texto | **copiar** | Compacto, serializa como número, diff trivial |
| Transforms em fixpoint leaves→elements | **copiar** | Simples de explicar e de implementar |
| Reconciliação por `NodeKey` | **copiar** | Reuso de DOM, sem virtual DOM completo |
| `contenteditable` como superfície | **adaptar com cautela** | É a origem de 80% da complexidade (IME, selection, mutation observer) |
| Três sistemas de config de nó | **evitar** | Escolher schema declarativo único |
| Tema como papéis → classes | **copiar** | "zero CSS" com integração limpa |
| Estilos inline no root/export | **evitar** | Viola neutralidade; host deve decidir |
| Extensões com `dependencies` + `conflictsWith` + `dispose` | **copiar** | Grafo validável na inicialização |
| A11y como pacote opt-in | **adaptar** | Deve ser default no nosso caso, não plugin |
