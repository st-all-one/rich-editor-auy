# Arquitetura do Tiptap

Todos os caminhos são relativos a `ref/tiptap/`. Cópias comentadas dos arquivos centrais estão em
`codigo/` (ver `codigo/INDEX.md`). Medições “observado” vêm da execução descrita em
`exemplos/README.md` (Chromium headless, Tiptap 3, CDN ESM).

## 1. Modelo de dados

Não existe "documento do Tiptap". O documento é um **`Node` do ProseMirror**, e o Tiptap só deriva
um `Schema` a partir das extensões:

```
extensões → resolveExtensions (flatten + sort) → getSchemaSpecByResolvedExtensions
          → new Schema(spec)  → EditorState.create({ doc, schema, selection })
```

- `getSchemaByResolvedExtensions` monta `new Schema(getSchemaSpecByResolvedExtensions(...))`, ou
  faz merge com um `schemaSpec` externo (`packages/core/src/helpers/getSchemaByResolvedExtensions.ts:19-41`).
  Esse segundo caminho é importante: permite reaproveitar um schema ProseMirror já existente e
  manter só o comportamento das extensões locais (e lança `RangeError` em tipo duplicado, linhas
  32-34).
- Cada `Node`/`Mark` contribui com **campos de `NodeSpec`/`MarkSpec`**: `content`, `marks`,
  `group`, `inline`, `atom`, `selectable`, `draggable`, `code`, `whitespace`,
  `linebreakReplacement`, `defining`, `isolating` (`helpers/getSchemaSpecByResolvedExtensions.ts:93-126`);
  em marks: `inclusive`, `excludes`, `group`, `spanning`, `code` (idem, 191-205).
- **Atributos** viram `attrs` do schema por `buildAttributeSpec`, que só repassa `default` e
  `validate` (linhas 32-51). Todo o resto (`parseHTML`, `renderHTML`, `rendered`) é
  comportamento de DOM, não schema.
- O schema do doc é `block+` (`packages/extension-document/src/document.ts:6-9`) e o parágrafo é
  `inline*` com `priority: 1000` (`packages/extension-paragraph/src/paragraph.ts:63-71`) — o
  `priority` só ordena processamento, não conteúdo.
- Conteúdo inicial pode ser string HTML, JSON, `Node` ou `Fragment`
  (`helpers/createNodeFromContent.ts:21-127`). JSON inválido cai em `console.warn` e vira string
  vazia, **a menos que** `enableContentCheck: true` esteja ligado (linhas 56-64).

Observado: com StarterKit 3, `schema.nodes` = `[paragraph, blockquote, bulletList, codeBlock,
doc, hardBreak, heading, horizontalRule, listItem, orderedList, text]` e `schema.marks` =
`[link, bold, code, italic, strike, underline]`.

## 2. Extensões: herança trocada por composição

`Extendable` (copiado em `codigo/core-Extendable.excerpt.ts`) é a base de `Extension`, `Node` e
`Mark` (`packages/core/src/Extension.ts:14-20`, `Node.ts:341-350`, `Mark.ts:143-150`):

- `options` é **getter**: sempre `{...callOrReturn(addOptions())}` (`Extendable.ts:615-623`);
  `storage` idem (`625-633`). Não há campo mutável — `configure()` cria uma cópia com
  `addOptions: () => mergeDeep(this.options, options)` (`Extendable.ts:636-650`).
- `extend()` cria nova instância com `parent = this` e `child = extension`
  (`Extendable.ts:652-668`). Essa cadeia é o que permite `getExtensionField` responder “herde do
  pai se eu não definir, e passe o valor do pai como `this.parent`” — inclusive recursivamente
  (`packages/core/src/helpers/getExtensionField.ts:12-31`).
- **Configuração é função de contexto**: qualquer campo pode ser função, e o `this` recebe
  `{name, options, storage, editor, type, parent}` — é por isso que `addCommands()` pode fazer
  `commands.setMark(this.name)` sem saber o nome em tempo de definição
  (`packages/core/src/ExtensionManager.ts:70-102`).
- **Composição**: `addExtensions()` empacota outras extensões — StarterKit é exatamente isso:
  um `Extension.create` que empurra 22 extensões condicionadas por `options.x !== false`
  (`packages/starter-kit/src/starter-kit.ts:213-273`).
