# Avaliação — Quill 2.0.3 como referência para o `rich-editor-auy`

Notas na escala 0–5 de [`dossie/_RUBRICA.md`](../_RUBRICA.md), sempre com evidência. A
pergunta central — **quão fácil é usar Quill sem interferir no CSS?** — é respondida na
[§4](#4-css-neutralidade-resposta-direta).

Base de evidência: leitura do código em `ref/quill/packages/quill/src` (commit `539cbff`),
testes em `test/unit` e `test/e2e`, e **execução real** do build publicado 2.0.3 (mesma versão)
em Chromium 156 headless com Playwright. Saídas em
[`exemplos/saida-probe.json`](exemplos/saida-probe.json),
[`exemplos/saida-interact.json`](exemplos/saida-interact.json),
[`exemplos/saida-delta.json`](exemplos/saida-delta.json) e
[`exemplos/saida-minimal-dom.html`](exemplos/saida-minimal-dom.html). Execução contra o fonte
local **não** foi feita: o monorepo não tem `node_modules` e o build exige webpack/Babel;
declaro isso em [`exemplos/README.md`](exemplos/README.md).

---

## 1. Notas por dimensão

| Dimensão | Nota | Evidência | Comentário |
|---|---:|---|---|
| **Usabilidade de integração** | **4** | `exemplos/minimal.html` + `exemplos/saida-minimal-dom.html`: init em uma chamada, toolbar por seletor, conteúdo inicial convertido; `src/core/quill.ts:198-233` | Init é realmente simples e erros de container/tema são explícitos (`:798-802`). O atrito aparece depois: é preciso entender Delta, registry e o CSS estrutural. Não é "1 linha e pronto" para um editor de produto. |
| **API** | **4** | `src/core/quill.ts:304-765` (métodos de documento), `:925-1007` (`overload`), `test/types/quill.test-d.ts`, `src/quill.ts:120-135` (exports) | API pequena e tipada, com sobrecargas para `(index,length)`/`Range`/`(format,value)`. O `overload` é complexo e algumas assinaturas aceitam combinações difíceis de ler; pacote ESM com tipos. Não há validação de schema. |
| **Neutralidade de estilo** | **2** | `src/assets/core.styl:12-18`, `:40-110`, `:208-215`; `snow.styl:8-10`, `bubble.styl:8-10`; `webpack.common.cjs:41-43` | Existe separação core/tema, mas **não é zero-CSS**: listas, numeração, placeholder e `pre-wrap` vêm de `core.styl`. Substituir exige reimplementar seletores e DOM. `snow`/`bubble` ainda injetam chrome (tooltip/pickers) no container. |
| **Modelo de dados** | **4** | `codigo/05-quill-delta.ts:69-171`, `:236-545`; `src/core/editor.ts:162-166`; `src/blots/block.ts:19-23`, `:180-193` | Delta plano, serializável, com `compose/transform/invert/diff` e semântica clara de atributos (`null` remove). Falta schema/validação: o construtor assume ops bem formados (`05-quill-delta.ts:59`) e qualquer atributo é aceito. |
| **Extensibilidade** | **4** | `src/core/quill.ts:127-180`; `src/core/theme.ts:30-52`; `src/core/utils/createRegistryWithFormats.ts:5-40`; observado `customFormat` em `saida-probe.json`; `src/modules/syntax.ts:14-143`; `src/modules/tableEmbed.ts:259` | Formatos, blots, módulos, temas e handlers são registráveis sem fork; `formats: [...]` restringe o registry e resolve `requiredContainer`. Limite: não há ciclo de vida declarativo de plugin nem schema de atributos; dados derivados (syntax) precisam de filtro manual. |
| **Performance** | **4** | `saida-probe.json` (`bench`): 1.000 parágrafos ≈37 ms, `getContents` ≈0,1 ms, 200 inserts ≈335 ms; `saida-interact.json`: 120 teclas ≈247 ms; `src/core/editor.ts:279-315`; `src/blots/block.ts:17-90` (`cache`) | Bom para documentos médios e grandes-médios: carga de 48.890 caracteres em ~37 ms, leitura em ~0,1 ms, digitação no caminho rápido (`characterData`) e cache por bloco. O ponto fraco é o padrão patológico (200 `insertText` de API no início = O(n) cada, ~1,7 ms/insert) e a ausência de virtualização; sem benchmark oficial nem teste de estresse (nota 5 exigiria isso). |
| **Acessibilidade** | **3** | `src/modules/toolbar.ts:34`, `:166-200` (com teste `test/unit/modules/toolbar.spec.ts:180-193`); `src/ui/picker.ts:45-108` (com 6 testes de ARIA em `test/unit/ui/picker.spec.ts:44-129`); `src/modules/uiNode.ts:8-100`; `saida-probe.json`: `rootRole=null`, `rootAriaLabel=null` | Chrome (toolbar/pickers) tem ARIA real **e testado**: `role=toolbar`, `aria-label`/`aria-pressed` sincronizados com a seleção, `role=button`/`aria-expanded`/`aria-hidden`/`aria-controls`, Enter/Escape no picker, correção de caret em `.ql-ui`. O que falta é o nó editável: sem `role`/rótulo, sem `aria-live`, checkbox de lista sem `role=checkbox`/`aria-checked`, placeholder só `::before`, e nenhum teste com leitor de tela. É "contenteditable básico" com bom chrome — 3, não 4. |
| **Documentação** | **4** | `ref/quill/packages/website/` (site/docs versionados no mesmo monorepo); `CHANGELOG.md:1-45`; tipagem publicada (`test/types/quill.test-d.ts`, `src/types.d.ts`); comentários de código (ex.: `src/core/composition.ts:24-26` cita o bug do WebKit) | Documentação de uso é boa e versionada com o código: guias/API no pacote `website`, CHANGELOG detalhado, `.d.ts` e sobrecargas que documentam a API. O README da biblioteca é curto e não há guia de migração 1.x→2 no repositório; não auditei página por página do site, o que impede 5. |
| **Manutenção** | **4** | `CHANGELOG.md:1-45`; `.github/workflows`; `test/unit/vitest.config.ts:22-27`; `test/e2e`; build em `scripts/build` | Projeto ativo, 2.0 GA em 2024, releases 2.0.1–2.0.3, testes em browser real, E2E e fuzz, CI. Sem LTS nem política de depreciação formal; mantido principalmente pela Slab. |
| **Adequação ao `rich-editor-auy`** | **3** | Delta (`codigo/05-quill-delta.ts`), blots (`src/blots/*`), dependência de `document` (`src/core/selection.ts`, `src/blots/scroll.ts`), CSS estrutural (`core.styl`) | Como **modelo** é referência forte (ops, composição, inverso, projeção). Como **biblioteca** não encaixa: é DOM-bound, não é headless, exige contenteditable e CSS. Serve para copiar semântica, não arquitetura de execução. |

**Média simples:** 3,6/5. Para o nosso caso, o valor está no modelo (Delta) e na disciplina de
separar core CSS de tema, não na stack de execução. A média subiu de 3,3 na revisão porque
performance, acessibilidade e documentação estavam subestimadas (ver `revisao.md`); a nota de
neutralidade de estilo continua 2 e é justificada na §4.

---

## 2. Matriz de usabilidade (meta ≥85%)

Status: `exec` = executado e observado; `teste` = verificado em teste do repositório;
`código` = verificado no código; `doc` = documentação; `não` = não verificado.

| # | Item | Status | Evidência |
|---|---|---|---|
| 1 | Instalação e pré-requisitos | **exec + doc** | CDN 2.0.3 carregado em Chromium; `exemplos/saida-minimal-dom.html`; npm `quill` e CSS separados (`package.json`, `webpack.common.cjs:38-44`). Não executei `npm install` do monorepo (sem dependências). |
| 2 | Inicialização mínima | **exec** | `new Quill('#editor',{theme:'snow',modules:{toolbar:'#toolbar'}})`; DOM resultante com `ql-container ql-snow`, `div.ql-editor[contenteditable=true]` (`saida-minimal-dom.html`). |
| 3 | Edição básica (texto, parágrafo, negrito/itálico/link) | **exec** | Digitação real, Enter, `Ctrl+B`, clique em toolbar e format via API (`saida-interact.json`: `typing`, `enter`, `boldShortcut`, `toolbarClick`; `saida-probe.json`: `afterEditsDelta`). Link: sanitização e formato `link` em `afterEditsHTML`/`urlSanitizedHTML`; clique no botão de link não foi exercitado (usa `prompt`/tooltip). |
| 4 | Seleção, teclado, undo/redo, clipboard, IME | **exec + teste** | `Ctrl+Z`/`Ctrl+Shift+Z` (`saida-interact.json`: `undo`, `redo`); seleção com Shift+setas (`boldShortcut`); colar texto e HTML (`pastePlain`, `pasteHTML`); IME via CDP (`imeDuringComposition`, `imeCommitted`); testes de browser em `test/unit/core/selection.spec.ts` e `test/e2e/history.spec.ts`. |
| 5 | Schema e restrições de conteúdo | **código** | `formats` restringe o registry e resolve containers (`createRegistryWithFormats.ts:5-40`); **não há schema de atributos nem validação de ops** (`05-quill-delta.ts:59`). Verificado no código, não por execução de caso inválido. |
| 6 | Custom node/mark/plugin | **exec + código** | Attributor registrado em runtime gerou `highlight` no Delta e `class="ql-highlight-true"` no DOM (`saida-probe.json`, `customFormat`). Módulos/temas: `theme.addModule` e `Quill.register` (`quill.ts:127-180`; `theme.ts:30-52`). Custom **blot de nó** (embed próprio) não foi executado; apenas lido (`tableEmbed`, `syntax`). |
| 7 | Persistência e serialização | **exec** | `getContents`/`setContents` em `minimal.html` e `probe.html`; Delta inicial (`loadDeltaOfInitialHTML`) e round-trip via `setContents` (`minimal.html`). `getSemanticHTML` observado com `&nbsp;` e tags canônicas (`afterEditsHTML`). |
| 8 | Renderização e não interferência no CSS | **exec + código** | CSS computado: `white-space: pre-wrap`, `font-family: Helvetica`, `font-size: 13px`, `p{margin:0}` (`saida-probe.json`, `css.editor`); listas sem marcador nativo (`li{list-style:none}` + `.ql-ui:before`, `core.styl:71-110`), `ol { padding-left: 19.5px }` (`css.list`). O campo `noCss.*` do probe **não** mede ausência de CSS (a página carrega `quill.snow.css` e o segundo editor herda `.ql-editor`); substituição total não foi executada — **lacuna registrada** (§4 e §5). |
| 9 | Acessibilidade | **código + teste + exec parcial** | Toolbar `role=toolbar`, `aria-label/pressed` (`toolbar.ts:34`, `:166-200`; teste `toolbar.spec.ts:180-193`); picker ARIA **com testes dedicados** (`picker.ts:45-108`; `picker.spec.ts:44-129`); teclado/`.ql-ui` (`uiNode.ts:8-100`); raiz **sem** role/label (`saida-probe.json`); sem teste com leitor de tela. |
| 10 | Colaboração | **código** | Apenas primitivas OT: `transform`/`invert`/`transformStack` (`history.ts:139-180`; `05-quill-delta.ts:236-545`). Sem protocolo, presença ou cursores; **não executado** nenhum cenário colaborativo. |
| 11 | Performance e limites | **exec** | `bench` em `saida-probe.json`; `typing120` em `saida-interact.json`; mecanismos em `editor.ts:279-315`. Sem teste de estresse/virtualização. |
| 12 | Erros, migração e diagnóstico | **código + doc** | Logger com níveis (`src/core/logger.ts:1-30`; `Quill.debug`); erros explícitos de container/tema/blot (`quill.ts:798-802`, `:218-222`); `CHANGELOG.md:1-45` cobre 2.0 → 2.0.3. Migração 1.x→2 **não** foi executada nem auditada. |
| 13 | Licença e maturidade | **doc** | Quill BSD-3-Clause (`ref/quill/LICENSE:1-12`, `package.json:20`); `quill-delta` **BSD-3-Clause** (tarball 5.1.0, `package/LICENSE`, (c) 2022 Slab) — o dossiê dizia MIT antes da revisão; 2.0.3; releases 2024; CI. |

**Cobertura:** 13/13 itens com evidência registrada; 8 itens com execução real plena (1, 2,
3, 4, 6, 7, 8, 11) e 1 com execução parcial (9, a11y: atributos medidos no probe, mas sem
leitor de tela); 4 com evidência só de código/teste/doc (5, 10, 12, 13). Pelo critério do guia
(≥85% da matriz com evidência), **13/13 = 100% com evidência**, sendo **9/13 com alguma
execução**. Se "executado" for exigido para todos, a cobertura cai para ~69% e o dossiê
declara as lacunas na §5.

---

## 3. O que o usuário real vê (síntese executada)

- **Init**: uma chamada; toolbar declarativa ou por HTML; o container é esvaziado e o conteúdo
  inicial é convertido para Delta (`quill.ts:208-211`, `:265-272`). O DOM final tem
  `contenteditable="true"` e um `ql-tooltip` oculto injetado (`saida-minimal-dom.html`).
- **Primeira edição**: digitar funciona; `text-change` entrega `Delta`. Em 120 teclas, ~247 ms
  no ambiente de teste (`saida-interact.json`).
- **Formatação**: `Ctrl+B`, clique na toolbar e `quill.format` convergem para o mesmo Delta
  (`boldShortcut`, `toolbarClick`, `afterEditsDelta`).
- **Enter**: novo parágrafo com formatos de bloco herdados (`handleEnter`,
  `keyboard.ts:334-352`; observado `enter`).
- **Undo/redo**: `Ctrl+Z` restaura; `Ctrl+Shift+Z` refaz; agrupamento por 1 s
  (`history.ts:111-137`; observado `undo`/`redo`).
- **Colar**: texto simples e HTML externo viram Delta com `bold`, `italic`, `list`
  (`pastePlain`, `pasteHTML`); URLs perigosas viram `about:blank` e imagens inválidas `//:0`
  (`urlSanitizedHTML`, `link.ts`, `image.ts`).
- **IME**: durante a composição o Delta fica congelado; ao commitar, entra o texto final
  (`imeDuringComposition` → documento vazio, `[{"insert":"\n"}]`; `imeCommitted` →
  `[{"insert":"日本\n"}]`). Não há `initial` no cenário: o script limpa o editor antes do IME.
- **readOnly**: teclado não altera; API altera (`readOnlyTyping`, `readOnlyApiStillWorks`);
  `contenteditable` vira `"false"` (`scroll.ts:98-100`).
- **Autofill de lista**: `- ` no início do parágrafo vira `list: bullet`
  (`listAutofill`; binding em `keyboard.ts:550-600`).

---

## 4. CSS-neutralidade: resposta direta

**Quão fácil é usar Quill sem interferir no CSS? Não é fácil, e "sem CSS" não é uma opção
suportada.** O mínimo viável é `quill.core.css` + o CSS do produto; trocar o core por CSS
próprio é possível, mas é **reimplementação**, não neutralidade.

Evidência:

1. `core.styl` define o contrato estrutural: `.ql-editor { white-space: pre-wrap; outline:
   none; padding; overflow-y }` (`src/assets/core.styl:40-53`); `li { list-style-type: none;
   padding-left; position: relative }` e numeração por `.ql-ui:before` + counters
   (`:71-110`); placeholder por `.ql-blank::before` (`:208-215`). Sem isso, o editor continua
   editável, mas listas, indentação e espaços degradam.
2. `snow.css` e `bubble.css` **importam** core e base (`snow.styl:8-10`, `bubble.styl:8-10`),
   e o build publica `quill.core.css` separado (`webpack.common.cjs:41-43`). Ou seja: existe
   um "mínimo" empacotado, mas ele já impõe classes, seletores e a estrutura
   `p / ol > li[data-list] / div.ql-code-block`.
3. O tema injeta DOM: `ql-tooltip` e pickers vivem dentro do container
   (`src/themes/snow.ts:106-125`; observado em `saida-minimal-dom.html`). Isso afeta layout e
   CSS do app, mesmo que você não use a toolbar visual.
4. Medições de CSS aplicado: `white-space: pre-wrap`, `font-family: Helvetica, Arial,
   sans-serif`, `font-size: 13px`, `p { margin: 0 }` (`saida-probe.json`, `css.editor`). São
   decisões do Quill, não do seu design system; para mudá-las é preciso sobrescrever as
   classes ou reescrever a folha.

O que **funciona bem**: separar `quill.core.css` (estrutural) de um tema próprio; usar
`theme: null`/default e não carregar `snow`/`bubble`; mapear os seletores estruturais
(`.ql-editor`, `.ql-ui`, `li[data-list]`, `.ql-indent-N`, `.ql-code-block-container`) para o
seu CSS. O que **não funciona**: esperar que o Quill não toque no estilo do documento ou que
`contenteditable` renderize listas corretamente sem CSS.

**Nota de neutralidade: 2/5.** Para o `rich-editor-auy` (zero CSS por princípio), o valor não
é usar Quill, e sim copiar a ideia de "invariantes estruturais explícitos + tema separado",
com os invariantes no renderer, não numa folha global.

**Por que 2 e não 3 (nem 1).** A nota 3 da rubrica é "permite trocar, mas interfere". Quill
chega perto disso: `theme: 'default'` não injeta toolbar/tooltip, existe `quill.core.css`
separado e o app pode sobrescrever seletores. Mas a troca **não é suportada**: sem
`white-space: pre-wrap`, `li { list-style: none }` + counters e `.ql-ui:before`, o documento
renderiza errado (listas sem marcador, espaços colapsados, placeholder ausente), e os nós
`.ql-ui`/`li[data-list]`/`ql-code-block-container` são produzidos pelos blots, não pelo CSS.
Ou seja, "trocar o CSS" exige reescrever contra uma estrutura DOM que o Quill impõe — é
reimplementação, não neutralidade. A nota 1 ("exige CSS/tema" no sentido de que nada funciona
sem ele) seria severa demais: o texto continua editável só com CSS mínimo, e há um artefato
oficial para esse mínimo. 2 é o ponto correto: **traz um tema/estrutura pesada por padrão,
mas há um caminho documentado de mínimo**. Em nenhuma leitura Quill é "zero CSS".

---

## 5. Lacunas e o que não foi verificado

- **Fonte local não executada**: o monorepo não tem `node_modules`; não rodei build, lint nem
  Vitest. A execução usou o pacote publicado **2.0.3** (mesma versão declarada em
  `ref/quill/packages/quill/package.json:3`), via CDN. Diferença possível entre fonte e dist
  não foi auditada.
- **Navegadores**: só Chromium 156 headless. Firefox/Safari têm caminhos específicos
  (`navigator.platform`, `isComposing`, microtask WebKit) e não foram exercitados.
- **IME real**: simulei composição via CDP (`Input.imeSetComposition`/`insertText`), como os
  próprios testes E2E do Quill (`test/e2e/fixtures/Composition.ts`). Não usei um IME de
  sistema (japonês/chinês) nem teclado móvel.
- **A11y real**: não rodei leitor de tela (NVDA/VoiceOver) nem auditoria axe; a nota 3 é
  baseada em atributos observados, nos testes de ARIA de `picker.spec.ts`/`toolbar.spec.ts`
  e nas ausências do nó editável.
- **Colaboração**: não executada; só primitivas de OT verificadas no código.
- **Tabelas, upload/drag-and-drop, syntax highlight, temas bubble**: lidos no código, não
  exercitados na execução.
- **Migração 1.x → 2.x**: não executada; apenas CHANGELOG lido.
- **Substituição total de CSS**: não montei uma página só com CSS próprio; a conclusão da §4
  vem do código (`core.styl`) e das medições de CSS aplicado. A degradação sem CSS é
  **inferida** da leitura (não medida).
- **Performance em documentos grandes**: 1.000 parágrafos é média, não estresse; sem
  virtualização, o comportamento em 50k+ blocos não foi medido. A nota 4 se apoia nas
  medições de 1.000 parágrafos e no caminho rápido de digitação, não em tuning executado.
- **Revisão independente (2026-11-07)**: as cópias foram conferidas por `diff` contra a fonte
  (idênticas após o cabeçalho); `delta-steps.cjs` foi reexecutado e reproduziu
  `saida-delta.json` byte a byte; a licença do `quill-delta` foi corrigida para BSD-3-Clause;
  e as citações de linha de `chop`/`length`/`changeLength`/construtor foram corrigidas. Ver
  [`revisao.md`](revisao.md).

**Confiança final: 87%.** Código central lido integralmente (editor, scroll, selection,
history, clipboard, blots, formatos e CSS), comportamento executado nos eixos de init,
edição, seleção, undo, clipboard, IME, readOnly e serialização. As lacunas acima impedem
100%, principalmente acessibilidade real, navegadores não-Chromium e colaboração.
