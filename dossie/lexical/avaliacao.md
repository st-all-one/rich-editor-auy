# Avaliação — Lexical (`ref/lexical`, v0.53.0, MIT)

Rubrica: 0 = inexistente/nocivo, 5 = referência de mercado. Cada nota vem com evidência
(`arquivo:linha`, teste, ou saída de comando). Onde não houve execução, está escrito
`não executado`.

Método: leitura direcionada de `packages/lexical/src/` (33.422 linhas de `.ts` só no core), dos
pacotes listados na tarefa, de `examples/vanilla-js` e de testes; e um **probe executado** contra
`lexical@0.53.0` publicado no npm (`exemplos/README.md`). O probe **não** cobre o checkout
HEAD nem navegador real.

---

## 1. Matriz por dimensão

| Dimensão | Nota | Evidência | Comentário |
|---|---:|---|---|
| **Usabilidade (para quem integra)** | **2/5** | `examples/vanilla-js/src/main.ts:16-20` + `AppExtension.ts:20-44`; `README.md` §7; probe: `KEY_ENTER_COMMAND tratado só pelo núcleo? false` | O exemplo mínimo tem 20 linhas e funciona, mas **esquecer `RichTextExtension` produz um editor que aceita texto e ignora Enter** — modo de falha silencioso. Para paridade com um editor "normal" são necessários 5–8 pacotes. |
| **API** | **4/5** | `LexicalNode.ts:1100-1137,1645-1690`; `LexicalUpdates.ts:1308-1400`; `LexicalCommands.ts` (295 linhas, comandos tipados) | Superfície pequena e consistente: `update`/`read`/`dispatchCommand`/`register*`. Tipagem forte com `Klass`/`LexicalNodeReplacement`. Perde 1 pela convivência de três sistemas de config de nó e pela pegadinha de `dispatchCommand` em comandos de notificação. |
| **Neutralidade de estilo** | **4/5** | `find node_modules/lexical node_modules/@lexical -name '*.css'` → vazio (probe); `LexicalParagraphNode.ts:72-80`; `LexicalTextNode.ts:194-265`; **contra-evidência**: `LexicalEditor.ts:1749-1751` e `LexicalTextNode.ts:998` | Nenhum CSS embutido, nenhum reset, nenhuma fonte; tema é mapa papel→classe. Mas o core **impõe** estilo inline — 3 propriedades no root em todo `setRootElement` e `white-space: pre-wrap` em cada texto exportado. Como inline vence folha de estilo, o host não consegue sobrescrever sem `!important`; o âncora 5 ("nenhum CSS/estrutura imposta") não é satisfeito. Fica 4, não 5. |
| **Modelo de dados** | **5/5** | `LexicalEditorState.ts:193-330`; `LexicalNode.ts:1072-1085`; probe (JSON round-trip, split por formato) | Árvore + `NodeMap` imutável, chaves estáveis, seleção como dado, serialização determinística. É o melhor modelo entre os projetos comparados para o nosso caso. |
| **Extensibilidade** | **4/5** | `defineExtension`/`LexicalBuilder` (`LexicalBuilder.ts:81-227`); `RichTextExtension.ts:111-141`; `NodeChildrenConfig` (`LexicalNode.ts:137-140`) | Nó novo é cidadão de primeira classe (dados + DOM + JSON + HTML + transform). `dependencies`/`conflictsWith`/`dispose` resolvem composição de forma verificável. Perde 1 pela dívida dos três sistemas de schema. |
| **Performance** | **4/5** | `LexicalGenMap.ts`; `$reconcileNode` caminho rápido (`LexicalReconciler.ts:1816-1844`); `__bench__/README.md`; medição nossa (`exemplos/saida-observada.txt`, 5 execuções: 1.000 parágrafos = 13,7–63,6 ms build / 3,9–14,3 ms `toJSON` / 242 KB; 0,95–3,50 ms por edição discreta em 1k parágrafos) | Muito bom para edição incremental. Sem serialização incremental e sem garantia de O(1) por edição em documento grande. Números próprios têm dispersão de ~5×; valem como ordem de grandeza. |
| **Acessibilidade** | **3/5** | `lexical-a11y/src/index.ts:506,590,649,696,709,722`; `examples/vanilla-js/index.html:14-17` (role/aria à mão) | A estrutura está certa e é sofisticada (região viva única, ZWSP para re-anúncio, focus trap, roving tabindex). Mas tudo é **opt-in** e os atributos base do `contenteditable` são responsabilidade do host. |
| **Documentação** | **4/5** | `packages/lexical-website/docs/` — 6 diretórios (concepts, extensions, serialization, getting-started, react, collaboration) e 63 arquivos `.md/.mdx`, incluindo `concepts/editor-state.md`, `concepts/transforms.md`, `concepts/node-state.md`, `concepts/named-slots.md`, `extensions/` (10 arquivos, com `migration.mdx`), `serialization/dom-render.md` e até `concepts/compared-with-prosemirror.md`; `upgrading.md`, `error.md`, `testing.md`; `CHANGELOG.md` com 131 versões e `Detect:`/`Migration:` | Documentação densa, atualizada e com um capítulo de comparação com ProseMirror que é honesto sobre os tradeoffs. O que confunde não é estar desatualizada: é que **duas gerações de API convivem na mesma doc** (`getting-started/creating-plugin.mdx` vs `extensions/defining-extensions.md`), sem um "prefira este" explícito — e boa parte da geração nova está marcada `experimental`. |
| **Manutenção** | **3/5** | `.changeset/` com 130 entradas pendentes; `package.json` `version: 0.53.0`; `diff -rq` npm 0.53.0 × HEAD: 12 arquivos de `packages/lexical/src` diferentes + `LexicalWarning.ts` novo | Projeto vivo, com disciplina de breaking change. Risco: pré-1.0, churn de API, e o HEAD do checkout já divergiu do publicado. (`registerHistory` **não** diverge — ver §4.) |
| **Adequação ao `rich-editor-auy`** | **3/5** | ver §3 | Modelo e arquitetura: copiar. Runtime: inaplicável (é TS/JS sobre `contenteditable`; nós somos Rust headless). Serve como **especificação de design**, não como dependência. |

