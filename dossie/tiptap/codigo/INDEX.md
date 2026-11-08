# `codigo/` — o que foi copiado e por quê

Origem de tudo: `ref/tiptap/` (monorepo Tiptap, versão **3.30.3** — `ref/tiptap/packages/core/package.json`, `packages/pm/package.json`).
Licença: **MIT**, "Copyright (c) 2025, Tiptap GmbH" (`ref/tiptap/LICENSE.md:1-3`); cada pacote declara `"license": "MIT"` (ex.: `ref/tiptap/packages/core/package.json:16`).
Como é MIT, a cópia de arquivos completos é permitida; a origem está no topo de cada arquivo e repetida aqui.

Arquivos completos estão **verbatim**. Trechos têm um cabeçalho de 3 linhas adicionado pelo dossiê e, abaixo dele, o código original sem alteração (o intervalo exato está no cabeçalho).

| Arquivo aqui | Origem em `ref/` | Tipo | Por que foi copiado |
|---|---|---|---|
| `core-CommandManager.ts` | `packages/core/src/CommandManager.ts:1-208` | íntegro | Coração do modelo imperativo do Tiptap: `commands`, `chain()`, `can()`, `buildProps`, `createChainableState`, `preventDispatch`. É o que faz `editor.commands.toggleBold()` e `editor.can().chain().run()` existirem. |
| `core-style.ts` | `packages/core/src/style.ts:1-71` | íntegro | **É todo o CSS que o Tiptap injeta por padrão.** O arquivo tem 1353 bytes, mas o valor injetado é a string entre crases, com **1329 caracteres** — número medido no `<style data-tiptap-style>` em execução e conferido por aritmética (1353 − `export const style = \`` − crase final − `\n`). Prova concreta do que é "headless" e do que é estrutural/inevitável para `contenteditable`. |
| `core-Editor.createView.excerpt.ts` | `packages/core/src/Editor.ts:633-715` | trecho | Montagem real do `EditorView`: composição de `dispatchTransaction`, `transformPastedHTML`, `reconfigure({plugins: []})` antes de `setProps`, `prependClass()` e `injectCSS()`. Explica classes, plugins e DOM. |
| `core-Editor.dispatchTransaction.excerpt.ts` | `packages/core/src/Editor.ts:717-809` | trecho | O pipeline de transação: `captureTransaction`, `applyTransaction`, eventos `beforeTransaction`/`transaction`/`selectionUpdate`/`focus`/`blur`/`update` e o contrato de `preventUpdate`. É onde o Tiptap "embrulha" o ProseMirror. |
| `core-Extendable.excerpt.ts` | `packages/core/src/Extendable.ts:587-668` | trecho | `class Extendable`: `options`/`storage` como getters derivados de `addOptions`/`addStorage`, `configure()` via `mergeDeep`, `extend()` com cadeia `parent`/`child`. É a mecânica que substitui herança por composição. |
| `extension-bold.bold.tsx` | `packages/extension-bold/src/bold.tsx:1-163` | íntegro | Extensão completa e real de **mark**: `parseHTML` com 4 regras (incluindo `style`), `renderHTML` em JSX (`/** @jsxImportSource @tiptap/core */`), `addCommands`, `addKeyboardShortcuts`, `addInputRules`, `addPasteRules` e hooks de Markdown. Serve de gabarito para o nosso "node/mark de publicação". |

Complementos copiados para `exemplos/experimentos/` (não são "coração" do projeto, mas são o que conseguimos **executar de verdade**):

| Arquivo | Origem | Uso |
|---|---|---|
| `exemplos/experimentos/mergeAttributes.ts` | `packages/core/src/utilities/mergeAttributes.ts:1-128` | Cópia verbatim do merge de `class`/`style`/atributos usado por todo `renderHTML`. Rodado com Node 22 (type stripping) — saída em `exemplos/README.md`. |

## O que ficou de fora (e por quê)

- `Editor.ts` inteiro (944 linhas): os dois trechos acima cobrem lifecycle e transações; o resto é getters (`getJSON`, `getHTML`, `isActive`, `$pos`) já resumidos em `arquitetura.md`.
- `ExtensionManager.ts` (589 linhas): descrito em `arquitetura.md` com linhas citadas; copiar inteiro não acrescentaria modelo, só volume.
- `Node.ts`/`Mark.ts`: são `Extendable` + tipagem de config; o exemplo real (`bold.tsx`) mostra o contrato melhor que o arquivo de tipos.
- `style.scss` dos demos: é CSS do **produto demo**, não da biblioteca — citado apenas como contraste.
