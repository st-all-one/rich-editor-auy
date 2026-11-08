# Revisão independente — dossiê Lexical

Revisor não é o autor. Escopo: `dossie/lexical/` contra `ref/lexical/` (commit `5326e062`,
`0.53.0`, MIT). Método: leitura dirigida do código citado, `diff` das cópias, e **reexecução** dos
três probes contra `lexical@0.53.0` do npm em `/tmp/lexical-probe` (Node 22.23.2, sem DOM).

Nada em `ref/` foi alterado.

---

## 1. Achados (por afirmação central)

### 1.1 "O core não edita sozinho" — CONFIRMADO

- O core registra **exatamente cinco** handlers de tradução, todos em
  `packages/lexical/src/LexicalEvents.ts:1153-1175`: `BEFORE_INPUT_COMMAND`, `INPUT_COMMAND`,
  `COMPOSITION_START_COMMAND`, `COMPOSITION_END_COMMAND`, `KEY_DOWN_COMMAND`. Nenhum muta o
  documento; todos despacham comandos.
- `KEY_ENTER_COMMAND` só é tratado por `registerRichText`
  (`packages/lexical-rich-text/src/index.ts:1286`).
- Probe reexecutado: sem modo, `Enter` → `false` e root com 1 parágrafo; com
  `registerRichText`, → `true` e 2 parágrafos.
- **Nuance que o dossiê omitia:** o core já aplica um **keymap default fixo** (setas, Enter, Tab,
  `Delete`/`Backspace` de palavra, Ctrl/Cmd+B/I/U, Ctrl/Cmd+Z/Shift+Z →
  `LexicalEvents.ts:2163-2240`). É tradução, não edição, mas é comportamento imposto que o host
  não escolhe. Adicionado ao README §1 e a `arquitetura.md` §5.2.

### 1.2 Zero `.css` nos pacotes — CONFIRMADO COM ESCOPO

- `find packages -name '*.css'` = **48 arquivos**, mas **nenhum** em pacote de biblioteca:
  todos em `lexical-playground` (app), `lexical-devtools` (extensão de navegador) e
  `lexical-website` (site). `find node_modules/lexical node_modules/@lexical -name '*.css'` do
  probe = vazio.
- O dossiê dizia "Zero CSS no pacote" (singular) e "nenhum CSS" no README — verdadeiro para o que
  é publicado, ambíguo para o monorepo. Corrigido em `arquitetura.md` §11 e `avaliacao.md` item 10.

### 1.3 Três estilos inline na raiz — CONFIRMADO (linha errada)

- `LexicalEditor.ts:1749-1751` escreve `userSelect`, `whiteSpace`, `wordBreak` em todo
  `setRootElement`; `data-lexical-editor="true"` em `:1752`. O dossiê citava `1748-1752`.
- Consequência não explorada antes: inline vence folha de estilo, então o host **não** consegue
  trocar esses três sem `!important`. Registrado no README §5 e em `avaliacao.md`.

### 1.4 `white-space` no HTML exportado — CONFIRMADO (linha errada)

- `LexicalTextNode.exportDOM` injeta `element.style.whiteSpace = 'pre-wrap'` em **todo** texto
  exportado (`LexicalTextNode.ts:998`) e `text-transform` para lowercase/uppercase/capitalize
  (`:1000-1008`). O dossiê citava `:1000-1005`. Corrigido no README §5, `arquitetura.md` §11 e
  na lista "evitar" do README §11.

### 1.5 `EditorState` / `NodeMap` / transforms / reconciliação / seleção / IME / clipboard / undo / Yjs

Verificado no código, com correções pontuais:

| Afirmação | Estado | Ação |
|---|---|---|
| `EditorState` com `_nodeMap`, `_selection`, `_movedNodes`, `_splitNodes`, `_textEdits` (`LexicalEditorState.ts:193-261`) | correto | — |
| `toJSON` legado × compacto; `JSON.stringify` cai no legado porque recebe o nome da propriedade (`:308-330`) | correto | — |
| `$beginUpdate`/`$applyAllTransforms`/`$applyTransforms` (`LexicalUpdates.ts:1308-1400`, `:333-428`, `:251-275`) | correto | — |
| `static transform()` em `:1907`; `getTransformSetFromKlass` citado como `LexicalEditor.ts:994-1135` | **linha errada** (função em `:933`, chamada em `:1102`) | corrigido |
| `$reconcileNode` caminho rápido citado como `:1805-1835` | **linha errada** (fast path em `:1816-1844`) | corrigido em `arquitetura.md` §4 e `avaliacao.md` |
| `storeDOMWithKey` citado como `LexicalUtils.ts:689-707` | **arquivo/função trocados**: `storeDOMWithKey` está em `LexicalReconciler.ts:2294`; `:689` é `setNodeKeyOnDOMNode` | corrigido |
| `RangeSelection`/`NodeSelection`/`Point` (`LexicalSelection.ts:838/614/177`) | correto | — |
| `$internalCreateSelection` `:3568`, `$updateDOMSelection` `:4019` | correto | — |
| `_compositionKey` exclui nó de transforms (`LexicalUpdates.ts:276-285`) | correto | — |
| Clipboard: 3 MIME types (`clipboard.ts:89,118,173`) | correto | — |
| Undo/redo, merge 300 ms (`lexical-history/src/index.ts:74`, `:854`) | correto; probe reexecutado (`abc`→`a`→`abc`; `CAN_UNDO_COMMAND` direto = `false`) | — |
| Yjs: `createBinding` (`Bindings.ts:175`), sync `195/359`, V2 `489/576` | correto | — |
| `createHeadlessEditor` proíbe 8 métodos (`lexical-headless/src/index.ts:26-41`) | correto; o dossiê lista 3 — é exemplificativo, mas completo agora | — |

