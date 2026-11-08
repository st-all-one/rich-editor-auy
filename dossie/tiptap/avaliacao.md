# Avaliação do Tiptap para o `rich-editor-auy`

Escala 0–5. Toda nota tem evidência (`caminho:linha`, teste, demo ou execução). Onde não houve
execução, está escrito **não verificado**.

Método: leitura de ~40 pontos do código (core, extensões, React, Vue, Yjs, static-renderer),
3 arquivos de teste, 5 demos, e **uma execução real** em Chromium headless com Tiptap 3 via CDN
(comandos e saída em `exemplos/README.md`).

## 1. Tabela de notas

| Dimensão | Nota | Evidência | Comentário |
|---|---|---|---|
| **Usabilidade** | 4/5 | Init em 6 linhas (`demos/src/Examples/Minimal/React/index.jsx:1-23`); comandos e consultas observados em execução (`toggleBold` → `<p><strong>hello</strong></p>`, `can().undo()` → `true`) | O caminho "primeira edição" é excelente. A partir daí o usuário precisa construir toda a UI e descobrir sozinho `isActive`, `can()`, `useEditorState`; não há componentes prontos. |
| **API** | 4/5 | `commands`/`chain()`/`can()` consistentes (`core/src/CommandManager.ts:29-152`); augmentação de tipos por módulo (`extension-bold/src/bold.tsx:13-30`) | Coerente e tipada, mas com armadilhas: `setOptions` só aceita 19 chaves e avisa em vez de falhar (`core/src/Editor.ts:305-320`, `utilities/getEditorRuntimeOptions.ts:3-23`); `chain().run()` devolve o **AND** dos comandos e pode ser `false` com efeito já aplicado — observado `chain().selectAll().toggleBold().undo().run()` ⇒ `false` com `<strong>` no documento. |
| **Neutralidade de estilo** | 4/5 | Zero classes padrão em nodes/marks (só `language-*` em `extension-code-block/src/code-block.ts:153-156`, confirmado: `<pre><code class="language-js">`); CSS injetado é estrutural, 1329 chars (`codigo/core-style.ts`); `injectCSS:true` é default (`core/src/Editor.ts:100`) e a classe `tiptap` é sempre prependada (`Editor.ts:697-699`). Opt-out **verificado em documento isolado**: `injectCSS:false` ⇒ 0 elementos `style`/`link` e editor funcional | Melhor que qualquer outro editor avaliado: nada de tema, toolbar ou `class` obrigatória, e o default é desligável. Quatro ressalvas para a nossa meta zero CSS: (a) injeta ~1,3 KB por default (desligável), (b) adiciona `.tiptap` — **não desligável**, e a `class` do usuário vem depois (`tiptap ProseMirror my-wrap`), (c) algumas extensões **renderizam estilo inline** (`text-align` — observado `<p style="text-align: center;">` — e `background-color`), (d) "zero estilo" exige reescrever esses `renderHTML`. |
| **Modelo de dados** | 5/5 | Documento = `Node` do ProseMirror; schema derivado de extensões (`helpers/getSchemaSpecByResolvedExtensions.ts:60-239`); JSON é `doc.toJSON()` (`Editor.ts:792-800`); interop com schema externo (`getSchemaByResolvedExtensions.ts:19-41`) | Não inventa formato. JSON estável, schema declarativo, e ainda permite reaproveitar schema já existente — exatamente o que precisamos para não criar um segundo modelo de documento. |
| **Extensibilidade** | 5/5 | `Extendable` com `options`/`storage` como getters e cadeia `parent`/`child` (`codigo/core-Extendable.excerpt.ts`); campo de config pode ser função com contexto (`helpers/getExtensionField.ts:12-31`); `addExtensions()` para kits (`starter-kit/src/starter-kit.ts:213-273`) | É a melhor parte do projeto. Uma extensão pode contribuir schema, DOM, comandos, teclas, input/paste rules, plugins, node/mark views e hooks de ciclo de vida — tudo num objeto. |
| **Performance** | 3/5 | `CommandManager.commands` aloca um objeto completo por acesso (`CommandManager.ts:34-56`); `options`/`storage` recriam objetos (`Extendable.ts:615-633`); React só evita re-render com `shouldRerenderOnTransaction:false` + `useEditorState` (`react/src/useEditor.ts:322-341`) | Nada patológico observado, mas **não há benchmark** no repo e o demo `Performance` é instrucional. Custo dominante é do ProseMirror/`contenteditable`. **Não medido** (não verificado). |
| **Acessibilidade** | 3/5 | `role="textbox"` garantido (`Editor.ts:610-630`, testado em `Editor.test.ts:198-271`); `tabindex` automático só quando editável (`extensions/tabindex.ts:34-58`); demo dedicado com `aria-label`/`aria-multiline`/`Alt+F10` (`demos/src/Examples/Accessibility/React/index.tsx:11-28`) | Base correta (foco, tabindex, role), mas `aria-multiline` veio `null` na execução; não há anúncio de mudança de formatação, nem gestão de foco em menus, nem live regions. Toda a a11y "de produto" fica com o integrador. |
| **Documentação** | 3/5 | No repo só há `docs/package-documentation.md`, `docs/package-references.md` e READMEs de pacote quase idênticos (ex.: `packages/core/README.md:5-17`); a documentação real está em tiptap.dev (link, não conteúdo) | Para estudar o projeto, os **demos e testes** são a documentação de fato (`demos/src/Examples/*`, 325 arquivos de teste). Isso encarece a integração sem rede. |
| **Manutenção** | 3/5 | 61 pacotes, 90 changesets pendentes, CHANGELOG de 455 KB, versão declarada 3.30.3 (checkout em `main`, sem tag); renomeações recentes (`undoRedo` no lugar de `history` — `starter-kit/src/starter-kit.ts:43-46`, `extension-list` agrupando listas, Vue 2 removido em `.changeset/remove-vue-2.md`) | Projeto sério e testado, mas com superfície e churn grandes. Copiar o *modelo* é barato; acompanhar o *ecossistema* não é. |
| **Adequação ao nosso caso** | 3/5 | Ver §4 | Copiar: extensão como unidade, `commands`/`can`, schema derivado, `renderHTML`/`parseHTML` por atributo, static-renderer. Evitar: CSS injetado, classe fixa, estilo inline como default, 61 pacotes, `onCreate` assíncrono. |

