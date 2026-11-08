# Tiptap — dossiê de referência

> Estudo de caso sobre `ref/tiptap/` (monorepo Tiptap, 61 pacotes, MIT).
> Foco: o que é de fato "headless", o que vem com CSS/classes, como o usuário monta UI, e o que
> copiar/adaptar/evitar no `rich-editor-auy`.
>
> **Identidade da fonte (importante para toda citação):** o checkout **não é a tag 3.30.3**. É
> `main` no commit `5fd5d2f` (2026-11-05); `packages/*/package.json` ainda diz `3.30.3` porque há
> **90 changesets não publicados** (`.changeset/`, `ls .changeset | wc -l`). Ou seja: as citações
> `arquivo:linha` descrevem o **HEAD em desenvolvimento**, enquanto as execuções via CDN usam o
> **artefato publicado**. Onde os dois divergem, este dossiê marca explicitamente — e há pelo
> menos uma divergência real (§2 e `avaliacao.md` §5, item 4).

## 1. O que é (e o que não é)

Tiptap é uma **biblioteca** (não um app) que embrulha o ProseMirror: "The Tiptap Editor is a
headless, framework-agnostic rich text editor that's customizable and extendable through
extensions" (`ref/tiptap/README.md:9`) e "based on the highly reliable ProseMirror library"
(`ref/tiptap/README.md:10`). O monorepo publica ~60 pacotes independentes (`packages/*/package.json`),
um pacote de re-exportação do ProseMirror (`packages/pm/package.json:2-4`, `"description":
"prosemirror wrapper package for tiptap"`) e um pacote React/Vue só de cola (`packages/react`,
`packages/vue`).

Modelo de negócio importa para a avaliação: o repositório é o produto open source; colaboração em
tempo real (Yjs) está aqui, mas "Pro Extensions need a valid subscription"
(`ref/tiptap/README.md:37-38`). Comentários, versionamento, IA e "Tiptap Cloud" **não estão neste
repo** — não é possível avaliá-los lendo este código.

Comparação direta com o ProseMirror puro (que estudamos em `dossie/prosemirror/`):

| Aspecto | ProseMirror puro | Tiptap |
|---|---|---|
| Unidade de configuração | `Schema` + array de `Plugin` | lista de **extensões** que geram schema, plugins, comandos, teclas, regras de input/paste, node views |
| Comando | função `(state, dispatch) => boolean` | `editor.commands.x()` com dispatch automático e `chain()`/`can()` |
| Estado do editor | você instancia `EditorState`/`EditorView` | `Editor` embrulha os dois (e permite não montar) |
| UI | você escreve tudo | **também** você escreve tudo, mas com `isActive`, `can()` e hooks reactivos prontos |
| CSS | `prosemirror.css` opcional | **injeta** CSS estrutural por padrão (`injectCSS: true`) + classe `.tiptap` |

## 2. "Headless" — o que realmente significa aqui

Headless, neste projeto, quer dizer **sem UI de produto**: nenhum botão, menu, barra ou tema. Não
quer dizer "sem DOM" nem "sem CSS". A evidência:

1. **Sempre há DOM e `contenteditable`.** O editor é um `EditorView` do ProseMirror
   (`ref/tiptap/packages/core/src/Editor.ts:633-695`). Em execução real (Chromium, ver
   `exemplos/README.md`) o elemento raiz é `div.tiptap.ProseMirror` com
   `contenteditable="true"`, `role="textbox"` e `tabindex="0"`.
2. **Há CSS injetado por padrão.** `injectCSS: true` é default (`Editor.ts:100`) e `injectCSS()`
   cria uma `<style data-tiptap-style>` com o conteúdo da constante `style` de `style.ts`
   (`Editor.ts:293-297`; `packages/core/src/utilities/createStyleTag.ts:1-21`).
   Medido: **1 style tag, 1329 caracteres** — exatamente os caracteres entre as crases de
   `style.ts` (o arquivo tem 1353 bytes, que incluem `export const style = \`` e o `\n` final).
   Detalhe que o dossiê antes não registrava: `createStyleTag` **reaproveita a tag existente**
   (`querySelector('style[data-tiptap-style]')`, `createStyleTag.ts:2-8`), então o documento tem
   no máximo uma tag, por mais editores que existam — o que também torna ambíguo qualquer teste
   de "não injetou CSS" feito com um segundo editor na mesma página.
