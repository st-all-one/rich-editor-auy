# Revisão independente — dossiê Zettlr 4.9.0

Revisor: sessão de revisão (rodada extra). Data: 2026-11-07.
Fonte de verdade: `ref/Zettlr/` (árvore completa, `node_modules` ausente).

---

## 1. Método

1. Li `dossie/_GUIA.md`, `dossie/_RUBRICA.md`, `dossie/_REVISAO-CHECKS.md` e o
   dossiê inteiro (`README.md`, `arquitetura.md`, `avaliacao.md`,
   `codigo/`, `exemplos/`).
2. Reexecutei os três exemplos existentes e escrevi/executei um quarto
   (`exemplos/link-edges.ts`) para testar casos de borda do parser:
   - `bun run link-index.ts` → saída **idêntica** à registrada no dossiê;
   - `bun run replace-links.ts` → saída idêntica;
   - `bun run md2html.ts` → saída idêntica, inclusive os artefatos de espaço;
   - `bun run link-edges.ts` → nova evidência (ver §2).
3. Confirmei cada afirmação central no código, com `sed`/`grep`/`diff`; corrigi
   todos os `caminho:linha` que não batiam (lista em §3).
4. Conferi as cópias de `codigo/` por `diff` contra `ref/`: todas batem
   **byte a byte** com o intervalo declarado (após o cabeçalho).
5. Não abri o app Electron, não rodei `mocha`, não rodei export Pandoc.

---

## 2. Achados novos (não estavam no dossiê)

### 2.1 `[[alvo|]]` deixa o link invisível

`zkn-link-parser.ts:70` guarda com `pipeIdx > 0 && pipeIdx < contents.length`.
O segundo teste **não** rejeita pipe no fim: `pipeIdx` nunca é igual a
`contents.length` quando há pipe. Verificado em execução
(`exemplos/link-edges.ts`):

```
"[[alvo|]]" -> target = "alvo" | title = ""
"[[|título]]" -> target = "|título" | title = undefined
```

Consequências: `[[alvo|]]` cria `ZknLinkTitle` vazio; `render-links.ts:44-49`
esconde `ZknLinkContent` + `ZknLinkPipe` (porque conteúdo, título e pipe
existem), então **o link desaparece da tela**. `[[|título]]` faz o pipe virar
parte do alvo e o link não resolve.

O dossiê afirmava "rejeita pipe no começo ou no fim (`:71`) para não gerar
link/título vazio" — **corrigido** em `arquitetura.md` §3, `README.md` §3,
`avaliacao.md` (matriz, linha "Schema") e `codigo/INDEX.md`.

### 2.2 Âncora `#` é inconsistente entre clique e índice

`[[arquivo#Heading]]` mantém a âncora no alvo (execução: `target =
"Metodo#Intro"`). `force-open.ts:52` corta no `#` e abre o arquivo, mas
`retrieveOutbound`/`retrieveInbound` comparam a string inteira e **não** contam
o link. Ou seja: o clique funciona e o backlink não aparece. Registrado em
`README.md` §3, `arquitetura.md` §3 e `exemplos/README.md`.

### 2.3 O guarda de `]]` em `extract-file-id` é código morto

`extract-file-id.ts:51` testa `match[1].endsWith(']]')`, mas `match[1]` é o
**grupo de captura** (o ID), que nunca termina em `]]`. Só o guarda de `[[`
(`:47`) funciona. O dossiê afirmava que ambos rejeitavam — **corrigido** em
`arquitetura.md` §2 e `codigo/INDEX.md`.

### 2.4 O "não reescreve link Markdown" tem cobertura de teste

Além do meu exemplo, o próprio repositório fixa o comportamento:
`test/replace-links.spec.ts:45-46` mantém `[should not work]([[Zettelkasten]])`
intacto. Promovido de `E` para `C`+`T` nas evidências (`arquitetura.md` §4,
`README.md` §3, `avaliacao.md` §3).

### 2.5 Clique em link também dispara busca global

`win-main/MainEditor.vue:519-522`: com `zkn.autoSearch` (default `true`),
seguir um wikilink abre o alvo **e** emite `globalSearch` com o alvo. É UX real
que faltava no fluxo descrito; adicionada em `README.md` §2.

### 2.6 `idf` das tags usa a soma de ocorrências, não o nº de documentos

`service-providers/tags/index.ts:199-201`: `N` é a soma de `files.length` de
todas as tags. A fórmula citada estava certa, mas ambígua; explicitada em
`arquitetura.md` §4.

### 2.7 Contagens e números