Média simples: **3,7/5**. Para o nosso problema específico (Rust headless + zero CSS + referências
entre publicações), o valor está no **modelo de extensão e de comando**, não no pacote.

## 2. Matriz de usabilidade (meta ≥85%)

Status possíveis: `código`, `teste`, `exemplo`, `doc`, `execução` (observado por mim) e
`não verificado`.

| # | Item | Status | Evidência |
|---|---|---|---|
| 1 | Instalação e pré-requisitos | **código + doc** | `packages/core/package.json` (ESM/CJS/tipos, `deps: @tiptap/pm`, `prosemirror-model`); monorepo exige Node ≥24 e pnpm 12.14.0 (`ref/tiptap/package.json:84-87`) — a máquina do dossiê tem Node 22.23.2, logo **build do monorepo não executado**. Consumo via CDN esm.sh funcionou (execução). |
| 2 | Inicialização mínima | **execução + exemplo** | `new Editor({element, extensions:[StarterKit], content})` criou `div`, `contenteditable=true`, `role="textbox"`, `tabindex="0"`, 1 style tag (1329 chars), className `tiptap ProseMirror` (com `editorProps.attributes.class = 'my-wrap'`: `tiptap ProseMirror my-wrap`). Saída em `exemplos/README.md`. |
| 3 | Edição básica (texto, parágrafo, negrito/itálico/link) | **execução** | Formatação por comando: `selectAll`+`toggleBold` ⇒ `<p><strong>hello</strong></p>`. Digitação real em Chromium (`page.keyboard`): `**negrito** ` ⇒ `<strong>negrito</strong>`, `Enter` cria parágrafo (`exemplos/README.md` §3). E2E do repo também cobre (`demos/src/Examples/Minimal/index.spec.ts`). |
| 4 | Seleção, teclado, undo/redo, clipboard, IME | **execução (undo/redo/teclas) + código (clipboard/IME)** | Undo/redo observados (reverteu o `Enter` e o `<strong>`; `redo` reaplicou). Input rule por teclado observada. `Ctrl+A` ⇒ `isActive('bold')===false` em doc misto. Teclas declaradas por extensão (`bold.tsx:131-136`) + fallbacks (`extensions/keymap.ts:19-136`). Clipboard: só código (`clipboardTextSerializer.ts:25-53`, `paste.ts:6-25`) — **colar não verificado**. IME: só código (`InputRule.ts:324-338`, `view.composing`) — **não verificado em execução**. |
| 5 | Schema e restrições de conteúdo | **execução + código** | Observado: `<p>ok</p><table>…</table>` entra como `<p>ok</p><p>x</p>` (nó desconhecido descartado, texto preservado), sem erro porque `enableContentCheck:false` é default (`Editor.ts:112`). Código: `content`/`marks`/`group` viram `NodeSpec` (`getSchemaSpecByResolvedExtensions.ts:93-126`); `node.check()` só com `errorOnInvalidContent` (`createNodeFromContent.ts:51-53`). |
| 6 | Custom node/mark/plugin | **execução + código** | Nó `callout` criado com `Node.create` + `addAttributes`/`parseHTML`/`renderHTML`/`addCommands` renderizou `<div data-callout data-tone="warn">cuidado</div>` e voltou no `getJSON()` com `attrs.tone` — round-trip completo, sem plugin (`exemplos/cdn/probe.html`, saída em `exemplos/README.md` §3). Mark completo real em `codigo/extension-bold.bold.tsx`. Plugins crus via `addProseMirrorPlugins` (`ExtensionManager.ts:186-200`) — lidos, não executados. |
| 7 | Persistência e serialização | **execução + código** | `getHTML()` ⇒ `<p><strong>hello</strong></p>`; `getJSON()` ⇒ `{"type":"doc",…}`; `getText()` ⇒ `"ok\n\nx"`; JSON do `callout` preserva `attrs`. Caminhos: `DOMSerializer` + documento temporário (`helpers/getHTMLFromFragment.ts:4-12`); `@tiptap/html/server` para Node (`packages/html/src/generateHTML.ts:20-25`). |
| 8 | Renderização e não interferência no CSS | **execução (isolada)** | Default: 1 `<style data-tiptap-style>` (1329 chars, = conteúdo da constante `style`), classe `tiptap ProseMirror`, **nenhum** `style=` inline na raiz. Opt-out re-verificado em **documento isolado** (iframe próprio, um único editor, `@tiptap/core@3.30.3` fixado): `injectCSS:false` ⇒ **0** `style`/`link` no head e editor funcional (`selectAll`+`toggleBold` ⇒ `<p><strong>sem css</strong></p>`). A prova anterior (2º editor na mesma página, `styleTagsBefore===styleTagsAfter===1`) era **inválida**: `createStyleTag` reaproveita a tag existente (`createStyleTag.ts:2-8`), logo dois editores com CSS também dão 1. A classe `tiptap` permanece mesmo com `injectCSS:false`; `editorProps.attributes` aplica `aria-label` e `class` do usuário **depois** de `tiptap ProseMirror` (observado `tiptap ProseMirror my-wrap`). |
| 9 | Acessibilidade (ARIA, foco, leitor de tela, contenteditable) | **execução + código + doc** | Observado: `role="textbox"`, `tabindex="0"`, `aria-multiline` **ausente**; `setEditable(false)` ⇒ `contenteditable="false"` e `tabindex` removido. Demo dedicado com `role="application"`, `aria-label`, `Alt+F10` (`Accessibility/React/index.tsx:11-28`). Leitor de tela real **não testado**. |
| 10 | Colaboração | **código** | `Collaboration` instala `ySyncPlugin`/`yUndoPlugin`, assumindo o undo (`collaboration.ts:90-240`), avisa se `undoRedo` coexistir (110-116), trata IME/input rules com `stopCapturing` (241-249) e tem caminho de `contentError` (221-283). Conversões em `packages/y-tiptap/src/index.ts:16-26`. **Não executado** (exigiria Y.Doc + provider). |
| 11 | Performance e limites conhecidos | **código (sem números)** | Alocações por acesso em `CommandManager.ts:34-56` e `Extendable.ts:615-633`; React sem re-render por transação por default (`react/src/useEditor.ts:322-341`). Nenhum benchmark no repo; **performance não medida**. |
| 12 | Erros, migração e diagnóstico | **código + execução** | `contentError` com `disableCollaboration()` (`Editor.ts:520-570`), `enableContentCheck` opcional, `console.warn` para JSON inválido (`createNodeFromContent.ts:56-64`), nome duplicado (`resolveExtensions.ts:17-24`), opção não-runtime (`Editor.ts:305-320`), aviso de leitura de `editor.state` dentro de decoration `create()` (`Editor.ts:398-415`). Migração: changesets e `codemods/tiptap-v4-migration/`. |
| 13 | Licença e maturidade | **doc + código** | MIT (`LICENSE.md:1-3`, `packages/core/package.json:16`), 3.30.3, 61 pacotes, 325 arquivos de teste, 90 changesets, CI GitHub Actions (`ref/tiptap/.github/workflows`). |