**Média simples:** 3,6/5. Se excluirmos "adequação ao nosso caso" (que mede outra coisa):
**3,67/5**. (A média caiu de 3,7/3,78 porque neutralidade foi de 5 para 4 — ver acima.)

## 2. Matriz de usabilidade (15 itens, meta ≥85% evidenciado)

Status possíveis: `verificado no código`, `verificado em teste`, `verificado em exemplo`,
`verificado em doc`, `verificado por execução`, `não verificado`.

| # | Item | Status | Evidência |
|---|---|---|---|
| 1 | Instalação e pré-requisitos | **verificado por execução** | `npm i lexical@0.53.0 …` OK em 13,9 s; `engines.node >=20.19.0`, `pnpm>=11` no monorepo (`package.json:11-14`); Node 22.23.2 usado. Monorepo exige pnpm, `node_modules` ausente no checkout (`test -d node_modules` → NO), então build/lint **não executados** |
| 2 | Inicialização mínima | **verificado em exemplo + execução** | `examples/vanilla-js/src/main.ts:16-20` (6 linhas) e `AppExtension.ts:20-44`; probe criou editor headless e populou o root |
| 3 | Edição básica (texto, parágrafo, negrito/itálico/link) | **verificado por execução** | probe: Enter cria `<p>` com `RichTextExtension` e não sem ela; `FORMAT_TEXT_COMMAND: 'bold'` → `format:1`; `formatText('italic')` → `format:2`; **link não importa sem `@lexical/link`** (evidência negativa útil) |
| 4 | Seleção, teclado, undo/redo | **verificado em código + execução** | `RangeSelection`/`NodeSelection` (`LexicalSelection.ts:614,838`); mapeamento tecla→comando (`LexicalEvents.ts:2163-2240`); undo/redo observados: `abc` → `a` → `abc`; `CAN_UNDO` só aparece via listener |
| 5 | Clipboard | **verificado no código** | 3 MIME types (`lexical-clipboard/src/clipboard.ts:89,118,173`). Não executado: exige `DataTransfer` real de navegador — happy-dom não implementa o fluxo de paste de forma fiel |
| 6 | IME | **verificado no código + teste** | `_compositionKey` exclui o nó de transforms (`LexicalUpdates.ts:276-285`); sufixos por plataforma (`LexicalConstants.ts:100-112`); `LexicalIosKoreanIME.test.ts:9-23`, `LexicalAndroidChromeComposition.test.ts`, `Composition.spec.mjs`. Não executado em navegador |
| 7 | Schema e restrições de conteúdo | **verificado no código** | `NodeChildrenConfig` (`LexicalNode.ts:137-140`), `validateNodeStructure` (`LexicalStructure.ts:1892`), `NodeStructureExtension`; schema de serialização (`LexicalSchema.ts:276-290`). Marcado `@experimental` |
| 8 | Custom node / mark / plugin | **verificado por execução** | `LoudTextNode` com `static getType/clone/transform` rodou; serializou como `"type":"loud-text"`; transform executou 2× (fixpoint). Marks são bitmask, não classe de nó (`LexicalConstants.ts:38-56`) |
| 9 | Persistência e formato de serialização | **verificado por execução** | `toJSON()` observado (legado e compacto, `LexicalEditorState.ts:295-330`); `parseEditorState(obj)` + `setEditorState` round-trip OK; HTML export observado; markdown por transformers (`MarkdownTransformers.ts:1373-1423`) verificado só no código |
| 10 | Renderização e não interferência no CSS | **verificado por execução + código** | zero `.css` nos pacotes de biblioteca (`find`; os 48 do monorepo estão em playground/devtools/website); HTML exportado sem tema sai sem classes; DOM usa só tags semânticas (`LexicalTextNode.ts:168-192`) — mas o root recebe 3 estilos inline e o export injeta `white-space`/`text-transform` |
| 11 | Acessibilidade (ARIA, foco, leitor de tela) | **verificado no código** | `lexical-a11y/src/index.ts:506-729`; `role`/`aria-*` do root escritos à mão no exemplo (`index.html:14-17`) — ou seja: **não é automático**. Não executado com leitor de tela |
| 12 | Colaboração | **verificado no código** | `createBinding` (`Bindings.ts:175`), V2 experimental (`:216`), sync bidirecional (`SyncEditorStates.ts:195,359`), cursores (`SyncCursors.ts`), `_movedNodes`/`_textEdits` no estado (`LexicalEditorState.ts:195-233`). Não executado (exige servidor y-websocket) |
| 13 | Performance e limites conhecidos | **verificado por execução (micro)** | medições próprias em `exemplos/README.md`; bench oficial presente mas **não executado** (exige pnpm install no monorepo) |
| 14 | Erros, migração e diagnóstico | **verificado em código + execução (parcial)** | `onError`/`onWarn` (`LexicalEditor.ts:1127-1130`); invariants com mensagem acionável (`$getTextNodeOffset: invalid offset 5 for size 4` observado); `CHANGELOG` com `Detect:`/`Migration:`; devtools em `@lexical/devtools` |
| 15 | Licença e maturidade | **verificado no código + doc** | `LICENSE:1-3` MIT; `version 0.53.0`; 131 releases documentadas; 130 changesets pendentes |

