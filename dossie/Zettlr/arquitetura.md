# Arquitetura do Zettlr (4.9.0)

Base: `ref/Zettlr/source/` (514 arquivos `.ts`/`.vue`). Tudo abaixo cita
`caminho:linha` do `ref/`.

---

## 1. Forma geral: main como servidor, janelas como SPAs

O README oficial descreve o desenho: main process orquestra janelas, cada
`BrowserWindow` é uma SPA Vue e a comunicação é IPC, não `fetch`
(`ref/Zettlr/README.md:229-267`). Na prática:

- Providers singleton em `source/app/service-providers/` (`fsal`, `links`,
  `citeproc`, `documents`, `config`, `css`, `appearance`, `search`, `tags`,
  `targets`, `windows`, ...).
- Cada provider registra `ipcMain.handle('<nome>-provider', ...)` — é a "API".
  Ex.: `links/index.ts:46-59`, `documents/index.ts:270-370`,
  `citeproc/index.ts:196-210`, `fsal/index.ts:95-120`.
- O renderer importa `@common/*` e usa `window.ipc` exposto pelo preload
  (`README.md:262-266`).

Consequência para nós: **não existe biblioteca embutível**. Qualquer reuso é de
ideia/estrutura, não de pacote.

## 2. Modelo de arquivo (FSAL)

### Descritores

O FSAL transforma o sistema de arquivos em descritores serializáveis
(`source/types/common/fsal.ts:84-98`):

```ts
interface MDFileDescriptor extends FSMetaInfo {
  ext: string; id: string; type: 'file'
  tags: string[]; links: string[]; citekeys: string[]
  bom: string; wordCount: number; charCount: number
  firstHeading: string|null; yamlTitle: string|undefined
  frontmatter: any|null; linefeed: string
}
```

Ou seja: **um arquivo é um documento + metadados derivados + relações
(`links`, `citekeys`, `tags`)**. Não há entidade "nota" separada do arquivo;
não há banco de dados. O `.md` é a fonte de verdade.

### Extração de metadados

`fsal/util/file-parser.ts:57-158` faz, em sequência:

| Etapa | Linha | Efeito |
|---|---|---|
| BOM e linefeed | 66-67 | preserva bytes para gravação fiel |
| ID | 68 | `extractFileId(file.name, content, idREPattern)` |
| AST | 71 | `markdownToAST(content)` (Lezer) |
| tags | 73-74 | `ZettelkastenTag.value`, lowercase |
| links | 76-77 | `ZettelkastenLink.target` |
| citekeys | 79-80 | `Citation.parsedCitation.items[].id` |
| primeiro H1 | 82-87 | `firstHeading` (título de exibição) |
| contadores | 91-95 | palavras/caracteres sobre o AST |
| frontmatter | 100-156 | só `title, subtitle, author, date, keywords, tags, lang, bibliography` |

O `FRONTMATTER_VARS` (`file-parser.ts:39-49`) é uma allowlist explícita: o
frontmatter bruto é guardado, mas só esses campos são interpretados. Isso é um
bom padrão: **não** transformar YAML arbitrário em comportamento.

### ID

`fsal/util/extract-file-id.ts:29-57`:

1. tenta o **nome** primeiro (sem âncora `exact`, `:33-36`);
2. senão varre o conteúdo com `matchAll` do regex configurado (`:39-54`);
3. tenta ignorar matches que começam em `[[` (`:47`) ou terminam em `]]`
   (`:51`), para não confundir o ID de um arquivo com o ID citado num link.

Atenção ao segundo guarda: ele testa `match[1].endsWith(']]')`, mas `match[1]` é
o **grupo de captura** (o próprio ID), que nunca termina em `]]`. Ou seja, o
guarda de fechamento é código morto; só o de abertura (`[[`) funciona. Um ID
seguido de `]]` sem `[[` à esquerda ainda seria aceito.

