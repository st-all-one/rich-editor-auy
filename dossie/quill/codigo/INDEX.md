# Índice das cópias de código — `dossie/quill/codigo/`

Código real copiado de `ref/quill/` (Quill 2.0.3, commit `539cbff`) e do pacote npm
`quill-delta@5.1.0`. Nenhum destes arquivos participa de build; são material de análise.
Os arquivos `01`–`04` são **cópias integrais** com cabeçalho de proveniência; `06` é uma
**coleta de trechos** com marcadores de linha.

| Arquivo local | Origem | Linhas | Licença | Por que foi copiado |
|---|---|---|---|---|
| `01-core-editor.ts` | `ref/quill/packages/quill/src/core/editor.ts` | 482 (+8 de cabeçalho) | BSD-3-Clause | É o tradutor Delta ↔ DOM: `applyDelta`, `update` com otimização de `characterData`, `getDelta`, `getFormat`, `removeFormat`. Sem ele não se entende onde o modelo vira DOM e vice-versa. |
| `02-blots-scroll.ts` | `ref/quill/packages/quill/src/blots/scroll.ts` | 442 (+8) | BSD-3-Clause | Raiz da árvore de blots e fronteira com o `MutationObserver`: `insertContents`, `deltaToRenderBlocks`, `update`, batching e `enable/disable`. |
| `03-core-selection.ts` | `ref/quill/packages/quill/src/core/selection.ts` | 486 (+8) | BSD-3-Clause | Tradução seleção nativa ↔ `Range(index,length)`, `cursor` blot, composição/IME e restauração de seleção após mutações. |
| `04-modules-history.ts` | `ref/quill/packages/quill/src/modules/history.ts` | 208 (+8) | BSD-3-Clause | Undo/redo inteiramente baseado em `Delta.invert`/`transform`, com coleta por tempo (`delay`) e `maxStack`. |
| `05-quill-delta.ts` | pacote npm `quill-delta@5.1.0`, `src/Delta.ts` | 573 (+9) | BSD-3-Clause | O modelo de documento canônico: `insert/delete/retain`, `compose`, `transform`, `invert`, `diff`, `chop`, atributos e handlers de embed. **Não está em `ref/quill/`**: em 2.0.3 é dependência externa (`ref/quill/packages/quill/package.json:13`). |
| `06-modules-clipboard-trechos.ts` | `ref/quill/packages/quill/src/modules/clipboard.ts` | 287 (trechos 30–72, 100–142, 160–253, 381–419, 620–670) | BSD-3-Clause | Matchers de colagem, `convertHTML` (DOMParser + normalizadores), cópia via `getSemanticHTML`, `traverse` pós-ordem e normalização de texto/espaços. |

## Proveniência e licença

- Quill: BSD-3-Clause, Copyright (c) 2017-2024 Slab, (c) 2014 Jason Chen, (c) 2013
  salesforce.com. Texto integral em `ref/quill/LICENSE`. Cópia integral permitida com
  preservação do aviso; os cabeçalhos adicionados não alteram o conteúdo original.
- quill-delta: **BSD-3-Clause**, Copyright (c) 2022, Slab, Inc. (ver `package/LICENSE` do
  tarball `https://registry.npmjs.org/quill-delta/-/quill-delta-5.1.0.tgz`). O revisor
  conferiu o arquivo do tarball: não é MIT, ao contrário do que `quill-delta` usava em
  versões antigas e do que este índice afirmava antes da revisão.

## O que **não** foi copiado e por quê

- `core/quill.ts` (1.050 linhas): fachada importante, mas o essencial (`modify`, `overload`,
  `shiftRange`, `setContents`) está descrito em `arquitetura.md` com linhas exatas.
- `modules/keyboard.ts` (833 linhas): citado por linha na análise; a cópia integral inflaria
  o dossiê sem acrescentar modelo.
- `assets/*.styl`: o ponto central para a pergunta de CSS; citado por linha
  (`src/assets/core.styl:40-215`), não copiado.
