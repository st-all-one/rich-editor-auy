# Zettlr — dossiê de referência

**Versão analisada:** `4.9.0` (`ref/Zettlr/package.json:13`).
**Licença:** GNU GPL v3 (`ref/Zettlr/LICENSE:1`; `package.json:14`).
**Veredito de uso:** estudo apenas. Nada de `codigo/` pode ser incorporado ao
`rich-editor-auy` sem contaminar o projeto com GPL-3.0. O valor aqui é
**conceitual**: modelo de arquivo, resolução de links, autoridade de documento.

---

## 1. Para que serve (produto, não biblioteca)

Zettlr é um **aplicativo desktop Electron** para escrita acadêmica em Markdown:
notas Zettelkasten, citações via CSL/citeproc, e exportação por Pandoc. O README
se descreve como "Your One-Stop Publication Workbench"
(`ref/Zettlr/README.md:9`) e lista como recursos centrais: citações integradas,
saída para LaTeX/Word, exportação via Pandoc/LaTeX/Textbundle, temas + CSS
customizado, Zettelkasten e busca full-text (`README.md:69-85`).

Não existe API pública de biblioteca. Tudo é montado em torno de
`BrowserWindow` + IPC; a "API" é o conjunto de canais `ipcMain.handle`
(`source/app/service-providers/links/index.ts:46`,
`source/app/service-providers/documents/index.ts:270-370`). Consumir o Zettlr
"como biblioteca" significaria recortar módulos e reimplementar os providers.

## 2. Fluxo real do usuário

1. **Instalação** — instaladores para macOS/Windows/Linux, Flatpak, Homebrew,
   APT, WinGet, Chocolatey, Arch (`README.md:87-115`). Não requer Pandoc
   instalado (usa o Pandoc empacotado: `useBundledPandoc: true` em
   `source/app/service-providers/config/get-config-template.ts:362`).
2. **Primeira execução** — a janela principal carrega workspaces (pastas raiz);
   o onboarding está em `source/win-onboarding/` e o tutorial em
   `static/tutorial/`.
3. **Primeira edição** — Markdown "semi-WYSIWYG": o texto bruto permanece o
   documento, e *renderers* escondem marcas de sintaxe quando o cursor está
   fora da faixa (`source/common/modules/markdown-editor/renderers/render-links.ts:21-90`).
4. **Links internos** — digitar `[[` dispara autocomplete de arquivos
   (`source/common/modules/markdown-editor/autocomplete/files.ts:108-119`);
   a inserção respeita `zknLinkFormat`, `linkWithIDIfPossible` e
   `zknAddFileTitle` (`autocomplete/files.ts:63-100`). Ctrl/Cmd+clique abre o
   alvo (`plugins/click-listeners.ts:38-70`) e, com `zkn.autoSearch`, também
   dispara busca global (`win-main/MainEditor.vue:519-522`).
5. **Citações** — `@chave` é reconhecido pelo parser inline
   (`parser/citation-parser.ts:229-310`), renderizado como widget via
   citeproc (`renderers/render-citations.ts:36-83`) e completado com as chaves
   da biblioteca do arquivo (`autocomplete/citations.ts:47-90`).
6. **Publicação** — exportação unitária (`commands/export.ts`) ou de projeto
   (lista ordenada de arquivos em `.ztr-directory`,
   `fsal/fsal-directory.ts:37-49`; `commands/dir-project-export.ts:66-98`).
   O preview de impressão renderiza o mesmo Markdown em HTML com citações
   resolvidas (`source/win-print/App.vue:89-104`).

## 3. O que o app ensina sobre "referências entre publicações"

Esta é a parte mais valiosa para o `rich-editor-auy`, e ela é **mais pobre do
que parece**:

- **A referência é textual e frágil por desenho.** `[[alvo]]` guarda uma string;
  não há UUID no link. O índice mapeia `caminho → links[]` e `caminho → id`
  (`links/index.ts:80-86`), reconstruído inteiro a cada evento do FSAL
  (`links/index.ts:63-71`).