O regex vem de `common/regular-expressions.ts:23-35` (`getIDRE`), que
"embrulha" o padrão num grupo de captura se o usuário não puser nenhum, e com
`exact: true` ancora em `^...$`. Defaults: `idRE: '(\\d{14})'`,
`idGen: '%Y%M%D%h%m%s'`, `linkFormat: 'link|title'`,
`linkWithIDIfPossible: false`, `autoSearch: true`, `customDirectory: ''`
(`config/get-config-template.ts:374-380`).

### Cache

`FSALCache` mantém descritores em shards JSON no `userData/fsal/cache`
(`fsal/fsal-cache.ts:1-40`), com evicção por "acessados"
(`fsal-cache.ts:145-160`). `fsal-file.ts:105-116` só reparseia quando `modtime`
muda e **pula arquivos > 10 MB** (`fsal-file.ts:94-100`). O comentário do
cache declara ganho de 25–90% no boot com ~6.000 arquivos
(`fsal-cache.ts:9-18`).

### Diretórios, projetos e `sorting`

Cada diretório pode ter `.ztr-directory` com
`{ sorting, project, icon, color }` (`fsal/fsal-directory.ts:27-46`); quando os
valores voltam ao default o arquivo é apagado (`:60-77`). Um "projeto" de
publicação é:

```ts
PROJECT_TEMPLATE = {
  title: 'Untitled',
  profiles: [],           // perfis Pandoc a rodar
  files: [],              // lista ORDENADA de arquivos
  cslStyle: '',
  templates: { tex: '', html: '' }
}
```
(`fsal/fsal-directory.ts:40-50`). Verificado no código de export: `config.files`
é mapeado para caminhos absolutos e a ordem é preservada
(`commands/dir-project-export.ts:76-98`). **O "livro" é essa ordem**, não o
grafo de links.

## 3. Editor: CodeMirror 6 + parser Lezer estendido

### Composição

`editor-extension-sets.ts`:

- `getCoreExtensions` (`:152-230`): histórico, seleção múltipla, `drawSelection`,
  busca, indentação configurável, autosave? não — aqui é só editor; inclui
  `hookDocumentAuthority(...)` (`:210-216`) e `configField` (um `Facet` com toda
  a configuração do editor, lido por quase todo plugin).
- `getMarkdownExtensions` (`:270-337`): parser Markdown, highlighter, renderers,
  linters (`spellcheck`, `yamlFrontmatterLint`, opcional `mdLint`,
  `languageTool`), `tocField`, `typewriter`, `readabilityMode`,
  `formattingToolbar`, `footnoteHover`, `filePreview`, `citationTooltips`,
  `tagClasses()` e `clickListeners` (`:316-341`).

### Parser

`parser/markdown-parser.ts` monta `markdownLanguage` + parsers próprios
(citation, footnote, frontmatter, math, pandoc link/table/attributes/div-span,
zkn link/tag, highlight, admonition; `:244-254`) e declara os nós novos com
`defineNodes` (`:257-315`). Note o comentário: mudanças ali **devem** ser
refletidas em `util/custom-tags.ts` e `theme/syntax.ts` (`:256-257`) — acoplamento
manual de três arquivos.

O parser de wikilink (`parser/zkn-link-parser.ts:38-90`) é um `InlineParser`
Lezer que:

- adiciona delimitador em `[[` e devolve `-1` para o parser de link padrão não
  engolir os colchetes internos (`:45-47`);
- em `]]`, consome o conteúdo interno (`ctx.takeContent`), então cria
  `ZknLinkContent`, `ZknLinkPipe` e `ZknLinkTitle` conforme o formato
  configurado (`:61-87`);
- tenta **rejeitar** pipe nas pontas com `pipeIdx > 0 && pipeIdx < contents.length`
  (`:70`), mas só a ponta esquerda é realmente rejeitada. Verificado em
  execução: `[[alvo|]]` gera `ZknLinkTitle` vazio (`value: ""`) e
  `[[|título]]` faz o pipe virar parte do alvo (`target: "|título"`). O
  renderer então esconde conteúdo e pipe, e `[[alvo|]]` fica **invisível**.

Formato configurável: `[[link|title]]` (Obsidian/MediaWiki/VimWiki) ou
`[[title|link]]` (GitHub) — `zkn-link-parser.ts:18-30`.