3. **A classe `tiptap` é adicionada ao elemento raiz** por `prependClass()`
   (`Editor.ts:697-699`), resultando em `tiptap ProseMirror` (nomes observados em execução; os
   testes do próprio repo dependem disso: `demos/test/helpers.ts:9` usa `page.locator('.tiptap')`).
   Como o método concatena **à frente** do `className` já existente
   (`this.view.dom.className = \`${this.className} ${this.view.dom.className}\``,
   `Editor.ts:698`), a `class` do usuário vem depois: observado
   `tiptap ProseMirror my-wrap` para `editorProps.attributes.class = 'my-wrap'`. A classe é
   imposta e não há opção para removê-la.

O que **não** vem: nenhuma cor, fonte, espaçamento, borda, ícone, menu, toolbar, popup. O CSS
injetado é funcional/estrutural — `white-space`, `font-variant-ligatures`, o cursor de gap, o
`<img class="ProseMirror-separator">` e o `[contenteditable=false]`
(`packages/core/src/style.ts:1-71`, copiado em `codigo/core-style.ts`). É praticamente o
`prosemirror.css` embrulhado, necessário para o `contenteditable` não quebrar, não um tema.

Consequência prática para nós: **"zero CSS" é atingível com Tiptap**, mas **opt-in** e precisa ser
provado num documento limpo. A prova do explorador (`styleTagsBefore === styleTagsAfter === 1` com
um segundo editor) era **inválida**: como `createStyleTag` reaproveita a tag por `querySelector`
(`createStyleTag.ts:2-8`), um segundo editor **com** CSS também deixaria a contagem em 1. Refeito
em documento isolado (iframe próprio, um único editor, Tiptap 3.30.3 fixado): `injectCSS: false`
⇒ **0 elementos `style`/`link` no head**, e o editor continua funcional
(`selectAll`+`toggleBold` ⇒ `<p><strong>sem css</strong></p>`). Ver `exemplos/README.md` §6.

O que **não** se desliga: a classe `tiptap` continua sendo prependada mesmo com `injectCSS:false`
(observado `tiptap ProseMirror`), e nenhum node/mark traz classe padrão. Busca por classes default
em `packages/*/src` não encontra nenhuma além de:
`language-*` no `<code>` do code block (`packages/extension-code-block/src/code-block.ts:153-156`,
confirmado em execução: `<pre><code class="language-js">`) e classes do lowlight
(`packages/extension-code-block-lowlight/src/lowlight-plugin.ts:61`). Tudo o mais é atributo do
usuário (`HTMLAttributes`) ou **estilo inline** — confirmado em execução:
`TextAlign` ⇒ `<p style="text-align: center;">centro</p>` e o atributo `textAlign` no JSON
(`packages/extension-text-align/src/text-align.ts:74-90`); `Highlight` com `multicolor`
emite `background-color` + `data-color` (`packages/extension-highlight/src/highlight.ts:90-99`) —
esta última lida no código, não executada.

Ou seja: o "zero CSS" do Tiptap é **opt-in** (`injectCSS:false`) e **incompleto** — o **estilo por
atributo inline** é o default de várias extensões, então quem quer zero de estilo precisa reescrever
esses `renderHTML` (ou usar só o StarterKit e passar `injectCSS:false`).

## 3. Como o usuário monta UI

Sem framework (JS puro):

```js
const editor = new Editor({ element, extensions: [StarterKit], content: '<p>hello</p>' })
editor.commands.toggleBold()
editor.isActive('bold')            // true  (observado)
editor.can().chain().undo().run()  // true  (observado)
```

- **Comandos**: `editor.commands.*` executa um comando e despacha a transação
  (`packages/core/src/CommandManager.ts:29-56`). `chain()` acumula comandos num único `tr` e
  `run()` devolve o **AND** dos retornos (`CommandManager.ts:66-99`). Consequência observada: um
  encadeamento pode **aplicar efeitos e ainda devolver `false`** —
  `chain().selectAll().toggleBold().undo().run()` devolveu `false` com o `<strong>` já no
  documento; `chain().focus().undo().run()` com histórico vazio devolveu `false`. Não use o
  retorno de `run()` como "nada mudou".
- **Habilitação de botões**: `can()` reconstrói os comandos com `dispatch: undefined`
  (`CommandManager.ts:134-152`), o que permite `disabled={!editor.can().chain().toggleBold().run()}`
  sem mutar o documento. É exatamente o que o MenuBar de referência faz
  (`demos/src/Examples/Default/React/menuBarState.ts:8-10`).
- **Estado ativo**: `editor.isActive(name, attrs)` (`Editor.ts:775-790`) e
  `editor.getAttributes()` (`Editor.ts:809-814`).
