# Revisão independente — dossiê `tiptap`

Revisor: rodada extra, independente do explorador. Escopo: `dossie/tiptap/` contra `ref/tiptap/`.
Nada fora de `dossie/tiptap/` foi alterado (`ref/`, `plan/`, `AGENTS.md` e outros dossiês intactos).

Método: (a) leitura integral dos quatro documentos do dossiê; (b) verificação de **161 citações
`arquivo:linha`** contra a árvore real; (c) `diff` de todas as cópias de `codigo/` contra a origem;
(d) **reexecução** dos experimentos em Chromium headless, com a versão **fixada** em
`@tiptap/core@3.30.3` (o dossiê original usava `@3`, que hoje resolve 3.33.0); (e) sondas novas
para os pontos que a revisão exigiu: `injectCSS:false`, reuso da style tag, ordem de classes,
semântica de `chain()`, estilo inline de extensões.

---

## 1. Achados

### A1 — Crítico: a prova de `injectCSS: false` era inválida (conclusão certa, evidência não)

O dossiê afirmava "opt-out verificado: segundo editor com `injectCSS:false` não adicionou style tag
(`styleTagsBefore === styleTagsAfter === 1`)". Isso **não prova nada**: `createStyleTag` começa com
`document.querySelector('style[data-tiptap-style]')` e **devolve a tag existente**
(`packages/core/src/utilities/createStyleTag.ts:2-8`). Um segundo editor **com** `injectCSS:true`
também deixaria a contagem em 1. O teste era confundido pelo próprio mecanismo que se queria medir.

**Reexecutado corretamente** (documento isolado, um único editor, em iframe com `<head>` próprio):
`injectCSS: false` ⇒ **0** nós `style`/`link`, editor funcional
(`selectAll`+`toggleBold` ⇒ `<p><strong>sem css</strong></p>`). Arquivo novo e reprodutível:
`exemplos/cdn/isolated-nocss.html`; comando e saída em `exemplos/README.md` §6.1.
A conclusão do dossiê se mantém; a evidência agora sustenta.

### A2 — Crítico: identidade da fonte estava errada (afeta toda citação)

O dossiê chamava `ref/tiptap/` de "monorepo **3.30.3**". O checkout **não é a tag 3.30.3**: é
`main` no commit `5fd5d2f` (2026-11-05), sem tags locais (`git describe` falha), com **90 changesets
não publicados** e `packages/*/package.json` ainda em 3.30.3. Ou seja: as citações `arquivo:linha`
descrevem o **HEAD em desenvolvimento**, enquanto as execuções via CDN descrevem o **artefato
publicado** — builds diferentes.

Isso não é teórico: produziu um erro de atribuição (A3). Corrigido com um bloco de identidade no
topo do `README.md`, na §6 e no §13 do `arquitetura.md`.

### A3 — Errado: a "divergência 3.30.3 vs 3.33.0" de `isDestroyed`

O dossiê dizia que em 3.33.0 `isDestroyed` vem `true` após `unmount()`, enquanto o código de 3.30.3
garantiria o contrário (`return this.destroyed`). **Testei as duas versões publicadas** e ambas dão
`true`:

| Build | `get isDestroyed()` | após `unmount()` |
|---|---|---|
| `@tiptap/core@3.30.3` (esm.sh) | `editorView?.isDestroyed ?? true` | `true` |
| `@3` = 3.33.0 (esm.sh) | idem | `true` |
| `ref/tiptap` (main, `5fd5d2f`) | `this.destroyed` (`Editor.ts:901-903`) | `false` |

A divergência real é **HEAD vs. publicado**, não entre duas releases. Reescrito em
`arquitetura.md` §13 (item 8) e `avaliacao.md` §5 (item 4).

### A4 — Errado: linha do default de `injectCSS` e de `enableContentCheck`

- `injectCSS: true` está em `Editor.ts:100`, não em `112`.
- `enableContentCheck: false` está em `Editor.ts:112`, não em `122`.

Os dois estavam deslocados por um bloco de ~12 linhas. Corrigidos.

### A5 — Quatro intervalos de linha passavam do fim do arquivo

| Arquivo | Citado | Real |
|---|---|---|
| `helpers/getSchemaByResolvedExtensions.ts` | `24-45` | `19-41` (41 linhas) |
| `helpers/resolveExtensions.ts` | `14-28` | `13-25` (25 linhas) |
| `helpers/sortExtensions.ts` | `13-30` | `13-25` (26 linhas) |
| `extensions/clipboardTextSerializer.ts` | `25-60` | `25-53` (53 linhas) |

Também corrigido o `RangeError` de schema duplicado: linhas **32-34**, não 30-32.

### A6 — Impresso: `chain()` devolve AND, e o dossiê descrevia o efeito pela metade