Âncoras: `[[arquivo#Heading]]` **não** é separada pelo parser — o alvo sai como
`"arquivo#Heading"` (verificado em execução). `retrieveOutbound`/`retrieveInbound`
comparam essa string inteira e não resolvem, embora o clique funcione porque
`force-open` corta no `#`. É inconsistência entre clique e índice.

### Renderers (o "semi-WYSIWYG")

Cada renderer é um `ViewPlugin` que emite `Decoration`s; `base-renderer.ts` é o
helper compartilhado (`renderInlineWidgets`/`renderBlockWidgets`). O de links
(`renderers/render-links.ts:21-88`) esconde marcas via
`Decoration.replace({})` **somente** quando a seleção não está na faixa:

```ts
if (rangeInSelection(view.state.selection, node.from, node.to, includeAdjacent)) return false
```
(`render-links.ts:36-38`)

Para `ZknLink` esconde o *conteúdo* e o pipe, mantendo o título visível
(`:40-51`). Para link Markdown exige ≥3 `LinkMark` ou um `LinkLabel`, e não
esconde se o título estiver vazio (`:52-72`). Esse tipo de guarda é exatamente o
que evita "texto desaparecendo" quando o cursor entra. O gatilho usa
`previewModeShowSyntaxWhenCursorIsAdjacent`, com default `true` (`:25`).

Citações são widgets: `renderers/render-citations.ts:36-83` chama
`window.getCitationCallback(library)` (ponte IPC para o citeproc no main) e, em
falha, cai para o texto cru com `class="error"` (`:72-75`). Crossrefs
(`fig:`, `tbl:`, `eq:`, `sec:`) são tratados localmente, sem citeproc
(`:37-56`).

### Seleção, teclado, undo, clipboard, IME

- `Mod-z`/`Mod-Shift-z` mapeados explicitamente para `undo`/`redo`, com
  `undoSelection`/`redoSelection` em atalhos configuráveis
  (`keymaps/default.ts:274-277`).
- `Mod-b`/`Mod-i`/`Mod-t` aplicam negrito/itálico/task list (`:264-266`).
- `Mod-Shift-v` cola como texto puro; `Mod-Alt-c` copia como HTML
  (`:238-239`) — implementação em `util/copy-paste-cut.ts`.
- IME: nada customizado. O comportamento vem do CodeMirror 6, que trata
  `compositionstart/update/end` internamente; não encontrei código do projeto
  interceptando composição (`grep -ri "composition"` em `source/` não retorna
  handlers próprios). **Não verificado em execução** (app não foi aberto).
- `EditorState.allowMultipleSelections.of(true)` (`editor-extension-sets.ts:185`).

### Documento e sincronização

A autoridade vive no main (`documents/index.ts`):

- `getDocument` cria um `Document` em memória com
  `{ currentVersion, minimumVersion, updates[], document: Text, saveTimeout }`
  (`:619-664`).
- `pushUpdates` aplica `ChangeSet.fromJSON` na cópia canônica
  (`:690-760`); se a versão do cliente divergir, retorna `false`
  (`:696-698`). Há rollback quando a versão chega perto de
  `Number.MAX_SAFE_INTEGER` (`:728-733`) e descarte de histórico acima de
  `MAX_VERSION_HISTORY` (`:739-743`).
- `pullUpdates` devolve `false` quando o cliente ficou fora da janela de versões
  (`:667-689`), o que dispara `reloadStateEffect` no renderer
  (`plugins/remote-doc.ts:130-135`).
- No renderer, `hookDocumentAuthority` (`remote-doc.ts:54-164`) usa
  `@codemirror/collab` (`collab({ startVersion })`, `:163`) e faz *long-poll*: cada
  `pull()` agenda outro `pull()`; cada `docChanged` agenda `push()`
  (`:64-160`).
- Autosave: `off | delayed | immediate`, via `setTimeout` no provider
  (`documents/index.ts:745-758`).