- **React**: `useEditor(options, deps)` devolve a instância e **não** re-renderiza por transação
  por padrão; `useEditorState({editor, selector, equalityFn})` assina apenas o recorte que a UI
  precisa (`packages/react/src/useEditor.ts:286-343`,
  `packages/react/src/useEditorState.ts:13-30`). `<EditorContent editor={editor}/>` só cria a
  `div` e monta o editor nela (`packages/react/src/EditorContent.tsx:125-154`), e é onde os node
  views React são renderizados via portal (`EditorContent.tsx:31-41,85-103`).
- **Vue**: `useEditor` cria no `onMounted` e destrói no `onBeforeUnmount`
  (`packages/vue/src/useEditor.ts:6-18`); `<EditorContent>` monta no `nextTick` e repassa
  `appContext`/`provides` aos node views (`packages/vue/src/EditorContent.ts:20-57`).

A UI, portanto, é **HTML/CSS/JS do usuário**: o editor só expõe comandos, consultas e eventos
(`beforeCreate`, `mount`, `create`, `update`, `transaction`, `selectionUpdate`, `focus`, `blur`,
`contentError`, `destroy` — `packages/core/src/types.ts:52-115`).

## 4. Exemplo mínimo real

O menor exemplo oficial usa só três extensões (`demos/src/Examples/Minimal/React/index.jsx:1-23`):
`Document`, `Paragraph`, `Text`. Em JS puro, o equivalente é:

```html
<div id="app"></div>
<script type="module">
  import { Editor } from '@tiptap/core'
  import StarterKit from '@tiptap/starter-kit'

  const editor = new Editor({
    element: document.getElementById('app'),
    extensions: [StarterKit],
    content: '<p>hello</p>',
  })

  editor.commands.selectAll()
  editor.commands.toggleBold()
  console.log(editor.getHTML())  // '<p><strong>hello</strong></p>'
</script>
```

Isso **foi executado** (Chromium headless, CDN esm.sh, Tiptap 3) e devolveu exatamente
`<p><strong>hello</strong></p>`; saída completa, comandos e ressalvas em `exemplos/README.md`.
O mesmo HTML está em `exemplos/cdn/index.html`.

Extensão própria também foi verificada em execução — um nó de bloco com atributo tipado, sem CSS e
sem plugin (`exemplos/cdn/probe.html`):

```js
const Callout = Node.create({
  name: 'callout', group: 'block', content: 'inline*', defining: true,
  addAttributes: () => ({
    tone: {
      default: 'info',
      parseHTML: el => el.getAttribute('data-tone') || 'info',
      renderHTML: attrs => ({ 'data-tone': attrs.tone }),
    },
  }),
  parseHTML: () => [{ tag: 'div[data-callout]' }],
  renderHTML: ({ HTMLAttributes }) =>
    ['div', mergeAttributes({ 'data-callout': '' }, HTMLAttributes), 0],
})
```

Resultado observado: `<div data-callout="" data-tone="warn">cuidado</div>` no HTML e
`{"type":"callout","attrs":{"tone":"warn"},…}` no `getJSON()` — round-trip completo.

## 5. Instalação e pré-requisitos

- Pacotes publicados no npm sob `@tiptap/*`; `@tiptap/core` depende só de `@tiptap/pm` e
  `prosemirror-model` (`packages/core/package.json`), `@tiptap/pm` reexporta os pacotes
  `prosemirror-*`. ESM com `exports` explícito, CJS e tipos
  (`packages/core/package.json:32-45`), `sideEffects: false`.
- Para consumir: `npm i @tiptap/core @tiptap/pm` + as extensões desejadas (ou `@tiptap/starter-kit`).
- **Para construir este monorepo**: `"engines": { "node": ">=24" }` e `pnpm@12.14.0`
  (`ref/tiptap/package.json:84-87`). A máquina usada neste dossiê tem Node 22.23.2 e **não** tem
  `node_modules` no `ref/` — build/testes do monorepo **não executados** (ver `exemplos/README.md`).
- Frameworks: `@tiptap/react` (React 19 nas devDeps; `use-sync-external-store`, `fast-equals` —
  `packages/react/package.json:53-57`) e `@tiptap/vue-3` (`packages/vue`, nome do diretório `vue`;
  o Vue 2 foi removido — `.changeset/remove-vue-2.md`).

## 6. Maturidade