O dossiê dizia "`chain().run()` pode devolver `false` mesmo tendo mudado o foco". Testado:
`chain().focus().undo().run()` com histórico vazio → `false`; e, mais forte,
`chain().selectAll().toggleBold().undo().run()` → **`false`** com o `<strong>` **já aplicado** no
documento. A formulação correta é "o retorno é o AND dos comandos; efeitos parciais acontecem".
Corrigido no `README.md` §3 e na linha de API da tabela.

### A7 — Faltava o detalhe que mais importa para "zero CSS": o estilo inline observado

O dossiê citava `text-align`/`background-color` como leitura de código. Executei: com
`TextAlign`, `<p style="text-align: center;">centro</p>` **e** `attrs.textAlign` no JSON —
as duas representações ao mesmo tempo. `Highlight` sem `multicolor` **descarta** `data-color` de
entrada (nuance que o dossiê não registrava); a emissão de `background-color` só ocorre com
`multicolor: true` e foi lida no código, não executada.

### A8 — Faltava o reuso da style tag, que explica o comportamento de múltiplos editores

O documento tem **no máximo uma** `<style data-tiptap-style>`; `createStyleTag` a reaproveita e
`unmount()` só a remove quando não resta nenhum `.tiptap` (`Editor.ts:236-250`). Registrado em
`README.md` §2 e `arquitetura.md` §5.

### A9 — Ordem das classes (refinamento, não erro)

`prependClass()` concatena `tiptap` **à frente** do `className` existente (`Editor.ts:698`).
Observado: `editorProps.attributes.class = 'my-wrap'` ⇒ `tiptap ProseMirror my-wrap`.
O dossiê só citava `tiptap ProseMirror`. Registrado, junto com o fato de a classe ser imposta e
**não** desligável (nem com `injectCSS:false`).

### A10 — Confirmados sem alteração (auditados, não reescritos)

- "Headless" = **sem UI**, não sem DOM nem sem CSS: correto e bem argumentado.
- `role="textbox"` sempre presente, `tabindex` só quando editável, `aria-multiline` **ausente** por
  default — reconfirmado por execução; o demo `Accessibility` seta `aria-multiline: 'true'` à mão
  (`demos/src/Examples/Accessibility/React/index.tsx:20-26`).
- `style.ts` ⇒ **1329 caracteres** injetados. Conferido por execução **e** por aritmética: 1353
  bytes do arquivo − `export const style = \`` − crase final − `\n` = 1329.
- `mergeAttributes`: `class` concatenada sem duplicar; `style` fundida por propriedade, último
  vence; `__proto__` não polui. Reexecutado também contra o build publicado.
- 19 opções de runtime: contadas no arquivo (`getEditorRuntimeOptions.ts:3-23`) — 19 exatas.
- 325 arquivos de teste: contados (`find packages demos -name '*.test.ts' -o -name '*.spec.ts' …`).
- 61 pacotes, 90 changesets, CHANGELOG de 455.9 KB: conferidos.
- Code block é a **única** classe default de node: `<pre><code class="language-js">` ida e volta.
- Cópias de `codigo/`: `diff` confirma **verbatim** para os três arquivos íntegros
  (`CommandManager.ts`, `style.ts`, `bold.tsx`) e para `experimentos/mergeAttributes.ts`. Os três
  trechos batem com os intervalos declarados (diferença: uma linha em branco separadora após o
  cabeçalho).

---

## 2. Correções aplicadas

| Arquivo | Correção |
|---|---|
| `README.md` | Bloco de identidade da fonte (checkout é `main`, não a tag); `injectCSS` default em `Editor.ts:100`; reuso da style tag; ordem das classes + classe não removível; reescrita da prova de zero-CSS com o experimento isolado e o confundidor explicitado; estilo inline **observado**; semântica real de `chain().run()`. |
| `arquitetura.md` | Intervalos `19-41`, `13-25`, `13-25`, `25-53`; `RangeError` em 32-34; novo bullet "Raiz e `<style>`" (reuso, `injectCSS`, `nonce`, `suffix`, remoção no `unmount`); §13 item 8 reescrito como "deriva checkout-vs-publicado" com a tabela dos três builds. |
| `avaliacao.md` | Linha de API (AND de `chain`); linha de neutralidade (opt-out verificado em isolamento, classe imposta, inline observado); item 2 e item 8 da matriz; linha de manutenção (versão declarada vs. checkout); §5 item 4 reescrito; nota sobre o alcance da execução; **nova §6 "Veredito"**. |
| `codigo/INDEX.md` | `core-style.ts`: distingue 1353 bytes do arquivo e 1329 caracteres injetados, e como o número foi conferido. |
| `exemplos/README.md` | Lista de arquivos atualizada; linha do `noCss` reclassificada como **inconclusiva** com a explicação; **nova §6** com as 8 sondas da revisão, comandos e saídas verbatim. |
| `exemplos/cdn/isolated-nocss.html` | **Novo**: prova reprodutível de `injectCSS:false` em documento isolado (Tiptap fixado em 3.30.3). |