- Gravação preserva BOM e linefeed do arquivo
  (`documents/index.ts:1582`), e o comentário explica a ordem
  "atualizar descritor antes de gravar" para evitar corrida (`:1544-1557`).

Isso é o mesmo problema que o nosso núcleo Rust precisa resolver, mas o Zettlr
resolve com **OT (collab) e um único processo autoritativo**, não com CRDT.
Não há colaboração em rede: o `@codemirror/collab` é usado só para
renderer↔main.

## 4. Links: índice, resolução, renomeação, grafo

### Índice

`LinkProvider` (`links/index.ts`) mantém dois `Map`:

```
_fileLinkDatabase: Map<absolutePath, string[]>   // links de saída
_idDatabase:       Map<absolutePath, string>     // ID (pode ser '')
```
(`links/index.ts:32-33`, inicializados em `:41-42`)

Reindexação **total** em qualquer evento do FSAL (`:63-71`, `:80-86`). Um TODO
no código admite que isso pode ser incrementado (`:64-66`). Também há TODO para
detectar **IDs duplicados** e avisar o usuário (`:43-44`) — ou seja, hoje não
avisa.

### Resolução

`findExact` (`links/index.ts:102-124`) — duplicado de
`fsal/index.ts:418-444` para evitar `getAllLoadedDescriptors()` por chamada
(comentário em `:96-101`):

```ts
if (isQueryID && descriptor.id === query) return descriptor
if (hasMdExt && descriptor.name === query) return descriptor
if (descriptor.name === query + descriptor.ext) return descriptor
```

`retrieveInbound(path)` (`:131-158`) procura, em todos os links de saída, o ID
do arquivo ou seu basename com/sem extensão. `retrieveOutbound(path)`
(`:160-178`) resolve cada string e **descarta silenciosamente** o que não casa.

### Clique

`clickListeners` (`plugins/click-listeners.ts`) exige Ctrl/Cmd, resolve o nó na
posição do mouse e chama `onWikiLink(url)`; o editor reemite
`zettelkasten-link` (`markdown-editor/index.ts:344-350`), que `MainEditor.vue`
traduz em comando `force-open` (`MainEditor.vue:509-523`). `force-open`:

```ts
const filename = linkContents.includes('#') ? linkContents.slice(0, linkContents.indexOf('#')) : linkContents
const file = await this._app.fsal.findExact(filename)
if (file !== undefined) await this._app.documents.openFile(...)
else if (await this._app.fsal.isDir(customDirectory)) await this._app.commands.run('file-new', ...)
```
(`commands/force-open.ts:50-61`)

O `#` é aceito e **ignorado** (não navega para heading). Para links Markdown
com `#âncora`, quem trata é `util/open-markdown-link.ts:44-60`, que busca no
`tocField` e move a seleção.

### Renomeação

`file-rename.ts:133-183`: coleta `retrieveInbound` **antes** de renomear
(`:136`), pergunta ao usuário ("Update N internal links?", `:157-166`), e para
cada arquivo de entrada roda `replaceLinks(content, oldName, newName)`
(`:173-180`).

`replace-links.ts:35-75` parseia o Markdown, extrai `ZettelkastenLink` e
substitui `link.targetRange` quando `target === oldTarget` ou
`target === oldTargetNoExt`; aplica em ordem reversa para não deslocar offsets
(`:65-72`). Limitações explícitas no cabeçalho (`:23-28`) e cobertas por teste
(`ref/Zettlr/test/replace-links.spec.ts:45-46` declara que o link
`[should not work]([[Zettelkasten]])` fica intacto) e por execução
(`exemplos/replace-links.ts`): não mexe em `[texto]([[alvo]])`, não altera code
spans, e não reescreve links que usam ID (pois o ID não muda com o nome —
comportamento correto).

### Grafo e arquivos relacionados

- `win-main/sidebar/RelatedFilesTab.vue:147-196` combina `inbound` (primeiro) +
  `outbound` + arquivos que compartilham tags, marcando
  `bidirectional|inbound|outbound`.
