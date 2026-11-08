# Lexical — dossiê de referência

**Repositório estudado:** `ref/lexical/` (monorepo `@lexical/monorepo`, MIT, Meta Platforms).
**Versão no checkout:** `0.53.0` (`ref/lexical/package.json:4`), commit `5326e06277973c8725fb3a4e33bd2c702941a258`
(2026-11-06). O checkout está **à frente** do publicado no npm: algumas APIs mudaram depois da
`0.53.0` (ver §"O que mudou e nos morde").
**Licença:** MIT (`ref/lexical/LICENSE:1-3`). Cópia de arquivos completos permitida com aviso.

---

## 1. Para que serve

Lexical é um **framework de editor de texto rico** — não um editor pronto. Ele entrega:

1. um **modelo de documento** em árvore com nós tipados (`LexicalNode`), guardado num `Map`
   imutável por versão (`EditorState`);
2. um **ciclo de transação** (`editor.update(fn)`) com transformações declarativas;
3. um **reconciliador de DOM** que mantém um `contenteditable` sincronizado com esse modelo;
4. um **sistema de comandos** (`COMMAND_PRIORITY_*`) que traduz eventos do navegador em intenções;
5. um **sistema de extensões** (`defineExtension` / `configExtension` / `LexicalBuilder`) para
   compor modos (rich-text, plain-text), nós, comportamento e UI.

O que ele **não** traz: nenhum CSS, nenhum botão, nenhum tema visual, e — o ponto mais importante —
**nenhum comportamento de edição no núcleo**. O core não sabe o que a tecla Enter faz.

Evidência: o núcleo só registra handlers que convertem evento → comando
(`ref/lexical/packages/lexical/src/LexicalEvents.ts:1153-1175`: `BEFORE_INPUT`, `INPUT`,
`COMPOSITION_START`, `COMPOSITION_END`, `KEY_DOWN`). Quem trata `KEY_ENTER_COMMAND`,
`DELETE_CHARACTER_COMMAND`, `FORMAT_TEXT_COMMAND` é o pacote de modo:
`ref/lexical/packages/lexical-rich-text/src/index.ts:1286` (`registerRichText`).

Medido (ver `exemplos/`; saída literal em `exemplos/saida-observada.txt`):

```
KEY_ENTER_COMMAND tratado só pelo núcleo? false
filhos do root: [ 'paragraph' ]
=== 2) mesmo editor + registerRichText ===
KEY_ENTER_COMMAND tratado? true
filhos do root: [ 'paragraph', 'paragraph' ]
texto: "Hello world\n\n"
```

Nuance: "não sabe" é sobre **tratar**. O núcleo já traduz um teclado padrão fixo — setas, Enter,
Tab, `Delete` de palavra, Ctrl/Cmd+B/I/U e Ctrl/Cmd+Z/Shift+Z — em comandos
(`ref/lexical/packages/lexical/src/LexicalEvents.ts:2163-2240`). Ele despacha; quem age é o modo.

## 2. Produto vs. biblioteca

| É biblioteca | É produto (não copiar) |
|---|---|
| `lexical` (core, 4,3 MB de fonte) | `packages/lexical-playground` (6,6 MB, app React de demonstração) |
| `@lexical/rich-text`, `@lexical/plain-text`, `@lexical/list`, `@lexical/table`, … | `packages/lexical-website` (site de docs) |
| `@lexical/react` (bindings, não o modelo) | `examples/*` (24 diretórios de exemplo, cada um com `package.json` próprio) |

O playground é o lugar onde a API é exercitada de verdade — 100 **arquivos** de teste em
`packages/lexical/src/__tests__/unit/` (não 100 casos) e uma suíte Playwright em
`packages/lexical-playground/__tests__/e2e/` (`Composition.spec.mjs`, `Collaboration.spec.mjs`,
`History.spec.mjs`, `Keyboard.spec.mjs`, …). Ele **não** é a biblioteca.

## 3. Modelo mental em cinco frases

1. O documento é uma **árvore de nós**, não um DOM. Cada nó tem `__type`, `__key`, `__parent`,
   `__prev`, `__next` (`ref/lexical/packages/lexical/src/LexicalNode.ts:1070-1085`).