---

## 3. Notas da rubrica — mantidas, com a justificativa

| Dimensão | Nota | A evidência sustenta? |
|---|---|---|
| Usabilidade | **4** | Sim. Init mínimo e API clara; o atrito está em descobrir `isActive`/`can`/`useEditorState` sozinho. |
| API | **4** | Sim. Tipada e coerente; `setOptions` parcial e o `run()` que devolve AND impedem 5. |
| Neutralidade de estilo | **4** | **Só sob a régua "não impõe tema nem UI".** Sob a nossa régua (zero CSS/classe/inline) é **3**. O texto agora declara as duas leituras. |
| Modelo de dados | **5** | Sim. JSON do ProseMirror, schema derivado, interop com `schemaSpec` externo. |
| Extensibilidade | **5** | Sim. Melhor eixo do projeto; verificado com nó `callout` criado e com `bold.tsx` real. |
| Performance | **3** | Sim, com a ressalva que o dossiê já faz: custo dominante é ProseMirror/DOM e **não há medição**. |
| Acessibilidade | **3** | Sim. `role`/`tabindex` automáticos, `aria-multiline` e todo o resto por conta do integrador. |
| Documentação | **3** | Sim. Docs reais fora do repo; demos e testes fazem o papel. |
| Manutenção | **3** | Sim. 61 pacotes, churn de nomes, 90 changesets abertos, sem tag no checkout. |
| Adequação | **3** | Sim. É referência de **API**, não núcleo: modelo a copiar, defaults a inverter. |

Média simples: **3,7/5** (inalterada).

---

## 4. Veredito

**É a melhor referência de DX de extensão deste conjunto de estudos, mas não é zero estilo por
default — e serve como API, não como core.**

- A força é o contrato: uma extensão = schema + DOM + atributos + comandos + atalhos + regras +
  plugins + views + hooks, com `commands`/`chain()`/`can()` transacionais e tipagem por
  augmentação de módulo.
- A fraqueza, para nós, é a política de estilo: `injectCSS:true` por default (1329 chars), classe
  `tiptap` imposta e não removível, e estilo inline como default de extensões (`text-align`,
  `background-color`). "Zero CSS" existe, mas é opt-in e parcial.
- Consequência de adoção: copiar o **desenho** (extensão como unidade, comandos transacionais,
  `can` sem despacho, `renderHTML`/`parseHTML` por atributo, `mergeAttributes`) e **inverter os
  defaults** — atributo semântico no documento, CSS no front-end, nada injetado no `<head>`,
  classe imposta opcional.

---

## 5. Confiança final: **88%**

Justificativa (régua da rubrica: 75–89% ⇒ código e docs fortes, execução parcial):

- **A favor:** todas as 161 citações foram resolvidas contra arquivos existentes e conferidas
  quanto a faixa; as cópias foram validadas por `diff`; os experimentos centrais (init, comandos,
  `isActive`/`can`, undo/redo, nó custom, serialização, CSS default, **opt-out em isolamento**,
  classes, `chain`, `mergeAttributes`, code block) foram **executados** e as saídas estão coladas;
  a única divergência código↔runtime foi encontrada e explicada.
- **Contra (lacunas que impedem >90%):** nada de IME real, colagem/arrastar, Yjs, mobile ou leitor
  de tela; **zero números de performance**; o monorepo não foi construído (Node 22.23.2 vs
  `engines.node >= 24`); `@tiptap/html/server`, `static-renderer` e `@tiptap/markdown` não foram
  executados.

## 6. O que não foi verificado (e por quê)

| Item | Motivo |
|---|---|
| `pnpm install` / `build` / testes do monorepo | sem `node_modules`; `engines.node >= 24` e a máquina tem 22.23.2 |
| Testes Playwright de `demos/**/*.spec.ts` | exigem o monorepo construído |
| IME real (composição asiática, teclado virtual) | só leitura de código (`InputRule.ts:324-338`, `view.composing`) |
| Colagem e arrastar-e-soltar | só leitura de código (`clipboardTextSerializer.ts`, `paste.ts`) |
| Colaboração Yjs | exigiria `Y.Doc` + provider |
| Leitor de tela / a11y assistiva real | sem ambiente assistivo |
| Performance (tempo, memória) | nenhuma medição; observações são estruturais |
| `Highlight` com `multicolor: true` emitindo `background-color` | lido no código, não executado |
| `@tiptap/html/server`, `static-renderer`, `@tiptap/markdown` | fora do escopo mínimo da revisão |