- `win-stats/GraphView.vue:501` consome `get-link-database` e monta um grafo
  dirigido com d3 (`DirectedGraph`), incluindo isolates e filtro por componente.
- Nenhum dos dois influencia o modelo; são visualizações.

### Tags

`TagProvider` reconstrói `TagRecord[]` com `files[]` e
`idf = log(N / files.length)` (`service-providers/tags/index.ts:190-206`), onde
`N` é a **soma de ocorrências** de todas as tags, não o número de documentos
(`:199-201`). O IDF alimenta recomendações na sidebar, não o editor.

## 5. Citações e bibliografia

- Parser: `parser/citation-parser.ts` implementa a sintaxe Pandoc de citação
  (prefixo, `-@`/`+@`, locator com rótulos em en/de/fr, sufixo, múltiplos itens
  separados por `;`). `nodeToCiteItem` (`:229-310`) converte nós em
  `CiteItem { id, locator, prefix, suffix, label, 'suppress-author' }`.
  A tabela `locatorLabels` (`:46-80`) é notável: aceita `p.`/`pp.`/`S.`/`§` etc.
  e mapeia para o termo CSL.
- Banco: `getBibliographyForDescriptor` (`common/util/get-bibliography-for-descriptor.ts:27-55`)
  escolhe `frontmatter.bibliography` (primeiro item se array) ou a constante
  `CITEPROC_MAIN_DB`. No renderer, o caminho relativo é resolvido contra o
  diretório do arquivo (`win-main/MainEditor.vue:65-72`).
- Render: citeproc roda no **main** (`service-providers/citeproc/index.ts`), com
  `engine: CSL.Engine`, `_items: Record<string, CSLItem>` (`:78-92`) e watcher de
  arquivos de bibliografia (`:94-100`). A API exposta é `getCitation(...)`
  (`:513`), `makeBibliography(...)` (`:563`) e checagens de citekey
  (`:603-626`).
- Autocomplete ordena chaves por uso no documento atual
  (`autocomplete/citations.ts:47-90`) — detalhe de UX que vale copiar.
- Export: `citeproc` como filtro Pandoc (`static/defaults/HTML.yaml:15`) +
  bibliografia do projeto injetada nos defaults
  (`commands/exporter/index.ts:218-238`).

## 6. Exportação

Dois caminhos:

1. **Preview/impressão** — `win-print/App.vue:66-110` chama `md2html` com
   `onCitation`, `onBibliography` e `onImageSrc` (rebase de imagens para
   `safe-file://`). É o mesmo renderizador do AST do editor, logo o preview
   reflete o que o usuário vê.
2. **Pandoc** — `commands/exporter/index.ts`. `writeDefaults` (`:164-280`):
   carrega o perfil YAML do usuário, **habilita a extensão de wikilinks**
   conforme `zkn.linkFormat` (`:187-196`), habilita `mark`/`alerts` (`:200-215`),
   injeta bibliografia/CSL (`:218-237`), grava `metadata.zettlr.strip_links` e
   `strip_tags` (`:248-249`) e concatena filtros Lua do usuário (`:260-264`).
   O filtro embutido `static/lua-filter/links.lua:44-58` decide entre `unlink`
   (devolve o conteúdo) e `full` (remove) para `Link` com classe `wikilink`, e
   pode ser controlado por frontmatter `zettlr.strip_links` (`:21-26`).

Projetos: `dir-project-export.ts` valida que os arquivos da lista ainda existem,
avisa se faltam mas **exporta mesmo assim** (`:90-97`), e roda um exporter por
perfil (`:104-120`).

### O que o `md2html` faz com `[[...]]`

`markdown-to-html.ts:365-371`:

```ts
} else if (node.type === 'ZettelkastenLink') {
  return `${node.whitespaceBefore}[[${node.title?.value ?? node.target}]]`
}
```

Ou seja: no HTML gerado pelo **próprio Zettlr**, um wikilink **não vira `<a>`**,
vira texto literal `[[título]]` (ou `[[alvo]]`). Só o caminho Pandoc converte
em link — e lá o filtro decide se mantém, deslinka ou remove. Meu exemplo
`md2html.ts` confirma a saída literal.

