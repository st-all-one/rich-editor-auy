# Índice do código copiado

Todos os arquivos abaixo são **cópias verbatim** (nenhuma linha alterada, nenhum cabeçalho
inserido) dos tarballs oficiais publicados no npm. Consequência prática: **o número de linha
citado no dossiê vale igualmente para a cópia e para a fonte** (`codigo/node.ts:54` ==
`/tmp/pm-upstream/prosemirror-model-1.25.12/src/node.ts:54`).

Licença: **MIT**, Copyright (C) 2015-2017 Marijn Haverbeke e outros — texto integral em
`codigo/LICENSE` (copiado de `prosemirror-model-1.25.12/LICENSE`). MIT permite redistribuição de
arquivos completos com aviso de copyright; o aviso está neste INDEX e em `LICENSE`.

## Por que só os tarballs, e não `ref/prosemirror/`

`ref/prosemirror/` contém **apenas o meta-repositório** (README de encaminhamento, `bin/pm.js`,
`demo/`, `LICENSE`). O próprio `README.md:1` diz que o repositório mudou de casa, e `bin/pm.js:116`
(`install`) clona cada módulo como repositório irmão — ou seja, `src/` nunca esteve versionado ali.
`git ls-tree HEAD` no stub confirma: só `.github/`, `assets/`, `bin/`, `demo/`, `package.json`,
`tsconfig.json`, `LICENSE`, `README.md`, `CONTRIBUTING.md`, `.gitignore`.

Assim, a análise de código foi feita sobre os tarballs oficiais extraídos em `/tmp/pm-upstream/`
(comando registrado em `exemplos/README.md`), nas versões publicadas:

| pacote | versão | sha256 do `.tgz` |
|---|---|---|
| prosemirror-model | 1.25.12 | `0ac7e5deed004b7f983ca295fa1e84b19f7ee38995e39dee9809d9a4ebd5c6c8` |
| prosemirror-transform | 1.12.5 | `0b9b224a05fb67f8e44386f6daecbea46552dda075f4fd74a6daaa076d48e695` |
| prosemirror-state | 1.4.4 | `ad3eb60bd6db8e74b2576dc0b7c2800d6def362e0e990c9fb108fc4b724ee651` |
| prosemirror-view | 1.42.9 | `d92a5d70b1c9d20ffa582c710ba62f86a58cb8a7ec5c58b9abe402c8e9e5d403` |
| prosemirror-history | 1.5.1 | `ba3f0ac27eb586925b42e3acb840b7440ab6aaf9a368053874d9ece266df31b0` |
| prosemirror-collab | 1.3.1 | `2dcfc635cce412f028a838d47b351972cd46b6cf77e8e8357e2cbd15ebe3c0f0` |
| prosemirror-schema-basic | 1.2.5 | `0b2479d8164fcdd480e11a929259da55099708b903e5e42d8c30b06521254b31` |

Para `prosemirror-commands` (1.7.2), `prosemirror-keymap` (1.2.3), `prosemirror-inputrules` (1.5.1)
e `prosemirror-example-setup` (1.2.3) — usados só como contraste em keymap/menu — a fonte foi o clone
git do espelho oficial em `/tmp/pm-src/` (HEAD de cada repo; versões conferidas nos `package.json`
respectivos); nenhum trecho desses pacotes foi copiado para cá.

## Arquivos copiados

| arquivo local | origem | sha256 | por que foi copiado |
|---|---|---|---|
| `node.ts` | `prosemirror-model-1.25.12/src/node.ts` | `39defcbeff65f8de187e61d956a1b5b1080962eda32bb875ac1d1e025c31f0d2` | Define `Node`/`TextNode`: documento persistente, `nodeSize` (tokens +2), `copy/cut/slice/replace/resolve`. É o coração do modelo de dados. |
| `replace.ts` | `prosemirror-model-1.25.12/src/replace.ts` | `4f2fbf315fba1e70b7802f7c7f5e3478848c5f57f025c2b2169b680153f18b2c` | Define `Slice` (fragmento com lados abertos) e o algoritmo `replace`, incluindo `ReplaceError`. Explica por que colar/inserir exige "encaixe" estrutural. |
| `map.ts` | `prosemirror-transform-1.12.5/src/map.ts` | `ddb1fc54b8e49581b6f1f71b84689ccad8b2cb10c44ecc9c01d9b32045624e78` | `StepMap` (ranges `[start, oldSize, newSize]`), `MapResult` (`deleted*`, `recover`) e `Mapping` (pipeline + mirrors). É a resposta de posições/mapeamento. |
| `step.ts` | `prosemirror-transform-1.12.5/src/step.ts` | `1080eb080282f47c9ac6edf91b1ea788069a2644da4d7257b139ca610ab9e1ac` | Contrato `Step` (`apply/invert/map/getMap/merge/toJSON/jsonID`) e `StepResult`. Mostra como estender transformações com segurança. |
| `state.ts` | `prosemirror-state-1.4.4/src/state.ts` | `081d675bbeaa6598c17f45be84a84011c5e083b66f9c6282be0304a5f38f7f44` | `EditorState` imutável, campos base (`doc`, `selection`, `storedMarks`, `scrollToSelection`), plugins como campos e `applyTransaction`. |
| `prosemirror.css` | `prosemirror-view-1.42.9/style/prosemirror.css` | `1234171efd99cd16d1a6754c0f6586d0d8b69573432aff2e671cb6a26c9eaa57` | Prova material contra a tese "zero CSS": o pacote de view **exige** um stylesheet e o documenta em `src/README.md:5-6`. |

## O que foi analisado mas **não** copiado (para não inflar o dossiê)

Citados por caminho/linha em `arquitetura.md`/`avaliacao.md`, sempre com versão:

- `prosemirror-model-1.25.12/src/{schema.ts,content.ts,fragment.ts,mark.ts,resolvedpos.ts,to_dom.ts,from_dom.ts}`
- `prosemirror-transform-1.12.5/src/{transform.ts,replace_step.ts,replace.ts,mark_step.ts,structure.ts}`
- `prosemirror-state-1.4.4/src/{transaction.ts,plugin.ts,selection.ts}`
- `prosemirror-view-1.42.9/src/{index.ts,viewdesc.ts,input.ts,domchange.ts,domobserver.ts,clipboard.ts,decoration.ts,selection.ts}`
- `prosemirror-history-1.5.1/src/history.ts`, `prosemirror-collab-1.3.1/src/collab.ts`
- `prosemirror-schema-basic-1.2.5/src/schema-basic.ts`
- `prosemirror-commands@1.7.2/src/commands.ts`, `prosemirror-keymap@1.2.3/src/keymap.ts`,
  `prosemirror-inputrules@1.5.1/src/{inputrules.ts,rulebuilders.ts}`
- testes: `prosemirror-view-1.42.9` não publica testes no tarball; a suíte foi inspecionada no clone
  git: 3.271 linhas em `/tmp/pm-src/prosemirror-view/test/` (8.932 somando os testes de todos os
  pacotes clonados).
