# Índice das cópias de código

Origem: `ref/summernote/` — clone local de `summernote/summernote`, commit `ac5460e`
(2026-01-04), versão declarada `0.9.1` em `ref/summernote/package.json:4`.
Licença: **MIT** (`ref/summernote/LICENSE:1-3`), Copyright (c) 2015~ Summernote Team.
Todo o código copiado aqui é material de estudo; nada será reutilizado no `rich-editor-auy`.

Cada arquivo traz, no topo, um bloco `ORIGEM` com o caminho e as linhas exatas no `ref/`.
Arquivos marcados como *arquivo completo* são cópias integrais; os demais são **excertos**
delimitados por comentários `[... cortadas as linhas N-M ...]`.

| Arquivo | Origem em `ref/summernote/` | O que explica | Tipo |
|---|---|---|---|
| `01-summernote-entry.js` | `src/js/summernote.js:1-46` | Plugin jQuery, parsing de argumentos (`'code'` vs objeto), um `Context` por elemento, dispatch da API externa | completo |
| `02-Context.js` | `src/js/Context.js:1-243` | Ciclo de vida, `layoutInfo`, registro/invocação de módulos, `code()` (leitura/escrita do HTML), `triggerEvent` | completo |
| `03-History.js` | `src/js/editing/History.js:1-127` | Undo/redo por **snapshot de HTML + bookmark**; `historyLimit`; por que não há operações | completo |
| `04-range-excerpts.js` | `src/js/core/range.js:119-300, 380-600, 718-737` | `WrappedRange`: seleção normalizada, `deleteContents`, `wrapBodyInlineWithPara`, `insertNode`, `pasteHTML`, `bookmark` | excerto |
| `05-Editor-excerpts.js` | `src/js/module/Editor.js:339-500, 620-700, 735-800, 1024-1054` | O módulo central: listeners, keymap/undo, `beforeCommand`/`afterCommand`, `wrapCommand`, `insertImage`, `isEmpty` | excerto |
| `06-Codeview.js` | `src/js/module/Codeview.js:1-178` | Code view e `purify()` — o filtro por regex que é a única “sanitização” do projeto | completo |

## Como esses arquivos se encaixam

1. `01` recebe `$(el).summernote(...)`, cria `Context` (`02`) e guarda em `el.data('summernote')`.
2. `02` monta o layout via `ui.createLayout` e instancia os módulos listados em
   `src/js/settings.js:41-70`; chamadas como `summernote('code')` viram
   `Context.invoke('code')` (`02`, método `invoke`).
3. `05` escuta `keydown/keyup/input/paste/mouseup` no `.note-editable`; cada comando que muda
   conteúdo passa por `beforeCommand`/`afterCommand`, e este último chama `history.recordUndo()`
   (`src/js/module/Editor.js:690-699`).
4. `03` transforma o estado em string (`.html()`) + `bookmark` de seleção (`04`,
   `range.js:718-737`). O “modelo” do documento é, literalmente, o `innerHTML` do div.
5. `04` contém a álgebra de DOM usada por todo comando de edição (`insertNode`, `pasteHTML`,
   `deleteContents`) — não existe árvore semântica intermediária.
6. `06` mostra o caminho de entrada/saída do modo código, incluindo o filtro regex aplicado ao
   HTML que entra no editor.

## Aviso de precisão

Os caminhos/linhas citados valem para este clone (`ac5460e`). O projeto upstream mudou de
`.ts` para `.js` em versões recentes; em releases antigas os mesmos módulos aparecem como
`src/js/base/core/dom.ts` etc. Neste clone **não existe** `plugin/` na raiz: os plugins de
exemplo ficam em `public/plugin/{hello,databasic,specialchars}/`.