- **Ordem**: `resolveExtensions` = `sortExtensions(flattenExtensions(...))` e avisa em nome
  duplicado (`helpers/resolveExtensions.ts:13-25`); `priority` maior vem antes, default 100
  (`helpers/sortExtensions.ts:13-25`).
- **Plugins invertem a ordem de propósito**: no ProseMirror o primeiro plugin manda; no Tiptap as
  extensões são invertidas e reordenadas para que a **última** extensão registrada tenha
  precedência (`ExtensionManager.ts:104-112`). Esse é um detalhe fácil de errar ao reimplementar.
- **`destroy()` limpa a cadeia `parent.child`** para não vazar extensões compartilhadas via
  módulo (`ExtensionManager.ts:464-497`) — sinal de que a cadeia de `extend()` é um ponto real de
  retenção de memória.

## 3. Commands, `chain()` e `can()`

`CommandManager` (copiado inteiro em `codigo/core-CommandManager.ts`) é o adaptador entre
"intenção" e `Transaction`:

- `commands` (getter) percorre `rawCommands`, e para cada um devolve uma função que roda
  `command(...args)(props)` e **despacha o `tr`** se `preventDispatch` não estiver setado e não
  houver `customState` (`CommandManager.ts:34-56`).
- `chain()` cria `tr = startTr || state.tr` uma vez, empilha os retornos em `callbacks` e só
  despacha no `run()` — que devolve `callbacks.every(cb => cb === true)`
  (`CommandManager.ts:66-99`). Consequência de UX: um `chain().focus().toggleBold().run()` pode
  devolver `false` mesmo tendo mudado o foco, se algum comando falhar.
- `can()` roda os mesmos comandos com `dispatch: undefined` (`CommandManager.ts:134-152`), o que
  é o que torna barato desabilitar botões. O próprio repo testa o efeito de “can” não mutar
  estado (`packages/core/src/CommandManager.test.ts:19-61`).
- `buildProps` injeta `state` como **`createChainableState`** (`CommandManager.ts:174-208`): um
  estado que acompanha o `tr` em construção. Sem isso, um comando encadeado leria o documento
  antigo.
- `preventDispatch` é usado por extensões que querem controle manual — a colaboração seta
  `tr.setMeta('preventDispatch', true)` antes de chamar o undo do Yjs
  (`packages/extension-collaboration/src/collaboration.ts:126-143`).

## 4. Ciclo de vida

Ordem real no construtor (`packages/core/src/Editor.ts:133-190`):

1. `createExtensionManager()` → resolve extensões, constrói `schema`, inicializa `storage`s e
   **liga os hooks da extensão aos eventos do editor** (`ExtensionManager.ts:504-588`).
2. `createCommandManager()`, `createSchema()`.
3. `createDoc()` (parse do `content`), `resolveFocusPosition(autofocus)`, `EditorState.create`.
4. Se `options.element` existe → `mount()`.

`mount(element)` (`Editor.ts:192-241`) exige `document` (lança erro em SSR sem DOM, 196-200),
chama `unmount()` do view anterior, `createView()`, emite `mount`, anexa o style tag, e em
`setTimeout(..., 0)` emite **`create`** e marca `isInitialized`. Ou seja: `onCreate` **não** roda
de forma síncrona dentro do construtor.

`createView` (`copiado em codigo/core-Editor.createView.excerpt.ts`) faz duas coisas não óbvias:

- instancia o `EditorView` com `state.reconfigure({ plugins: [] })` e **só depois** aplica
  `setProps({ state, markViews, nodeViews })` — evita plugins rodando antes das node views
  existirem (`Editor.ts:646-668`);
- compõe `dispatchTransaction` e `transformPastedHTML` percorrendo todas as extensões
  (`Editor.ts:640-645`; composição em `ExtensionManager.ts:330-420`).

Eventos disponíveis: `beforeCreate`, `mount`, `create`, `unmount`, `destroy`, `update`,
`selectionUpdate`, `transaction`, `beforeTransaction`, `focus`, `blur`, `contentError`, `paste`,
`drop`, `delete` (`packages/core/src/types.ts:52-115` e `Editor.ts:150-167`).