**Cobertura:** 15 de 15 itens têm evidência de algum tipo → **100% da matriz do guia coberta**.
Mas a qualidade da evidência varia: **8 itens com alguma execução** (1, 2, 3, 4, 8, 9, 13 e 14
parcial), **7 apenas por leitura de código/teste/doc**. Não houve execução em navegador real, nem
build do checkout, nem suíte de testes do repo.

Declaração honesta: se o critério for "verificado por execução", a cobertura cai para ~53%. Se o
critério for "tem evidência rastreável", é 100%.

## 3. Adequação ao `rich-editor-auy`

### 3.1 O que o nosso projeto precisa e o que o Lexical oferece

| Necessidade do `rich-editor-auy` | Lexical oferece | Veredito |
|---|---|---|
| Modelo de documento em Rust, sem DOM | Modelo é TS e assume DOM em vários pontos (`$getDocument`, `MutationObserver`, `Selection`) | **Não reutilizável como código**; excelente como especificação |
| Zero CSS no core | Zero CSS de fato (verificado por `find`) | **Copiar o desenho** de tema-papéis |
| Schema / restrições de conteúdo | `NodeChildrenConfig` + `LexicalSchema` + `NodeState` — porém experimentais e triplicados | **Adaptar**; simplificar em um só mecanismo |
| Serialização estável e versionada | `version` por nó + `accepts` no schema + forma compacta | **Copiar** as ideias (`version`, compacto, `accepts`) |
| Referências entre publicações | Não há suporte nativo. O análogo mais próximo é `DecoratorNode` + `NodeState`/`$config` (`LexicalNode.ts:1768-1810`) | **Precisamos de conceito próprio**; `DecoratorNode` + `importDOM/exportJSON` é o molde |
| Front-end TS e PHP | `@lexical/react` é TS; não há cliente PHP. O protocolo viável é JSON do documento | **Copiar o contrato JSON** como fronteira de linguagem |
| Colaboração | Yjs maduro, com metadados de intenção no `EditorState` | **Copiar a lição**: guardar *intenção* (moved/split/text-edit), não só diff |
| A11y por padrão | `@lexical/a11y` é opt-in | **Evitar o opt-in**: no nosso caso, default |