2. Todo o documento vive num **`Map<NodeKey, LexicalNode>`** — o `NodeMap` — dentro de um
   `EditorState` (`ref/lexical/packages/lexical/src/LexicalEditorState.ts:193-261`).
3. `EditorState` é **imutável**: mutar um nó exige `node.getWritable()`, que clona o nó, troca no
   `Map` da versão *pendente* e marca o nó como *dirty*
   (`ref/lexical/packages/lexical/src/LexicalNode.ts:1645-1690`).
4. `editor.update(fn)` abre uma **transação**: clona o estado, roda `fn`, roda *transforms* até o
   fixpoint, coleta nós órfãos e então reconcilia o DOM
   (`ref/lexical/packages/lexical/src/LexicalUpdates.ts:1308-1400` e `:622-700`).
5. **Tudo o que o usuário faz** — tecla, clique, IME, colar — é traduzido em um **comando**
   (`dispatchCommand`), e comandos são interceptáveis por prioridade. Não há callback de
   "onKeyDown" no modelo: há `KEY_ENTER_COMMAND`.

## 4. Status e maturidade

- **Versão:** `0.53.0`; o prefixo `0.` é real: a API ainda quebra entre minors. O
  `CHANGELOG.md` tem 131 seções de versão e documenta cada quebra com `Detect:` e `Migration:`
  (`ref/lexical/CHANGELOG.md:1-25`). Há **130** changesets pendentes em `ref/lexical/.changeset/`
  (131 arquivos `*.md`, incluindo o `README.md` do diretório).
- **Adoção:** mantido pela Meta; o site do projeto afirma que é o editor por trás do
  Facebook/Workplace. O **repo não traz essa evidência** — não há telemetria, lista de produtos
  nem benchmark de adoção; `@lexical/react` é o binding "oficial" (único com pacote próprio de
  UI).
- **Governança:** monorepo único, todos os pacotes versionados juntos
  (`ref/lexical/CHANGELOG.md:3`: "All Lexical packages are released together at one version").
- **Risco de manutenção:** alto em relação a mudanças de API (três sistemas de configuração de nó
  coexistem hoje: `static getType/clone/importJSON`, `$config()` e `NodeState`/schema declarativo —
  ver `arquitetura.md` §6). Mas o *modelo* (nós + NodeMap + update) é estável desde 0.3.

## 5. Resultado visível ao integrar

Depois de `editor.setRootElement(el)`, o elemento recebe:

- `contenteditable` (responsabilidade sua),
- `data-lexical-editor="true"`,
- três estilos **inline** forçados: `user-select: text`, `white-space: pre-wrap`,
  `word-break: break-word` (`ref/lexical/packages/lexical/src/LexicalEditor.ts:1749-1751`),
  escritos por `setRootElement` **depois** de qualquer CSS do host — inline vence folha de estilo,
  então o host não consegue trocar esses três sem `!important`.

Cada nó cria seu próprio DOM em `createDOM`. O parágrafo vira `<p>` (`ParagraphNode.createDOM`,
`ref/lexical/packages/lexical/src/nodes/LexicalParagraphNode.ts:72-80`); texto formatado vira
`<strong>`, `<em>`, `<code>`, `<mark>`, `<sub>`, `<sup>`, cada um com `data-lexical-text="true"`
(`LexicalTextNode.ts:168-192`; `LexicalReconciler.ts:771-773`).

HTML exportado de um documento com negrito+itálico, observado no probe:

```html
<p class="par"><b><strong class="b" style="white-space: pre-wrap;">bold</strong></b><span style="white-space: pre-wrap;"> </span><i><em class="i" style="white-space: pre-wrap;">italic</em></i><span style="white-space: pre-wrap;"> underline</span></p>
```

Duas coisas a notar: (a) classes de tema (`par`, `b`, `i`) só existem porque foram declaradas no
`theme`; sem tema, o HTML sai **sem classe nenhuma**; (b) o export injeta `style="white-space:
pre-wrap"` inline em cada texto (`LexicalTextNode.ts:998`, e `text-transform` para
lowercase/uppercase/capitalize em `:1000-1008`) — o "zero CSS" vale para o *runtime*, não para o
*HTML exportado*.

JSON do mesmo documento (`editorState.toJSON()`) — este é o formato de persistência:

```json
{"detail":0,"format":1,"mode":"normal","style":"","text":"bold","type":"text","version":1}
```

`format: 1` = `IS_BOLD`; `2` = `IS_ITALIC`; bitmask em
`ref/lexical/packages/lexical/src/LexicalConstants.ts:38-56`.

## 6. Exemplo mínimo real

O exemplo *vanilla* do repo é o mais honesto porque não passa por React:
`ref/lexical/examples/vanilla-js/src/main.ts:16-20`.

```ts
import {buildEditorFromExtensions, HMRExtension} from '@lexical/extension';
import {configExtension} from 'lexical';
import {AppExtension} from './AppExtension';

const editor = buildEditorFromExtensions(
  AppExtension,
  configExtension(HMRExtension, {hot: import.meta.hot ?? null}),
);
editor.setRootElement(document.getElementById('lexical-editor'));
```

E o conteúdo inicial + tema, em `AppExtension.ts:20-44` (cópia integral em
`codigo/exemplo-AppExtension.ts`):

```ts
export const AppExtension = defineExtension({
  $initialEditorState() {
    $getRoot().append($createParagraphNode().append($createTextNode('Hello world')));
  },
  dependencies: [
    RichTextExtension,
    ClipboardDOMImportExtension,
    configExtension(HistoryExtension, {delay: 300}),
  ],
  name: '@lexical/examples/vanilla-js',
  namespace: 'Vanilla JS Demo',
  register(editor) {
    const stateRef = document.querySelector<HTMLTextAreaElement>('#lexical-state')!;
    return editor.registerUpdateListener(({editorState}) => {
      stateRef.value = JSON.stringify(editorState.toJSON(true), null, 2);
    });
  },
  theme: {
    paragraph: 'editor-paragraph',
    text: {bold: 'editor-text-bold', italic: 'editor-text-italic'},
  },
});
```

O HTML (`ref/lexical/examples/vanilla-js/index.html:11-19`) NÃO usa React:

```html
<div id="lexical-editor" contenteditable="true" role="textbox"
     aria-label="Rich text editor" aria-multiline="true"></div>
```

Observação importante: `role="textbox"`, `aria-label` e `aria-multiline` são escritos **à mão**.
O Lexical não os adiciona.

Equivalente em React (`ref/lexical/packages/lexical-react/src/LexicalComposer.tsx`,
`LexicalContentEditable.tsx`) — mais código, mesma máquina:

```tsx
<LexicalComposer initialConfig={{namespace: 'x', nodes: [HeadingNode, QuoteNode], theme}}>
  <RichTextPlugin contentEditable={<ContentEditable aria-placeholder="..." />}
                  placeholder={<p>Digite…</p>}
                  ErrorBoundary={LexicalErrorBoundary} />
</LexicalComposer>
```

## 7. O que você precisa instalar para "editar de verdade"

Não existe "instale e edite". A tabela abaixo é a distância entre `lexical` puro e algo usável,
com o pacote que fecha cada lacuna:

| Capacidade | Pacote / extensão | Evidência |
|---|---|---|
| digitar, Enter, Backspace, deletar palavra | `@lexical/rich-text` (`registerRichText`) | `lexical-rich-text/src/index.ts:1286+` |
| undo/redo | `@lexical/history` | `lexical-history/src/index.ts:613` |
| colar HTML/Lexical | `@lexical/clipboard` | `lexical-clipboard/src/clipboard.ts:89,118` |
| serializar HTML | `@lexical/html` | `lexical-html/src/index.ts:255` |
| título/citação | `HeadingNode`, `QuoteNode` (dentro de rich-text) | `lexical-rich-text/src/LexicalRichTextExtension.ts:130` |
| listas | `@lexical/list` | `packages/lexical-list` |
| links | `@lexical/link` | `packages/lexical-link` |
| markdown | `@lexical/markdown` | `lexical-markdown/src/index.ts` (exports) |
| colaboração | `@lexical/yjs` | `lexical-yjs/src/Bindings.ts:175` |
| anúncios para leitor de tela | `@lexical/a11y` | `lexical-a11y/src/index.ts:506` |
| foco/roving tabindex/trap | `@lexical/a11y` | `lexical-a11y/src/index.ts:696-729` |