`unmount()` preserva o `EditorState` (`this.editorState = this.editorView.state`) e remove o
`<style>` **só se não houver outro `.tiptap` na página** (`Editor.ts:222-251`; teste explícito em
`Editor.test.ts:670-708`). Remontar troca conteúdo? Não: `editorLifecycle.test.ts:21-35` mostra
conteúdo preservado após `unmount`/`mount`.

`setOptions` é deliberadamente parcial: apenas 19 chaves são runtime (`editable`, `editorProps`,
`parseOptions`, `enableContentCheck`, `emitContentError` e os `on*`)
(`utilities/getEditorRuntimeOptions.ts:3-23`); qualquer outra chave só gera `console.warn` em dev
(`Editor.ts:305-320`). Mudar `extensions` exige editor novo — decisão correta para nós copiarmos.

## 5. Renderer e DOM

- **Sem React/DOM virtual**: o Tiptap usa `renderHTML` → `toDOM` do schema
  (`helpers/getSchemaSpecByResolvedExtensions.ts:138-150`) e o `DOMSerializer` do ProseMirror para
  serializar (`helpers/getHTMLFromFragment.ts:4-12`).
- **Raiz e `<style>`**: o elemento raiz recebe a classe `tiptap` concatenada **à frente** do
  `className` existente (`prependClass()`, `Editor.ts:697-699`), e o CSS vai para **uma única**
  `<style data-tiptap-style>` por documento: `createStyleTag` faz `querySelector` do seletor e
  devolve a tag já existente se houver (`utilities/createStyleTag.ts:2-8`), aceitando `nonce`
  (`injectNonce`) e um `suffix` para variantes. `unmount()` remove a tag **só** quando não resta
  nenhum `.tiptap` no documento (`Editor.ts:236-250`).
- **Atributos**: `addAttributes()` define `{default, parseHTML, renderHTML, rendered}`; o DOM final
  é `mergeAttributes(HTMLAttributes da extensão, atributos renderizados)`
  (`helpers/getRenderedAttributes.ts:6-23`). `mergeAttributes` concatena `class` **sem duplicar** e
  funde `style` por propriedade (último vence) — verificado em execução:
  `{class:'prose p-4', style:'color:red'}` + `{class:'prose custom', style:'color:blue; margin:0'}`
  ⇒ `class: "prose p-4 custom"`, `style: "color: blue; margin: 0"`
  (`packages/core/src/utilities/mergeAttributes.ts:101-123`, saída em `exemplos/README.md`).
- **JSX opcional**: o core publica seu próprio `jsx-runtime` (`packages/core/src/jsx-runtime.ts`,
  exportado em `index.ts:9`), então extensões podem escrever
  `/** @jsxImportSource @tiptap/core */` + `<strong {...attrs}><slot/></strong>`
  (`packages/extension-bold/src/bold.tsx:1,85-91`). O `<slot/>` é o “buraco” 0 do `DOMOutputSpec`.
- **Node views**: `addNodeView()` devolve um renderer; o `ExtensionManager` monta o mapa
  `name → NodeViewConstructor` e injeta `editor`, `extension`, `HTMLAttributes`
  (`ExtensionManager.ts:258-320`). React/Vue usam isso para renderizar componentes no lugar do
  DOM nativo (`packages/react/src/ReactRenderer.tsx`, `packages/vue/src/VueRenderer.ts`).
- **Mark views**: mecanismo análogo e mais novo (`addMarkView`, `ExtensionManager.ts:422-460`,
  `packages/core/src/MarkView.ts`).
- **Sem Editor**: `@tiptap/static-renderer` reconstrói o schema e roda `renderHTML` sem `Editor`;
  por isso `addProseMirrorPlugins`/`onCreate` **não** rodam e atributos gerados por plugin (ex.:
  `data-id` do UniqueID) precisam de pré-processamento explícito
  (`packages/static-renderer/README.md:9-40`). Ele escapa texto/atributos e descarta
  `on*`/`srcdoc`, mas **não é sanitizer** (idem, 40-47).

## 6. Seleção, teclado, IME e clipboard

- **Teclado**: `Keymap` do core registra só fallbacks de `Enter`/`Backspace`/`Delete`/`Mod-a` e
  variações Mac (`packages/core/src/extensions/keymap.ts:19-136`); o resto vem de cada extensão
  via `addKeyboardShortcuts` (`ExtensionManager.ts:126-158`). `Mod-b`, `Mod-i`, `Mod-z` etc. são
  declarados por extensão (`bold.tsx:131-136`).