### 1.6 `codigo/INDEX.md` e cópias — CONFIRMADO

- `diff` dos quatro arquivos inteiros (`LexicalEditorState.ts`, `LexicalRichTextExtension.ts`,
  `exemplo-main.ts`, `exemplo-AppExtension.ts`) contra `ref/lexical/` = **sem diferenças**.
- Os dois trechos batem com os intervalos declarados (251-275, 288-306, 333-428; 168-192,
  194-265, 902-924), na ordem e com marcadores.
- `LICENSE` é MIT (`ref/lexical/LICENSE:1-3`); cabeçalhos preservados.
- Total 853 linhas confere: 331+141+144+164+45+28.
- **Erro corrigido:** o texto dizia "os dois arquivos com nome `*-(trecho)`"; os nomes reais são
  `LexicalUpdates-transforms.ts` e `LexicalTextNode-DOM-e-tema.ts`.

### 1.7 `exemplos/README.md` e probes — CONFIRMADO POR REEXECUÇÃO

- `node 01-headless-minimo.mjs`, `02-html-json-custom-node.mjs` e `03-custo.mjs` rodaram de novo;
  **01 e 02 reproduziram a saída literal** de `saida-observada.txt`.
- **Achado:** a variância de performance é muito maior que os "±20%" registrados. Cinco execuções
  deram, para 1.000 parágrafos, 13,7–63,6 ms de build e 0,95–3,50 ms por edição discreta
  (~5× de dispersão). O texto foi corrigido em README §11, `arquitetura.md` §12 e
  `exemplos/README.md`.
- **Erro factual corrigido:** o dossiê afirmava divergência npm × HEAD em `registerHistory`
  (`(editor, delayOrStore, …)`). **Não existe**: `diff` do `@lexical/history/src/index.ts` do npm
  0.53.0 contra o HEAD é vazio (994 linhas, idênticas) e a assinatura é
  `(editor, historyState, delay, dateNow?, onHistoryStateChange?, maxDepth?)`. `delayOrStore` é
  parâmetro interno de `createMergeActionGetter` (`:283`). A divergência npm × HEAD é real, mas em
  outro lugar: 12 arquivos de `packages/lexical/src` diferem e `LexicalWarning.ts` só existe no
  HEAD.

### 1.8 Notas da rubrica

| Dimensão | Antes | Depois | Justificativa |
|---|---:|---:|---|
| Usabilidade | 2 | **2** | confirmado: modo ausente = falha silenciosa; 5–8 pacotes para paridade |
| API | 4 | **4** | confirmado: superfície pequena e tipada; desconto pela tripla config e por `dispatchCommand` em comandos de notificação |
| Neutralidade | 5 | **4** | **ajustada**. O âncora 5 exige "nenhum CSS/estrutura imposta"; o core impõe 3 estilos inline no root e `white-space` em todo texto exportado. Não é tema embutido, mas é CSS imposto e não-sobrescrevível sem `!important` |
| Modelo de dados | 5 | **5** | `NodeMap` imutável, seleção como dado, JSON determinístico |
| Extensibilidade | 4 | **4** | nós de primeira classe; desconto pela dívida de três sistemas |
| Performance | 4 | **4** | medições próprias com dispersão de ~5×, mas ordem de grandeza favorável |
| Acessibilidade | 3 | **3** | base sofisticada, tudo opt-in, ARIA do root é do host |
| Documentação | **4** | **4** | ver §2 abaixo (divergência com o ticket) |
| Manutenção | 3 | **3** | vivo, pré-1.0, HEAD à frente do npm |
| Adequação ao caso | 3 | **3** | especificação de design, não dependência |

Média recalculada: **3,6/5** (sem "adequação": 3,67). Antes: 3,7 / 3,78.

---

### 1.9 Outros erros encontrados na varredura final

- **`CREATE_COMMAND` não existe** em nenhum pacote (`grep -rn CREATE_COMMAND packages/*/src` →
  zero). O README §3 o citava como comando genérico; trocado por `dispatchCommand`.
- A matriz de usabilidade tem **15** linhas, mas o cabeçalho e o mapa do dossiê diziam "13 itens".
  Corrigido em `avaliacao.md` e `README.md` §12.