### 3.2 Onde o Lexical seria a escolha certa (e não é o nosso caso)

Se o produto fosse **um editor web em React/TS**, Lexical seria provavelmente a melhor escolha
entre os projetos comparados: modelo imutável, extensão por nó, Yjs de primeira classe, tema
totalmente externo. Se o produto fosse **um editor headless em Node**, `@lexical/headless` cobre
apenas o modelo (proíbe `focus`/`getRootElement`/`registerRootListener` —
`lexical-headless/src/index.ts:26-41`), o que serve para geração de documento, não para edição.

Para um **core em Rust com múltiplos front-ends**, o valor do Lexical é 100% arquitetural. Copiar
código não faz sentido — não há como portar `LexicalReconciler.ts` (2.316 linhas) nem
`LexicalSelection.ts` (4.762 linhas) sem trazer o DOM junto.

### 3.3 Lista curta do que levar

1. **Núcleo sem comportamento.** Evento → comando → mutação. Modo é composição.
2. **`NodeMap` imutável + `getWritable()`** como única porta de mutação.
3. **Bitmask de formato por run** de texto.
4. **Transforms em fixpoint**, leaves antes de elements.
5. **Reconciliação por chave estável**, não por índice.
6. **Tema = papéis → tokens**, sem CSS.
7. **Metadados de intenção** no snapshot (moved/split/text-edit) para colaboração.
8. **`version` + `accepts`** por nó no JSON.
9. **Extensões com `dependencies`/`conflictsWith`/`dispose`**, validadas na inicialização.
10. **Erros com invariants acionáveis** em DEV (offset, tamanho, chave).

### 3.4 Lista curta do que não levar

1. Três sistemas de configuração de nó ao mesmo tempo.
2. Estilos inline impostos no root e no HTML exportado.
3. `contenteditable` como fonte de verdade.
4. Modo de edição como pacote separado **sem** default (falha silenciosa).
5. A11y opt-in.
6. Comandos de notificação e de ação com a mesma assinatura de retorno (`dispatchCommand`).

## 4. Limites desta avaliação

- **Não executei** build, lint, `vitest`, Playwright nem os benchmarks oficiais do checkout:
  `ref/lexical` não tem `node_modules` (`test -d node_modules` → NO) e instalar o monorepo
  (`pnpm-lock.yaml` de 1 MB, 39 pacotes, Chromium para e2e) está fora do escopo autorizado.
- O probe rodou contra **npm `lexical@0.53.0`**, não contra o HEAD do checkout. Divergência real
  verificada na revisão: `diff -rq node_modules/lexical/src × packages/lexical/src` acusa 12
  arquivos diferentes e `LexicalWarning.ts` só no HEAD. A assinatura de `registerHistory` é
  **idêntica** nas duas versões (`(editor, historyState, delay, dateNow?, onHistoryStateChange?,
  maxDepth?)`, `lexical-history/src/index.ts:613-620`); a divergência antes registrada aqui estava
  errada.
- Nenhum teste em navegador: tudo que depende de `Selection`, `beforeinput`, `MutationObserver` e
  IME foi avaliado **por código e pelos testes do próprio repo**, não por comportamento observado.
- Números de performance são microprobe em Node sem DOM; não são comparáveis ao bench oficial de
  jsdom.
- Não avaliei `lexical-devtools`, `lexical-eslint-plugin`, `lexical-mdast`, `lexical-compiler`
  nem o playground além de listar arquivos.