- **Ordem de teclas**: cada extensão registra seu próprio `keymap(...)` plugin; como as extensões
  são invertidas, a última registrada ganha (`ExtensionManager.ts:104-112`). É preciso conhecer
  essa regra para não criar conflito silencioso.
- **Input rules**: `handleTextInput` reescreve o texto digitado; `compositionend` é tratado com
  `setTimeout` para rodar a regra **depois** do IME (`packages/core/src/InputRule.ts:324-338`).
  `Enter` também dispara regras (útil em code block), `InputRule.ts:341-360`.
- **IME**: evidência de código, não de execução — `view.composing` é consultado em
  `packages/extension-details/src/details.ts:459-461`; bubble/floating menu reagem a
  `compositionend` (`extension-bubble-menu/src/bubble-menu-plugin.ts:413`,
  `extension-floating-menu/src/floating-menu-plugin.ts:325`); há changesets específicos para
  corrigir menus durante composição (`.changeset/fix-bubble-menu-ime-composition.md`,
  `.changeset/fix-floating-menu-ime-composition.md`). O comportamento real de IME (japonês,
  chinês, teclado virtual Android/iOS) **não foi testado** neste dossiê.
- **iOS/Android**: há detecção por UA e tratamento de `Enter` móvel
  (`packages/core/src/utilities/isiOS.ts`, `isAndroid.ts`,
  `packages/react/src/handleMobileEnter.ts` + plugin registrado em
  `EditorContent.tsx:139-148`). Não verificado em dispositivo.
- **Clipboard**: o core registra `Paste` (emite evento `paste`, `packages/core/src/extensions/paste.ts:6-25`),
  `Drop`, `Delete` e `ClipboardTextSerializer`, que serializa o texto por **ranges** da seleção
  (suporta `CellSelection` com múltiplos ranges) `packages/core/src/extensions/clipboardTextSerializer.ts:25-53`.
  Colagem de HTML passa por `transformPastedHTML` composto por extensão
  (`ExtensionManager.ts:380-420`), usado pelo Link (`autolink`/`pasteHandler`,
  `packages/extension-link/src/link.ts:1-17`).
- **Seleção**: é a do ProseMirror (`TextSelection`, `NodeSelection`, `CellSelection`); o Tiptap
  adiciona açúcar — `editor.$pos`, `editor.$node`, `NodePos`
  (`Editor.ts:900-940`, `packages/core/src/NodePos.ts`).

## 7. Serialização

| Formato | Caminho | Observação |
|---|---|---|
| JSON | `editor.getJSON()` → `state.doc.toJSON()` (`Editor.ts:792-800`) | É o mesmo JSON do ProseMirror; é o formato recomendado para persistir. |
| HTML | `editor.getHTML()` → `getHTMLFromFragment` → `DOMSerializer` + `innerHTML` de um documento temporário (`Editor.ts:802-806`, `helpers/getHTMLFromFragment.ts:4-12`) | Passa por `renderHTML` de cada node/mark, então atributos/estilos das extensões aparecem na saída. |
| Texto | `editor.getText({blockSeparator, textSerializers})`, default `'\n\n'` (`Editor.ts:808-830`) | Usa `renderText`/`getTextSerializersFromSchema`; observado `"ok\n\nx"` para dois parágrafos. |
| HTML/JSON fora do browser | `@tiptap/html` lança em Node e manda usar `@tiptap/html/server` + happy-dom (`packages/html/src/generateHTML.ts:20-25`, `packages/html/package.json`) | Caminho oficial para PHP/Node pré-renderizar. |
| Sem Editor | `@tiptap/static-renderer` (`renderToHTMLString`) | Ver §5. |
| Markdown | `@tiptap/markdown` + `parseMarkdown`/`renderMarkdown`/`markdownTokenizer` por extensão (`packages/markdown/src`, hooks em `bold.tsx:93-109`) | Recurso novo; não testado aqui. |

Observado: setar `<p>ok</p><table>…</table>` num editor com StarterKit resulta em
`<p>ok</p><p>x</p>` — o nó desconhecido é **descartado**, mas o texto interno sobrevive. Sem
`enableContentCheck`, isso é silencioso (só `console.warn` em JSON inválido).