| Afirmação original | Estado real | Ação |
|---|---|---|
| "48 testes em `test/`" | 47 arquivos `*.spec.ts`, 4 deles `.ignore` → **43 ativos** | corrigido em `avaliacao.md` |
| "102 ocorrências de `aria-*`/`role=`" | 34 `aria-*` + 66 `role=` = **100** | corrigido em `arquitetura.md` §8 |
| "514 arquivos TS/Vue" | 514 (`406` `.ts` + `108` `.vue`) | confirmado |
| "`editor.css` 313 linhas" | 313 | confirmado |
| "~30 variáveis `--zettlr-editor-*`" | ~30 (amostragem em `theme/editor.ts:25-110`) | confirmado |
| "média simples 2,9" | 29/10 = 2,9 | confirmado |

### 2.8 Nenhum diagnóstico de link quebrado (confirmado por busca)

`grep -rn "unresolved|broken link|dead link"` em `markdown-editor/` e
`service-providers/links/` retorna só mensagens de imagem/nota de rodapé.
A afirmação central do dossiê se sustenta.

---

## 3. Correções de caminho/linha aplicadas

Todas verificadas por leitura direta em `ref/Zettlr/`.

### `arquitetura.md`

| Antes | Depois | Motivo |
|---|---|---|
| `_fileLinkDatabase` em `links/index.ts:38-41` | `:32-33`, init `:41-42` | declaração dos `Map` |
| `file-parser.ts:46-160` (tabela) | `:57-158` | função real |
| BOM/linefeed `61-62`; contadores `89-93`; frontmatter `98-140` | `66-67`; `91-95`; `100-156` | deslocamento |
| `FRONTMATTER_VARS:39-48` | `:39-49` | fim real |
| `markdown-parser.ts:250-330`; `:90-100`; `:255-330`; `:258-259` | `:244-315`; `:244-254`; `:257-315`; `:256-257` | arquivo tem 315 linhas |
| `zkn-link-parser.ts:38-88`; `:44-50`; `:65-86`; `:71`; `:15-30` | `:38-90`; `:45-47`; `:61-87`; `:70`; `:18-30` | idem |
| `render-links.ts:40-57`; `:57-70` | `:40-51`; `:52-72` | ramos reais |
| `render-citations.ts:44-82`; `:68-73`; crossrefs `:47-65` | `:36-83`; `:72-75`; `:37-56` | idem |
| `remote-doc.ts:54-183`; `:126-135`; `:64-120` | `:54-164`; `:130-135`; `:64-160` | arquivo tem 164 linhas |
| `fsal-file.ts:100-107` (10 MB) | `:94-100` | `file.size > 10_000_000` |
| `fsal-file.ts:74-120` (cache) | `:105-116` | `hasCache`/`applyCache` |
| `exporter/index.ts:168-280`; `:185-190`; `:198-215`; `:227-240`; `:265-268` | `:164-280`; `:187-196`; `:200-215`; `:218-237`; `:260-264` | idem |
| `citation-parser.ts:229-300`; `:40-72` | `:229-310`; `:46-80` | idem |
| `get-bibliography-for-descriptor.ts:28-50` | `:27-55` | idem |
| `file-rename.ts:135-180`; diálogo `:154-166` | `:133-183`; `:157-166` | idem |
| `replace-links.ts` "ordem reversa `:62-74`" | `:65-72` | idem |
| `RelatedFilesTab.vue:149-196` | `win-main/sidebar/RelatedFilesTab.vue:147-196` | caminho errado |
| `GraphView.vue:500` | `win-stats/GraphView.vue:501` | caminho + linha |
| config defaults `372-382` | `374-380` | idem |
| BOM `documents/index.ts:1577-1582` | `:1581-1582` | idem |

### `README.md`

Mesmos ajustes de linha (`render-links.ts:21-90`, `files.ts:63-100` e
`:108-119`, `citation-parser.ts:229-310`, `render-citations.ts:36-83`,
`file-rename.ts:133-183`, `fsal-directory.ts:37-49`, `links/index.ts:160-178`,
`MainEditor.vue:632-657`, `force-open.ts:50-61`, `remote-doc.ts:54-164`,
`exporter/index.ts:164-280` e `:187-196`, `markdown-parser.ts:244-315`,
`GraphView.vue` → `win-stats`). Novas linhas sobre casos de borda e
`autoSearch`.

### `avaliacao.md`

As **notas não mudaram**: usabilidade 4, API 1, neutralidade 2, modelo 4,
extensibilidade 2, performance 3, a11y 3, docs 3, manutenção 4, fit 3
(média 2,9). Corrigi apenas evidências (linhas), a contagem de testes e
acrescentei os casos de borda ao comentário de usabilidade. A matriz de
cobertura continua com 1 item `—` (IME) e 3 parciais; cobertura declarada
~92%, agora com evidência adicional (`link-edges.ts`).