- 3.30.3, CHANGELOG agregado de 455 KB (`ref/tiptap/CHANGELOG.md`), 90 changesets pendentes
  (`ls .changeset | wc -l`), 325 arquivos de teste unitário/E2E, 61 pacotes.
- Testes reais de comportamento: `packages/core/src/Editor.test.ts` (837 linhas) cobre
  `registerPlugin`, `role="textbox"`, injeção/remoção de CSS, eventos, `contentError`;
  `packages/core/tests/integration/editorLifecycle.test.ts:21-35` prova que um editor **sem
  montar** ainda aceita `setContent` e depois monta com o conteúdo.
- E2E com Playwright sobre 60+ demos, digitando de verdade (`demos/src/Examples/Minimal/index.spec.ts`).
- Sinal de risco: a superfície é enorme (61 pacotes, 100+ extensões citadas em
  `README.md:24`), e mensagens de erro/avisos aparecem no console em vários caminhos normais
  (`resolveExtensions` avisa em nome duplicado; `CommandManager`/`setOptions` avisam em uso
  indevido — `packages/core/src/helpers/resolveExtensions.ts:17-24`, `Editor.ts:305-320`).

## 7. Prós e contras para o `rich-editor-auy`

**Copiar (com adaptação)**
1. **Extensão como única unidade de configuração** (`name`, `priority`, `addOptions`,
   `addCommands`, `addKeyboardShortcuts`, `addInputRules`, `addPasteRules`, `addProseMirrorPlugins`,
   `addNodeView`, hooks de ciclo de vida) — `packages/core/src/Extendable.ts:25-190`.
2. **`commands` + `chain()` + `can()`** com a mesma transação e `preventDispatch`: resolve
   "botão habilitado" e "toolbar que não suja o histórico" de forma barata
   (`CommandManager.ts:29-208`).
3. **`renderHTML`/`parseHTML` por atributo** (`addAttributes` com `renderHTML`/`parseHTML`) —
   dá controle fino de DOM sem tocar em schema (`helpers/getRenderedAttributes.ts:6-23`,
   `helpers/getSchemaSpecByResolvedExtensions.ts:128-150`).
4. **`schemaSpec` externo**: injetar um schema ProseMirror já existente mantendo o comportamento
   local das extensões (`helpers/getSchemaByResolvedExtensions.ts:19-41`) — útil se as publicações
   já têm schema próprio.
5. **`static-renderer`**: renderiza HTML sem instanciar `Editor`, com escaping e lista explícita de
   limitações (`packages/static-renderer/README.md:9-40`) — é o modelo certo para "publicar sem o
   editor carregado".

**Evitar**
1. **Injeção automática de CSS e classe fixa** (`injectCSS:true`, `.tiptap`): nós queremos zero CSS
   por contrato; expor `injectCSS` é dívida que o usuário precisa descobrir.
2. **Estilo inline como default** (`style: 'text-align: …'`, `background-color`): quebra
   "zero CSS" e mistura apresentação no documento. Preferir atributo semântico
   (`data-align`, `data-tone`) e deixar o CSS no front-end.
3. **Superfície de 61 pacotes** e nomes mutáveis (`extension-strike` virou parte de
   `@tiptap/extensions`, `UndoRedo` substituiu `History`, Vue 2 removido) — custo de manutenção e
   de documentação. Nosso alvo é um core pequeno e estável.
4. **`setOptions` parcial e silencioso**: só 19 chaves são runtime
   (`packages/core/src/utilities/getEditorRuntimeOptions.ts:3-23`); o resto só emite `console.warn`
   (`Editor.ts:305-320`). Se copiarmos a ideia, que seja com erro tipado, não aviso.
5. **Duas fontes de verdade para undo** (History vs. UndoRedo/Yjs): a colaboração precisa
   desativar o undo local e avisa em runtime
   (`packages/extension-collaboration/src/collaboration.ts:110-116`). Nosso design deve declarar
   explicitamente quem é dono da pilha.

## 8. Leitura recomendada, em ordem

1. `codigo/core-Editor.createView.excerpt.ts` — como o editor nasce no DOM.
2. `codigo/core-Editor.dispatchTransaction.excerpt.ts` — como uma transação vira eventos.
3. `codigo/core-CommandManager.ts` — como uma intenção de UI vira `tr`.
4. `codigo/core-Extendable.excerpt.ts` — como opções e storage são derivados e compostos.
5. `codigo/extension-bold.bold.tsx` — como uma marca de verdade é declarada.
6. `codigo/core-style.ts` — o único CSS que a biblioteca impõe.
7. `arquitetura.md` e `avaliacao.md` para o restante.