## 8. Prós

1. **Modelo de dados limpo e serializável.** `NodeMap` + `toJSON` é uma árvore JSON estável; nada
   de "documento = DOM". Round-trip JSON verificado no probe
   (`parseEditorState` → `setEditorState` → texto correto).
2. **Transações com semântica.** `editor.update` é o único lugar onde se muta; leitura é
   `editor.read`/`editorState.read`. Imutabilidade por versão permite `EditorState` inteiro como
   snapshot e comparação barata (`editorStateHasDirtySelection`,
   `LexicalEditorState.ts:66-84`).
3. **Extensibilidade por nó, não por plugin mágico.** Um nó novo = dados + `createDOM` +
   `updateDOM` + `exportJSON`/`importJSON` + `importDOM` + `transform`. Tipado de ponta a ponta.
4. **Colaboração de verdade.** Binding Yjs bidirecional com cursores e `XmlText` por nó
   (`lexical-yjs/src/SyncEditorStates.ts:195,359`), mais um V2 experimental
   (`:489,576`).
5. **Acessibilidade tratada como pacote, não como afterthought.** Região `aria-live` única,
   anúncio de undo/redo e de mudança editable, focus trap e roving tabindex são extensões
   (`lexical-a11y/src/index.ts:506,590,649,696,709,722`).
6. **Desempenho medido e instrumentado.** Benchmarks versionados para `GenMap`, `getWritable`,
   ciclo de update e serialização (`packages/lexical/src/__bench__/README.md`).

## 9. Contras

1. **Curva inicial alta e em camadas.** Três "jeitos" de definir um nó convivem (estáticos
   clássicos, `$config()`, schema declarativo + `NodeState`). Um iniciante lê a doc de 2022 e o
   código de 2026 e não batem.
2. **Acoplamento ao DOM real.** O core assume `contenteditable`, `MutationObserver`, `Selection`,
   `beforeinput`, quirks de iOS/Android/Firefox. `$getDocument()` deriva do editor ativo
   (`LexicalUtils.ts:2446-2449`); gerar HTML exige `document`
   (`lexical-html/src/index.ts:255-273` lança em headless sem DOM).
3. **Modo headless é limitado.** `createHeadlessEditor` proíbe `focus`, `getRootElement`,
   `registerRootListener` (`lexical-headless/src/index.ts:26-41`).
4. **Sistema de comandos exige disciplina.** `dispatchCommand` devolve "algum listener tratou?".
   Comandos de *notificação* (ex.: `CAN_UNDO_COMMAND`) não respondem nada — no probe,
   `dispatchCommand(CAN_UNDO_COMMAND)` retornou `false` embora `undo` funcionasse; o valor só
   aparece registrando um listener (`flags.undo === true`).
5. **Sem CSS, o resultado cru é feio e semanticamente errado em parte.** `indent`/`format` de
   elemento vêm por atributo/estilo inline; listas e tabelas dependem de classes de tema.
6. **Churn de API.** O HEAD do checkout está à frente do npm `0.53.0`: `diff -rq` entre
   `node_modules/lexical/src` (npm) e `packages/lexical/src` (HEAD) acusa 12 arquivos diferentes
   e um arquivo novo (`LexicalWarning.ts`). A assinatura de `registerHistory` é **idêntica** nas
   duas versões — `(editor, historyState, delay, dateNow?, onHistoryStateChange?, maxDepth?)`,
   `lexical-history/src/index.ts:613-620`; a divergência que este dossiê afirmava antes **não
   existe** (o `delayOrStore` é parâmetro interno de `createMergeActionGetter`, `:283`).
7. **React é o caminho mais documentado.** `@lexical/react` tem ~50 plugins prontos; o caminho
   vanilla/Svelte/PHP exige reimplementar UI (toolbar, typeahead, menus) que o React ganha de graça.

## 10. Acoplamento ao React — resposta curta

**O modelo é 100% independente de React.** Prova estrutural: `lexical/package.json` declara apenas
`@lexical/internal` como dependência (`ref/lexical/packages/lexical/package.json`), e
`examples/vanilla-js` funciona sem React. O React só aparece em `@lexical/react`, que é uma camada
de *bindings*: `LexicalComposer` guarda o editor em contexto, `ContentEditable` chama
`setRootElement`, cada `*Plugin` chama `registerX` e devolve o `unregister` num `useLayoutEffect`.

