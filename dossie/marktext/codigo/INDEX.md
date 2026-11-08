# Índice das cópias de código — MarkText / Muya

Projeto: `ref/marktext/` (monorepo MarkText). Engine: `ref/marktext/packages/muya/`.
Commit local: `a520bef` (2026-11-06). Versões declaradas: `marktext-monorepo` 0.24.0-dev
(`ref/marktext/package.json:3`), `@muyajs/core` 0.2.0 (`ref/marktext/packages/muya/package.json:3`).

Licença de todo o código copiado: **MIT** — `ref/marktext/LICENSE`:
"Copyright (c) 2017-present Luo Ran; Copyright (c) 2018-present MarkText Contributors".

Os três **trechos parciais** (`content-block.ts`, `inline-engine.ts`, `markdown-roundtrip.ts`)
trazem cabeçalho com origem, linhas, commit e licença. Os três **arquivos completos**
(`state-types.ts`, `json-state.ts`, `edit-transactions.ts`) são cópias **verbatim** — sem cabeçalho
adicional, para casar linha a linha com a fonte; origem, commit e licença estão nesta tabela.

| Arquivo local | Origem em `ref/` | Escopo | Por que foi copiado |
|---|---|---|---|
| `state-types.ts` | `packages/muya/src/state/types.ts` (arquivo completo, 288 linhas) | Modelo de dados | É o contrato canônico do documento: união `TState` de folhas e contêineres, com `name` como discriminante e o texto Markdown cru dentro de cada bloco. Sem ele não se entende nada do resto. |
| `json-state.ts` | `packages/muya/src/state/index.ts` (arquivo completo, 158 linhas) | Fonte da verdade em memória | Mostra o `JSONState` sobre `ot-json1`: `preview` (candidato sem commit), `apply`, `buildReplaceOp`, `getState` (clone) e `getMarkdown` (serializa sob demanda). Explica a separação entre estado e texto. |
| `edit-transactions.ts` | `packages/muya/src/editor/transactions/index.ts` (arquivo completo, 231 linhas) | Transações/undo | Implementa batching (`enqueue`, `flush`, `requestAnimationFrame`), escopos aninhados, rollback por exceção, regras de edição (`EDITING_RULES`) e a fronteira de histórico (`merge`/`isolated`/`ignore`). |
| `content-block.ts` | `packages/muya/src/block/base/content.ts:356-434` (trecho contínuo) | Folha editável | A classe `Content`: `contenteditable`, `path` derivado do pai, e o setter de `text` que converte diferença textual em `ot-text-unicode` ancorado no caminho JSON. |
| `inline-engine.ts` | `packages/muya/src/inline/index.ts` (completo, 95 linhas) + `packages/muya/src/inline/referenceDefinitions.ts` (completo, 72 linhas) | Inline + referências | O `patch()` que re-tokeniza e substitui `innerHTML` do bloco, e o índice de definições `[ref]: url` varrendo o estado (com cache por identidade de snapshot). |
| `markdown-roundtrip.ts` | `packages/muya/src/state/markdownToState.ts:56-200` (trecho contínuo) + `packages/muya/src/state/stateToMarkdown.ts:67` e `:108-169` (declaração da classe + corpo de `generate`/`_convertStatesToMarkdown`; campos e construtor omitidos) | Parser e serializador | O caminho Markdown → `TState` (via `marked` + pilha de pais) e `TState` → Markdown (por tipo de bloco, com resolvedor de separação). A normalização fence-vs-indentado fica em `_prepareBlockBoundary` (`stateToMarkdown.ts:170-180`), fora do trecho copiado. |

## O que **não** foi copiado e por quê

- `src/muya.ts` (1.646 linhas) — a API pública está resumida com citações de linha no `README.md`
  e no `arquitetura.md`; copiar inteiro não acrescentaria modelo.
- `src/state/stateToMarkdown.ts` completo (719 linhas) — só o laço principal foi copiado.
- `src/editor/index.ts` (walkers `pick`/`drop`, ~700 linhas) — descrito por citação de linha
  (`:617-638`), sem cópia, para manter `codigo/` em 6 arquivos.
- Folhas em `src/block/**` e `src/ui/**` — existem 39 blocos registrados
  (`ref/marktext/packages/muya/src/block/index.ts:41-81`) e 17 classes de plugin de UI em 22
  diretórios; a amostra
  `content-block.ts` cobre o padrão de folha.

## Aviso de uso

Estes arquivos são **material de análise**. Eles não compilam isoladamente: dependem de
`ot-json1`, `snabbdom`, `marked`, do lexer inline próprio e dos módulos internos citados.
A intenção é estudar o desenho — o `rich-editor-auy` é Rust headless e não reutiliza
TypeScript nem DOM. Nada aqui foi executado (ver `exemplos/README.md`).