### `codigo/INDEX.md`

`extract-file-id.ts:1-57`, `zkn-link-parser.ts:1-90`,
`remote-doc.ts:54-164`; comando de conferência corrigido de `sed 9,93` para
`11,93` (o cabeçalho local tem 9 linhas + 1 linha em branco); descrição do
guarda de pipe e do ID atualizada. Cabeçalho GPL e aviso de **não
reutilização** preservados.

### `exemplos/`

Novo `link-edges.ts` (executado; saída colada em `exemplos/README.md`) e
registro do achado na tabela de arquivos.

---

## 4. Auditoria das exigências do ticket

| Exigência | Situação | Evidência |
|---|---|---|
| GPL-3.0 e trechos pequenos; nenhum código reutilizável | **Confirmado** | `LICENSE:1`; `package.json:14`; 5 arquivos em `codigo/`, todos com cabeçalho de origem e aviso explícito de não reutilização; `diff` limpo contra `ref/` |
| `LinkProvider`: `path→links[]`, `path→id` | **Confirmado** | `links/index.ts:32-33`, `:84-85` |
| Reindexação total | **Confirmado** | `:63-71` (evento FSAL) → `:80-86` (`new Map(...)` completo a cada evento) |
| Três formas de casar | **Confirmado** | `:102-124`: ID; basename com extensão; basename sem extensão (o terceiro ramo roda sempre, mas só casa se `name === query + ext`) |
| Link quebrado descartado | **Confirmado** | `:160-178`; execução `outbound("Inexistente") → []` |
| Rename só em `[[...]]` | **Confirmado** | `replace-links.ts:35-75` + `test/replace-links.spec.ts:45-46` + execução |
| CodeMirror 6 | **Confirmado** | `package.json:50-80` (`@codemirror/*` 6.x), `editor-extension-sets.ts:152-352` |
| Citações/citeproc | **Confirmado** | `citation-parser.ts:229-310`, `citeproc/index.ts:513/563`, `HTML.yaml:15` |
| Export Pandoc | **Confirmado** (código; não executado) | `exporter/index.ts:164-280`, `links.lua:44-58` |
| Projeto como lista ordenada | **Confirmado** | `fsal-directory.ts:37-49` (`files: []` "sorted"), `dir-project-export.ts:76-98` |
| Exemplos executados | **Confirmado** | 4 scripts; saídas reproduzidas nesta revisão |

**Veredito mantido e reforçado:** o Zettlr é **app**, não biblioteca. O valor
para o `rich-editor-auy` é a **UX de referências** (digitar `[[`, título
separado do alvo, painel de relações, criar ao completar) e a **autoridade de
documento** (um processo dono do `Text`, versão por `ChangeSet`, autosave e
gravação fiel a BOM/LF). Nada disso é pacote: API pública = 1, e a única forma
de reuso seria copiar desenho, não código.

---

## 5. Confiança

**88%** (subiu de ~72% para 88% no eixo de verificação executada).

Justificativa pela âncora da rubrica: código e docs fortes, com **quatro**
exemplos executados e reproduzidos; falta execução do app (UI real, IME,
clipboard, leitor de tela) e do pipeline Pandoc. Tudo o que é afirmado sobre
links, parser, ID, rename, serializer e estrutura do projeto foi conferido em
código e, nos casos centrais, também em execução.

O que **não** verificaria antes: os casos de borda de pipe e âncora não estavam
cobertos por leitura estática; agora estão.

---

## 6. Lacunas e o que não foi executado

| Item | Motivo | Impacto na confiança |
|---|---|---|
| App Electron (`yarn start`) | exige build webpack + Electron 44; `node_modules` ausente | a11y visual, IME, clipboard e foco ficam inferidos do código |
| `yarn test` (`mocha`) | mesmo motivo; testes lidos, não rodados | `T` usado como intenção declarada, não como resultado observado |
| Export Pandoc real | exige binário Pandoc e o app | pipeline descrito por código + assets (`HTML.yaml`, `links.lua`) |
| Leitor de tela / contraste | não há teste automatizado no repo; app não aberto | nota de a11y segue em 3 |
| Performance medida | só os números declarados em `fsal-cache.ts:9-18` | nota 3, sem benchmark próprio |
| Colaboração em rede | não existe no produto; só renderer↔main | não é lacuna, é ausência confirmada |

Também não rodei `vue-tsc`/`eslint` sobre o repo (fora do escopo barato), então
não afirmo que ele compila — apenas que o código lido é coerente.