Custo real de não usar React: você escreve (a) o mount/`setRootElement`, (b) a toolbar chamando
`editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold')`, (c) o `registerUpdateListener` para
sincronizar estado de UI, (d) typeahead/menus. Nada disso é difícil, mas é código que o binding
React já tem.

## 11. Adequação ao `rich-editor-auy`

Nosso alvo é um editor **headless em Rust**, sem CSS, com front-end TS/PHP e referências entre
publicações. Veredito por peça:

**Copiar (conceito, não código):**
- A separação **evento → comando → mutação de modelo → reconciliação**, com o núcleo sem
  comportamento de edição. É o desenho que permite "modo" ser composição, não flag.
- `NodeMap` imutável por versão + `getWritable()` como única porta de mutação.
- Bitmask de formatação inline (`format: number`) por *run* de texto — compacto e diff-friendly
  (`LexicalConstants.ts:38-56`).
- Transforms em fixpoint, com *leaves antes de elements* e reset do conjunto dirty
  (`codigo/LexicalUpdates-transforms.ts`).
- Tema como **mapa de papéis → tokens** (`theme.paragraph`, `theme.text.bold`), jamais como CSS
  embutido: casa exatamente com "zero CSS" no core.

**Adaptar:**
- O modelo de extensão (`dependencies`, `config`, `build`, `register`, `conflictsWith`) — em Rust
  isso é um registry estático com validação de grafo na inicialização.
- `DOMConversionMap` (regras `tag → converter`) como tabela de import; nosso caso precisa de um
  parser próprio, mas a forma "regra por tag, com prioridade" é boa.
- Schema declarativo de estrutura (`children: 'inline' | 'block' | [...]`,
  `LexicalNode.ts:137-140`) — é o que mais se parece com o que queremos ("schema e restrições de
  conteúdo" sem escrever validador à mão).

**Evitar:**
- Três sistemas de configuração de nó ao mesmo tempo. Escolher UM schema declarativo desde o dia 1.
- Injetar estilo inline no elemento raiz (`user-select`, `white-space`, `word-break`,
  `LexicalEditor.ts:1749-1751`) e no HTML exportado (`LexicalTextNode.ts:998`). Nosso core deve
  emitir **estrutura + papéis**, e o host decide o CSS.
- Depender de `contenteditable` + `MutationObserver` como fonte de verdade. Para Rust headless,
  o modelo tem de ser a verdade e o DOM apenas projeção.
- Pacote "modo" separado do núcleo *sem* um modo default embutido: no Lexical, esquecer
  `registerRichText` produz um editor que aceita texto mas não aceita Enter — um modo de falha
  silencioso e confuso.

**Números para calibrar expectativa** (5 execuções do mesmo script, Node 22, `lexical@0.53.0`
publicado, sem DOM): construir 1.000 parágrafos levou **14–64 ms**, `toJSON` **3,9–14,3 ms**,
`stringify` **1,4–5,6 ms** (242 KB de JSON); 200 updates discretos sobre 1.000 parágrafos custaram
**0,95–3,50 ms cada**. A variância observada chega a ~5×, não ±20% — a máquina não estava isolada.
Serve como ordem de grandeza, não como benchmark. Ver `exemplos/README.md` e
`exemplos/saida-observada.txt`.

## 12. Mapa do dossiê

| Arquivo | Conteúdo |
|---|---|
| `arquitetura.md` | pacotes, árvore de nós, EditorState/updates/transforms, reconciliação, seleção/IME, comandos, extensões, schema, serialização, colaboração, a11y, performance, estilo |
| `avaliacao.md` | matriz 0–5 por dimensão + matriz de usabilidade (15 itens) com status e evidência |
| `codigo/INDEX.md` | origem → cópia, motivo, licença |
| `codigo/*.ts` | `EditorState`, `RichTextExtension`, trechos de `LexicalTextNode` e `LexicalUpdates`, exemplo vanilla |
| `exemplos/README.md` | probe real executado, comandos, saída observada, o que não foi executado |