**Cobertura da matriz: 13/13 itens com evidência** (100% no critério "tem caminho de código, teste,
demo ou execução"). Executados de fato: inicialização, edição por teclado, input rule, comandos,
`isActive`/`can`, undo/redo, nó customizado, serialização, CSS default e **opt-out em documento
isolado**, atributos do usuário, `setEditable` e schema. **Sem verificação por execução** (lacunas
explícitas): colagem, IME real, `Mod-b`, colaboração Yjs e qualquer número de performance.

Nota sobre o alcance da execução: todas as medições usaram o **pacote publicado** via esm.sh
(`@tiptap/core@3.30.3`, fixado nesta revisão), não o monorepo construído. O comportamento de runtime
confirma a leitura do código em quase tudo; a única exceção encontrada foi `isDestroyed` (§5, item 4).

## 3. O que o usuário realmente experiencia

1. **Primeiros 5 minutos**: `npm i` + 10 linhas → editor funcionando, texto editável, sem estilo.
   Sensação de "isto é só uma caixa branca" — é o esperado para headless.
2. **Primeira toolbar**: descobre `editor.commands`, depois `editor.isActive`, depois
   `editor.can()`; o padrão `chain().focus().toggleBold().run()` é o idioma da biblioteca
   (`demos/src/Examples/Default/React/MenuBar.tsx:27-40`).
3. **Primeiro problema real**: botão que não atualiza. Descobre que React **não** re-renderiza por
   transação e precisa de `useEditorState` com `selector` (`react/src/useEditorState.ts:13-30`) —
   é o principal "penhasco" de aprendizado.
4. **Segundo problema**: quer mudar extensões em runtime e recebe um `console.warn`
   (`Editor.ts:305-320`) — precisa recriar o editor e remedir o estado.
5. **Terceiro problema**: conteúdo antigo com nós que o schema não conhece some silenciosamente;
   a solução é `enableContentCheck: true` + `onContentError` — que **existe** e é bem pensado
   (`Editor.ts:520-570`), mas não é o default.
6. **Adoção de a11y**: descobre que só ganhou `role=textbox` e `tabindex`; menu, atalhos de menu e
   anúncios são todos seus (demo `Accessibility` é o guia).
7. **Persistência**: `getJSON()` é o caminho; `getHTML()` é para exibição. Ambos prontos.

## 4. Comparação explícita

### 4.1 Contra ProseMirror puro

| Critério | ProseMirror | Tiptap | Ganho do Tiptap |
|---|---|---|---|
| Começar | montar `Schema` à mão, `EditorState`, `EditorView`, keymaps | `new Editor({extensions})` | enorme |
| Adicionar negrito | `MarkSpec` + regra de parse + comando + keymap, em arquivos separados | 1 objeto de extensão (`bold.tsx`) | enorme |
| Toolbar | você calcula `isActive` com `state.doc.rangeHasMark` | `editor.isActive('bold')` | grande |
| "Pode aplicar?" | comando `(state, dispatch)` com `dispatch=null` na mão | `editor.can().chain()…run()` | grande |
| Controle total do DOM | total | total, via `renderHTML`/node views | empate |
| CSS | zero por padrão (`prosemirror.css` é opcional) | injeta CSS estrutural por padrão | **ProseMirror ganha** |
| Sem DOM / render off-line | você escreve `DOMSerializer` | `static-renderer` / `@tiptap/html/server` | Tiptap |
| Peso/conceitos | 1 biblioteca | 61 pacotes + camada de extensões | ProseMirror ganha em simplicidade |

### 4.2 Contra nossa meta de zero CSS

| Exigência nossa | Tiptap | Veredito |
|---|---|---|
| Nenhuma folha de estilo global | `injectCSS:true` injeta ~1329 chars em `<style data-tiptap-style>` (`Editor.ts:100,293-297`) | **divergente por default**, corrigível com `injectCSS:false` |
| Nenhuma classe obrigatória | sempre adiciona `tiptap` ao lado de `ProseMirror` (`Editor.ts:697-699`) | **divergente**; para nós seria melhor não haver classe imposta |
| Nenhum estilo inline no conteúdo | `text-align`, `background-color`, code block `language-*` | **divergente**: o estilo inline é como as extensões "funcionam sem CSS" |
| Estilo vem do front-end (TS/PHP) | possível (`HTMLAttributes`, `editorProps.attributes`) | **compatível** |
| Sem dependência de tema | nenhum tema, nenhum ícone, nenhum menu | **compatível** |
| Publicação sem editor rodando | `static-renderer` sem Editor | **compatível**, com ressalva de plugins não rodarem |

Conclusão: a arquitetura do Tiptap **permite** zero CSS, mas não o pratica. O que vale copiar é o
mecanismo (`renderHTML` + atributos + `mergeAttributes`), invertendo a política: atributo
semântico no documento, CSS só no front-end — e nada injetado no `<head>`.

### 4.3 Contra o nosso problema de referências entre publicações

Nada no núcleo resolve referência entre documentos. O que existe e serve de matéria-prima:

- `addAttributes` + `renderHTML`/`parseHTML` para `data-ref-id`/`href` (padrão de `Link`,
  `extension-link/src/link.ts:240-330`).
- `UniqueID` (`packages/extension-unique-id`) gera `data-id` **via plugin** — e o static-renderer
  admite que isso não roda sozinho (`static-renderer/README.md:16-31`), exigindo
  `generateUniqueIds()` antes de renderizar.
- Sugestão (`@tiptap/suggestion`) e `Mention` existem, mas são acoplados a UI de popup
  (posicionamento próprio) — para nós, o valor é só o formato de sugestão, não o componente.

Implicação de design: se copiarmos o Tiptap, a referência entre publicações deve ser um
**atributo de schema + comando de inserção**, com resolução feita **fora** do editor (nosso
front-end PHP/TS), para que render estático e editor concordem sem plugin.

## 5. Riscos e o que não foi verificado

1. **Não executei** build, lint nem testes do monorepo: `ref/tiptap` não tem `node_modules`, e o
   `package.json` exige Node ≥24 (a máquina tem 22.23.2). Nenhuma conclusão aqui depende disso.
2. **Não verifiquei** IME real (chinês/japonês/árabe/teclado virtual), colagem, arrastar-e-soltar,
   colaboração Yjs, mobile iOS/Android e leitor de tela. Digitação real, input rule, nó customizado,
   undo/redo, `setEditable`, `injectCSS:false` e `editorProps.attributes` **foram** verificados
   (`exemplos/README.md` §3).
3. **Não medi** performance; as observações são estruturais (alocações), não temporais.
4. **Identidade de versão — corrigido nesta revisão.** O checkout de `ref/tiptap` é `main` no
   commit `5fd5d2f` (2026-11-05), **não a tag 3.30.3**: `packages/*/package.json` ainda diz 3.30.3
   porque há 90 changesets não publicados. A "divergência 3.30.3 vs 3.33.0" que o dossiê relatava
   está mal atribuída. Rodei o mesmo teste com as duas versões **publicadas** (`@tiptap/core@3.30.3`
   e `@3` = 3.33.0) e ambas dão `isDestroyed === true` após `unmount()`; só o **código de HEAD**
   tem `get isDestroyed() { return this.destroyed }` (`Editor.ts:901-903`), que daria `false`.
   Ou seja, a diferença é **HEAD vs. artefato publicado**, não entre duas releases. Consequência
   prática: citações `arquivo:linha` e observações de runtime descrevem builds diferentes — mais um
   motivo para copiarmos o modelo, não a superfície. (Também por isso todas as execuções desta
   revisão fixaram `@3.30.3` quando o objetivo era casar com o checkout.)
5. Não avaliei extensões Pro (pagos): comentários, versionamento, conversão, IA — não estão no repo
   (`README.md:37-38`).

## 6. Veredito

**É a melhor referência de DX de extensão que este repositório de estudos encontrou — e não é zero
estilo por default.** As duas coisas são verdadeiras ao mesmo tempo e não se anulam:

1. **Melhor DX de extensão.** Uma única unidade (`Extension.create` / `Node.create` /
   `Mark.create`) contribui schema, `parseHTML`, `renderHTML`, atributos tipados, comandos,
   atalhos, input rules, paste rules, plugins do ProseMirror, node/mark views e hooks de ciclo de
   vida. `commands` + `chain()` + `can()` resolvem o trio "executar / compor / desabilitar" com a
   mesma transação. Nenhum outro projeto avaliado junta isso com tipagem por augmentação de módulo.
   Evidência: `codigo/core-Extendable.excerpt.ts`, `codigo/core-CommandManager.ts`,
   `codigo/extension-bold.bold.tsx` (extensão real, 163 linhas, sem tocar no core) e o nó `callout`
   que criei em ~15 linhas e fez round-trip HTML↔JSON.
2. **Não é zero estilo por default.** `injectCSS: true` é default e injeta 1329 chars de CSS
   estrutural; a classe `tiptap` é prependada e **não** há opção para removê-la; várias extensões
   escrevem `style` inline (`text-align`, `background-color`) porque é assim que "funcionam sem
   CSS". O "zero CSS" existe, mas é **opt-in (`injectCSS:false`), parcial e por extensão**.
3. **Serve como API, não como core.** O valor para o `rich-editor-auy` está no **contrato**
   (extensão como unidade, comandos transacionais, `can` sem despacho, schema derivado,
   `renderHTML`/`parseHTML` por atributo, `mergeAttributes`), não em adotar 61 pacotes, o DOM do
   ProseMirror e a política de estilo deles. Copiar o desenho e inverter os defaults — atributo
   semântico no documento, CSS só no front-end, nada injetado no `<head>`, classe imposta opcional.

Nota final: **3,7/5** na média simples; **4/5** em neutralidade de estilo **apenas** no sentido
"não impõe tema nem UI", caindo para **3/5** se a régua for a nossa ("nenhum CSS, nenhuma classe,
nenhum estilo inline por default").