## 7. Temas e CSS

Três camadas:

1. **App**: `assets/main.css` + classes `.dark` no `body`
   (`window-register/register-themes.ts:60-66`); custom.css do usuário é um
   `<link>` para `safe-file://.../custom.css` recriado a cada mudança
   (`:72-98`); `setSystemCss` injeta `--system-accent-color`
   (`:100-127`).
2. **Editor**: `EditorView.theme` com ~30 variáveis `--zettlr-editor-*`
   (`theme/editor.ts:25-110`), `EditorView.baseTheme` mapeando essas variáveis
   para elementos (`.cm-scroller`, seleção, etc., `:210+`) e 5 presets
   (`berlin`, `bielefeld`, `bordeaux`, `frankfurt`, `karl-marx-stadt` —
   `theme/themes/*.ts`, cada um só sobrescreve variáveis; ver
   `themes/berlin.ts:26-45`).
3. **Markdown**: `customTags` (`util/custom-tags.ts:22-50`) define `Tag`s Lezer
   e `theme/syntax.ts` as mapeia para cores.

Para nós: o padrão "variáveis CSS declaradas em TS, presets só sobrescrevem
variáveis" é diretamente aproveitável — e é o oposto de "zero CSS" que
queremos no núcleo: aqui o estilo é responsabilidade do renderer.

## 8. Acessibilidade

Verificado no código, **não** em leitor de tela:

- Regiões nomeadas: editor (`win-main/MainEditor.vue:3-7`), árvore de arquivos
  (`file-manager/FileTree.vue:5`), lista (`FileList.vue:7`), visualizadores
  (`file-viewers/PDFViewer.vue:6`), tab bar (`common/vue/TabBar.vue:7`).
- `aria-labelledby` nas telas com abas (`win-about/App.vue:19`,
  `win-assets/App.vue:19`).
- `role="region"` em containers principais; 100 ocorrências de `aria-*`/`role=`
  em `source/` (34 atributos `aria-*` + 66 `role=`; contagem por
  `grep -rho`).
- **Não há** `aria-live`/região de anúncios no projeto (`grep aria-live` → 0).
  O anúncio de mudanças fica por conta do CodeMirror 6, que já implementa
  `aria-live` interno e descrições de seleção; não há customização.
- Foco: CodeMirror 6 cuida do foco do conteúdo editável; os diálogos nativos
  são responsabilidade do Electron/Vue. Não encontrei focus trap próprio.
- Contraste: cores vêm de variáveis (`--grey-*`, `--red-2`) e dos presets; não
  há checagem automatizada de contraste no repositório.

## 9. Performance e limites declarados

- Cache de metadados em disco com ganho declarado de 25–90% no boot
  (`fsal/fsal-cache.ts:9-18`).
- Limite duro: arquivos > 10 MB não são parseados (`fsal/fsal-file.ts:94-100`).
- Índice de links reconstruído integralmente por evento (`links/index.ts:63-71`).
- `findExact` do FSAL é O(n) sobre todos os descritores
  (`fsal/index.ts:418-444`); o LinkProvider duplica a lógica para evitar a
  chamada, mas continua O(n) (`links/index.ts:102-124`).
- Histórico OT limitado por `MAX_VERSION_HISTORY`, com rollback de versão
  (`documents/index.ts:730-753`).
- Busca: `SearchProvider` percorre arquivos com fila e query booleana compilada
  (`service-providers/search/index.ts:30-70`); não há índice invertido.

## 10. Diagnóstico e erros

- Frontmatter inválido é silenciosamente ignorado (`file-parser.ts:137-140`,
  `catch` vazio com comentário).
- Citação não resolvida → widget com `class="error"` e texto cru
  (`render-citations.ts:68-73`).
- Falha ao aplicar `ChangeSet` → `dialog.showErrorBox` com instrução de copiar o
  texto e reiniciar (`documents/index.ts:712-719`).
- Link quebrado → **nenhum diagnóstico**.
- IDs duplicados → TODO não implementado (`links/index.ts:43-44`).