- `LexicalUpdates.ts:622-700` (commit/reconciliação) confere: `$commitPendingUpdates` em `:623`.
  `$generateHtmlFromNodes` de fato lança sem `document` (`lexical-html/src/index.ts:258-266`).

## 2. Divergência com a lista do ticket (documentação)

O ticket pedia `docs 3`; o dossiê auditado tem **4** e a evidência sustenta 4: 63 arquivos
`.md/.mdx`, 6 diretórios (`concepts` com 24 documentos, `extensions` com 10 incluindo
`migration.mdx`, `serialization`, `getting-started`, `react`, `collaboration`), mais
`upgrading.md`, `error.md`, `testing.md` e `concepts/compared-with-prosemirror.md`. O âncora 3
("cobre o básico") subestima isso; o desconto real (duas gerações de API sem "prefira este")
já está no comentário da linha. **Mantive 4.** Se a síntese exigir 3, é uma decisão editorial, não
uma correção de fato — o número 4 é o que a matriz de evidências suporta.

---

## 3. Correções aplicadas

| Arquivo | Correções |
|---|---|
| `README.md` | citação do probe trocada pela saída literal em português; nuance do keymap default; "100 testes" → "100 arquivos de teste"; `examples` 12+ → 24; changesets 133 → 130; alegação de adoção (Facebook/Workplace) marcada como não evidenciada no repo; refs de estilo inline `1748-1752` → `1749-1751` + `:1752`; ref do `white-space` de export; **remoção da falsa divergência de `registerHistory`** e substituição pela divergência verificada; números de performance em faixa; `CREATE_COMMAND` inexistente → `dispatchCommand`; matriz "13" → "15 itens" |
| `arquitetura.md` | campos de `LexicalNode` `1070-1085` → `1072-1085`; `getTransformSetFromKlass` `994-1135` → `933`/`:1102`; `storeDOMWithKey` corrigido para `LexicalReconciler.ts:2294` + `setNodeKeyOnDOMNode` `LexicalUtils.ts:689-707`; fast path `1805-1835` → `1816-1844`; prioridades `116-189` → `674-719`; linhas do `LexicalBuilder` (`addEdge :251`, conflitos `:300-321`, `constructEditor :177-208`, `buildEditor :210-231`); citação do probe em português; escopo do CSS (48 arquivos, nenhum em biblioteca); refs de estilo inline/export; tabela de performance em faixas |
| `avaliacao.md` | neutralidade 5 → 4 com contra-evidência; médias 3,7/3,78 → 3,6/3,67; "10 seções" → descrição real (6 diretórios, 63 arquivos); 133 → 130 changesets; performance em faixa; contagem de itens executados corrigida (8, não "6 = 8"); remoção da falsa divergência de `registerHistory`; item 10 explicitando o CSS inline; "13 itens" → 15 |
| `exemplos/README.md` | parágrafo de divergência npm × HEAD corrigido; nota de reexecução e faixa de tempos; esclarece que 01/02 reproduziram literalmente |
| `codigo/INDEX.md` | nomes reais dos trechos; seção de verificação de integridade (diff idêntico, intervalos, total de 853 linhas) |

---

## 4. Confiança

**87%.** O que sustenta: todas as afirmações centrais foram checadas contra o código-fonte com
`arquivo:linha`, as cópias foram validadas por `diff`, e os três probes foram reexecutados
(01/02 byte-idênticos à saída registrada). O que limita: nada foi executado em navegador real —
seleção, IME, clipboard, reconciliação de DOM e Yjs continuam avaliados só por código e pelos
testes do próprio repo; build/lint/vitest/Playwright do monorepo não rodaram (sem `node_modules`).

## 5. Lacunas (não executado)

- Navegador real: `contenteditable`, `beforeinput`, `MutationObserver`, seleção nativa, IME
  (iOS/Android/Firefox) e clipboard com `DataTransfer` verdadeiro.
- `pnpm install` + `vitest` (100 arquivos de teste do core), Playwright e2e e `pnpm bench`.
- Yjs/colaboração com `y-websocket` e dois clientes.
- `examples/vanilla-js` com Vite (código lido, não rodado).
- Leitor de tela (NVDA/VoiceOver) para conferir os anúncios de `@lexical/a11y`.
- Não avaliei `lexical-devtools`, `lexical-eslint-plugin`, `lexical-mdast` nem `lexical-compiler`.

## 6. Pendências menores conhecidas (não corrigidas)

- `avaliacao.md` cita `LexicalNode.ts:1768-1810` para `DecoratorNode` + `NodeState`; não
  reconferi essa faixa linha a linha.
- A tabela de pacotes em `arquitetura.md` §1 é ilustrativa (`…`), não exaustiva; `ls packages`
  dá 40.
- Os números de tamanho (`4,3 MB`, `6,6 MB`) foram medidos com `du -sh` e conferem.