## 8. React e Vue

**React** (`packages/react/src`):

- `useEditor` envolve a instância num `EditorInstanceManager` e usa `useSyncExternalStore`
  (`useEditor.ts:40-145,310-314`). Detalhes que importam: criação adiada + `scheduleDestroy` em 1
  tick para sobreviver a StrictMode (`useEditor.ts:253-276`); `immediatelyRender` desligado
  automaticamente em SSR/Next (`useEditor.ts:100-121`); comparação de opções para evitar recriar
  o editor quando só callbacks mudaram (`useEditor.ts:172-214`).
- `useEditorState` assina `transaction`/`update`, incrementa `transactionNumber` e aplica
  `selector` + `equalityFn` (default `circularDeepEqual` de `fast-equals`)
  (`useEditorState.ts:13-30,89-118`). Com `shouldRerenderOnTransaction: false` (default), a UI só
  re-renderiza quando o recorte selecionado muda — o demo `Performance` mede isso
  (`demos/src/Examples/Performance/React/index.jsx:8-60`).
- `EditorContent` monta o editor numa `div` e renderiza os node views React em **portais**
  (`EditorContent.tsx:31-41,125-154`).

**Vue** (`packages/vue/src`): `useEditor` é `shallowRef` + `onMounted`/`onBeforeUnmount`
(`useEditor.ts:6-18`); `EditorContent` observa `[editor, rootEl]`, monta no `nextTick` e passa
`appContext`/`provides` (`EditorContent.ts:20-57`). Bem menos maquinário que o React, com a
contrapartida de não haver "estado derivado" nativo — o usuário usa `computed` sobre `editor.state`.

## 9. Colaboração (Yjs)

- `Collaboration` é uma `Extension` com `priority: 1000` que instala `ySyncPlugin` + `yUndoPlugin`
  do `@tiptap/y-tiptap` (fork de `y-prosemirror`) sobre um `Y.XmlFragment`
  (`packages/extension-collaboration/src/collaboration.ts:90-108,172-240`).
- **Dono do undo é o Yjs**: os comandos `undo`/`redo` locais são substituídos e usam
  `preventDispatch` (`collaboration.ts:125-160`), e há aviso em runtime se `undoRedo` (histórico
  local) estiver presente (`110-116`). Há um patch em `yUndoPlugin.spec.view` para restaurar o
  `UndoManager` quando a view é recriada (`177-215`) — indício de que remontar o editor com Yjs é
  frágil.
- `filterTransaction` chama `undoManager.stopCapturing()` quando a transação vem de input rule,
  para o undo não parar no meio de uma palavra (`241-249`).
- Com `enableContentCheck`, conteúdo que o schema não aceita dispara `contentError` e oferece
  `disableCollaboration()` para não sincronizar a perda de volta (`221-234,250-283`). Comentário
  explícito: transação do Yjs não pode ser cancelada, então sempre retorna `true` (276-279).
- Conversões prontas em `@tiptap/y-tiptap`: `prosemirrorJSONToYDoc`, `yDocToProsemirrorJSON`,
  `prosemirrorToYXmlFragment`, `initProseMirrorDoc` (`packages/y-tiptap/src/index.ts:16-26`).
- Cursor remoto: `@tiptap/extension-collaboration-caret` (usa `yCursorPlugin`).

## 10. Acessibilidade

O que **existe**:

- `role="textbox"` sempre aplicado, inclusive quando o usuário passa `attributes` (função ou
  objeto) — `getEditorViewAttributes` (`Editor.ts:610-630`), testado em
  `Editor.test.ts:198-271`.
- `tabindex="0"` automático quando editável, ausente quando `editable: false`
  (`packages/core/src/extensions/tabindex.ts:34-58`), configurável por
  `coreExtensionOptions.tabindex.value`.
- Foco programático: `editor.commands.focus(pos)` (`helpers/resolveFocusPosition.ts`).
- O demo `Accessibility` documenta o mínimo recomendado: `aria-label`, `aria-multiline`,
  `aria-readonly` via `editorProps.attributes`, `role="application"` no contêiner e `Alt+F10` para
  a barra de menus (`demos/src/Examples/Accessibility/React/index.tsx:11-28`,
  `useMenubarNav.ts:1-60`).