- **Três formas de casar um alvo**, avaliadas em ordem: ID (regex do usuário,
  `findExact` em `links/index.ts:102-124`), basename **com** extensão, basename
  **sem** extensão (`descriptor.name === query + descriptor.ext`). Não há
  casamento por título, por caminho relativo nem por alias.
- **Link quebrado não é marcado.** `retrieveOutbound` simplesmente descarta os
  alvos que não resolvem (`links/index.ts:160-178`); meu experimento confirma
  isso (`exemplos/` → `outbound(...)` vazio). O *único* consolo é o autocomplete:
  alvos que aparecem em links mas não existem como arquivo entram na lista de
  completions com `id: ''`, permitindo "criar ao completar"
  (`source/win-main/MainEditor.vue:632-657`).
- **Clique em link inexistente** só cria arquivo se `zkn.customDirectory` estiver
  configurado; caso contrário o clique não faz nada
  (`commands/force-open.ts:50-61`).
- **Âncora em wikilink é inconsistente.** `[[arquivo#Heading]]` é aceito no
  clique (`force-open` corta no `#`), mas o alvo guardado é a string inteira
  `"arquivo#Heading"`, então inbound/outbound não resolvem (verificado em
  execução: `arquitetura.md` §3).
- **Pipe nas pontas não é rejeitado como o comentário promete.**
  `[[alvo|]]` cria título vazio e o renderer esconde alvo e pipe, deixando o
  link **invisível**; `[[|título]]` faz o pipe virar parte do alvo
  (`zkn-link-parser.ts:68-70`, verificado em execução).
- **Renomear propaga**, mas com escopo estreito: pergunta ao usuário e reescreve
  apenas `ZettelkastenLink` (`commands/file-rename.ts:133-183`;
  `common/util/replace-links.ts:35-75`). Links em `[texto]([[alvo]])` e em code
  spans **não** são reescritos — coberto pelo teste do próprio projeto
  (`test/replace-links.spec.ts:45-46`) e verificado em execução real
  (`exemplos/replace-links.ts`).
- **A publicação é uma lista ordenada de arquivos**, não um grafo: o "projeto"
  é um arquivo JSON com `files[]` na ordem de exportação
  (`fsal/fsal-directory.ts:37-49`), e a exportação concatena nessa ordem
  (`commands/dir-project-export.ts:76-98`).
- **Grafo é visualização, não fonte de verdade**: `win-stats/GraphView.vue:501`
  lê o mesmo `get-link-database`; nada no modelo depende dele.

Resumo: Zettlr ensina um modelo **wiki-link simples + índice derivado +
reescrita sob confirmação**. Ensina também os buracos desse modelo: sem
diagnóstico de link quebrado, sem estabilidade de identidade quando o nome muda
e o ID não é usado, sem propagação fora do formato `[[...]]`.

## 4. Arquitetura em uma tela

Electron clássico: **main process = servidor**, cada janela = SPA Vue servida
por IPC (o README oficial descreve exatamente isso, `README.md:229-267`).
Providers singleton no main (`source/app/service-providers/*`), renderer
compartilhando o mesmo preload (`source/common/modules/preload`).

- **Editor:** CodeMirror 6 com parser Lezer estendido (ZknLink, citação,
  frontmatter, admonitions, Pandoc div/span, tabelas) —
  `parser/markdown-parser.ts:244-315`, composição em
  `editor-extension-sets.ts:152-352`.
- **Documento:** uma única autoridade no main, versionada por `ChangeSet`
  (`documents/index.ts:619-760`), com autosave por timeout
  (`documents/index.ts:745-758`) e gravação preservando BOM/linefeed
  (`documents/index.ts:1527-1600`).