O que **não** existe (observado): `aria-multiline` não é default (o atributo veio `null`), nenhum
`aria-describedby`, nenhum anúncio de mudança de seleção/formatação para leitor de tela, e
nenhum landmark. Cabe ao integrador. Ou seja: Tiptap **não prejudica** a11y, mas também não a
resolve — a carga é toda da UI do produto.

## 11. Performance — o que é caro e onde

- **Custo é do ProseMirror/`contenteditable`**, não do wrapper: cada transação re-renderiza o DOM
  alterado e cada `update` no React pode re-renderizar a árvore. O Tiptap só evita o segundo
  (`shouldRerenderOnTransaction: false` + `useEditorState`).
- **`commands`/`can()` constroem objetos**: `CommandManager.commands` recalcula `Object.fromEntries`
  sobre todos os `rawCommands` a cada acesso (`CommandManager.ts:34-56`). Em toolbar que chama
  `can()` para cada botão em cada render, isso é O(nº comandos × nº botões).
- **`extensionManager.plugins`** recalcula e reordena todas as extensões (`ExtensionManager.ts:104-112`);
  chamado na montagem, não por transação.
- **`options` e `storage` são getters que recriam objetos** a cada acesso (`Extendable.ts:615-633`);
  código quente que leia `this.options` em loop paga alocação.
- Não há números de benchmark no repo. O demo `Performance` é instrucional, não benchmark
  (`demos/src/Examples/Performance/React/index.jsx`). **Performance não medida neste dossiê.**

## 12. Segurança e robustez (detalhes que valem copiar)

- `isValidCSSStyleValue` bloqueia `;`, `{`, `}` para impedir injeção de declaração via atributo
  (`packages/core/src/utilities/isValidCSSStyleValue.ts:1-8`), usado por TextAlign e Highlight.
- `mergeAttributes` usa `Object.defineProperty` para `__proto__`, evitando prototype pollution
  (`mergeAttributes.ts:79-91`); verificado em execução: `({}).polluted === undefined`.
- `static-renderer` escapa texto/atributos e remove `on*`/`srcdoc`, mas declara não ser sanitizer
  (`packages/static-renderer/README.md:40-47`).
- Link valida URL por `isAllowedUri`/linkifyjs antes de renderizar
  (`packages/extension-link/src/link.ts:110-140`).
- `getHTML()` usa `DOMSerializer`, então não há escape manual de atributo — quem escreve
  `renderHTML` é responsável pelo valor (daí os validadores acima).

## 13. Limites conhecidos

1. `onCreate` assíncrono (`setTimeout 0`) — não dá para assumir editor pronto após `new Editor()`.
2. Só 19 opções são runtime; trocar extensões exige recriar tudo.
3. `enableContentCheck: false` por default ⇒ perda silenciosa de nós desconhecidos.
4. Duas pilhas de undo possíveis (local e Yjs) com aviso em runtime, não erro de tipos.
5. A ordem efetiva de plugins/keymaps é invertida em relação à lista de extensões.
6. `@tiptap/html` não funciona em Node sem `happy-dom`.
7. Nenhuma API de "publicação" (link entre documentos, IDs estáveis) — `UniqueID` é extensão
   separada que depende de plugin, portanto **não roda** no static-renderer sem pré-processar.
8. **Deriva checkout-vs-publicado**: o checkout é `main` (commit `5fd5d2f`, 2026-11-05), não a tag
   3.30.3 — há 90 changesets não publicados. Divergência concreta verificada: no código de HEAD
   `isDestroyed` é `return this.destroyed` (`Editor.ts:901-903`), então sobrevive a `unmount()`;
   no **build publicado 3.30.3** é `this.editorView?.isDestroyed ?? true`, que dá `true` depois de
   `unmount()`. Rodei as duas pontas: com `@tiptap/core@3.30.3` do CDN, `isDestroyed` = `true` após
   `unmount()` (e `false`, com conteúdo preservado, depois de `mount()`); com `@3` (3.33.0), o mesmo.
   Portanto o comportamento observado **não é** uma mudança de 3.30.3→3.33.0: é a diferença entre o
   HEAD do repo e o artefato publicado. Consequência prática: citações de código e observações de
   runtime descrevem coisas ligeiramente diferentes, e a API de lifecycle não é estável o bastante
   para copiarmos por imitação.