- **Metadados/índices:** FSAL lê e faz cache em disco de descritores
  (`fsal/fsal-cache.ts:1-20`, `fsal/fsal-file.ts:74-120`), links e tags são
  derivados desse cache.
- **Exportação:** Pandoc + defaults YAML gerados em tempo de execução
  (`commands/exporter/index.ts:164-280`), com extensão `wikilinks_title_*`
  habilitada dinamicamente (`:187-196`) e um filtro Lua para remover/deslinkar
  wikilinks (`static/lua-filter/links.lua:44-58`).
- **Temas:** variáveis CSS no CodeMirror (`theme/editor.ts:25-210`) + `.dark`
  no `body` + `custom.css` externo
  (`common/modules/window-register/register-themes.ts:60-100`).

Detalhes, tradeoffs e caminhos: `arquitetura.md`.

## 5. Exemplo mínimo executável (real)

O único jeito barato de executar código real do Zettlr sem instalar o app é
importar módulos puros com Bun (que resolve os `paths` do tsconfig). Ver
`exemplos/README.md`. Exemplo:

```bash
cd dossie/Zettlr/exemplos
bun run link-index.ts     # índice de links real (extractFileId + extractFromFileDescriptors)
bun run replace-links.ts  # reescrita de links na renomeação
bun run md2html.ts        # serialização Markdown -> HTML do app
bun run link-edges.ts     # casos de borda do parser de [[...]]
```

Saída observada de `bun run link-index.ts`:

```
ID extraído do nome: "20240101120000"
ID de arquivo sem ID: ""
inbound(Metodo): [ "/notes/20240102090000 Revisao.md", "/notes/Sem ID.md" ]
outbound(Revisao): [ "/notes/20240101120000 Metodo.md" ]
outbound(Inexistente) por link quebrado: []
findExact("Metodo") [nome parcial]: null
findExact("20240101120000 Metodo"): /notes/20240101120000 Metodo.md
findExact("Metodo.md") [com extensão]: null
findExact("Revisao") [basename duplicado]: /notes/arquivo/Revisao.md
findExact("Inexistente"): null
```

## 6. Prós e contras para o nosso caso

**Copiar (ideias):**
- Índice derivado `links[]`/`id` por arquivo, reconstruível a partir do FSAL, sem
  banco de dados separado (`links/index.ts:80-86`).
- ID estável extraído de nome **ou** conteúdo, com exclusão explícita de
  ocorrências dentro de `[[...]]` (`extract-file-id.ts:33-56`) — um detalhe que
  quase todo projeto esquece.
- Autoridade única de documento com `ChangeSet` serializado: resolve
  multi-janela, autosave e undo sem duplicar lógica no renderer
  (`documents/index.ts:690-760`, `remote-doc.ts:54-164`).
- Completions que incluem alvos quebrados, para "criar ao completar"
  (`MainEditor.vue:632-657`).
- Exportação por defaults YAML + filtro Lua, que mantém o pipeline auditável
  (`commands/exporter/index.ts:164-280`).

**Adaptar (com correções):**
- Resolução por basename: precisa de desambiguação por caminho relativo e de
  aviso de ambiguidade (hoje o primeiro descriptor vence silenciosamente,
  `links/index.ts:102-124`).
- Renomeação: reescrever também links Markdown e frontmatter, e tratar colisão
  de nomes.
- Semântica de link quebrado: expor diagnóstico (underline + lista) em vez de
  descartar.

**Evitar:**
- Acoplar editor a IPC/Electron: o `remote-doc` só funciona porque existe um
  main process; nosso núcleo Rust headless deve ser a autoridade e o front-end
  um cliente.
- Reindexação total a cada evento do FSAL (`links/index.ts:63-71`): em vaults
  grandes isso é O(n) por mudança.
- Serialização HTML própria com artefatos de espaço (ver `exemplos/md2html.ts`;
  `md2html` emite ` ` extras e linhas em branco).
