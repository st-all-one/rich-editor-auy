# `exemplos/` — o que foi executado e o que não foi

Nada aqui depende do monorepo estar instalado: os exemplos carregam o Tiptap **3** publicado via
CDN ESM (`esm.sh`). Isso é honesto quanto à versão (pode divergir do HEAD de `ref/tiptap`) e
suficiente para verificar comportamento de API.

```
exemplos/
  cdn/index.html                 # smoke test mínimo (EXECUTADO)
  cdn/probe.html                 # sonda maior: teclado, input rules, nó custom, injectCSS:false (PARCIAL)
  cdn/isolated-nocss.html        # injectCSS:false em documento isolado (EXECUTADO na revisão, §6)
  minimal.mjs                    # exemplo mínimo canônico com npm install (NÃO EXECUTADO)
  experimentos/mergeAttributes.ts# cópia verbatim de core/src/utilities/mergeAttributes.ts
  experimentos/run-merge.mjs     # exercita a cópia acima (EXECUTADO)
```

---

## 1. `experimentos/run-merge.mjs` — EXECUTADO

Testa a função real de merge de atributos que toda extensão usa em `renderHTML`.
A cópia em `experimentos/mergeAttributes.ts` é idêntica a
`ref/tiptap/packages/core/src/utilities/mergeAttributes.ts` (MIT), sem dependências.

```bash
cd dossie/tiptap/exemplos/experimentos
node --version          # v22.23.2
node run-merge.mjs
```

Saída observada (verbatim):

```
v22.23.2
merged = {"class":"prose p-4 custom","style":"color: blue; margin: 0","data-id":"x"}
obj.polluted = undefined
own keys     = [ '__proto__' ]
hasOwn __proto__ = true
```

Leitura:
- `class` é **concatenada sem duplicar** (`prose` aparece uma vez) — `mergeAttributes.ts:101-111`.
- `style` é fundida **por propriedade**, último vence (`color: blue`), e a ordem é preservada
  (`color` antes de `margin`) — `mergeAttributes.ts:112-120`.
- `__proto__` vindo de JSON **não** polui `Object.prototype` (virou propriedade própria) —
  `mergeAttributes.ts:81-91`. Observação: ele fica **enumerável** em `Object.keys`, o que pode
  surpreender quem serializa o resultado.

Node 22 roda `.ts` por *type stripping* nativo; não foi instalado nada.

---

## 2. `cdn/index.html` — EXECUTADO em Chromium headless

Serve o diretório e abre a página; o resultado fica em `window.__RESULT__`.

```bash
cd dossie/tiptap/exemplos
python3 -m http.server 8731 --bind 127.0.0.1
# abrir http://127.0.0.1:8731/cdn/index.html e ler window.__RESULT__ no console
```

Executado aqui via Chromium headless controlado por Playwright, lendo `window.__RESULT__` por
`evaluate`. Resultado (verbatim, reordenado só para leitura):

```json
{
  "steps": ["imports-ok", "editor-created", "destroyed"],
  "error": null,
  "dom": {
    "tagName": "DIV",
    "className": "tiptap ProseMirror",
    "contentEditable": "true",
    "role": "textbox",
    "tabindex": "0",
    "ariaMultiline": null,
    "hasInlineStyleAttr": false
  },
  "styleTags": 1,
  "styleTagChars": 1329,
  "canBold": true,
  "htmlAfterBold": "<p><strong>hello</strong></p>",
  "jsonAfterBold": "{\"type\":\"doc\",\"content\":[{\"type\":\"paragraph\",\"content\":[{\"type\":\"text\",\"marks\":[{\"type\":\"bold\"}],\"text\":\"hello\"}]}]}",
  "isActiveBold": true,
  "canUndo": true,
  "htmlAfterUndo": "<p>hello</p>",
  "htmlAfterRedo": "<p><strong>hello</strong></p>",
  "htmlAfterUnknownNode": "<p>ok</p><p>x</p>",
  "text": "ok\n\nx",
  "nodeTypesInSchema": ["paragraph","blockquote","bulletList","codeBlock","doc","hardBreak","heading","horizontalRule","listItem","orderedList","text"],
  "markTypesInSchema": ["link","bold","code","italic","strike","underline"]
}
```

O que isso prova, item a item:

| Observação | Conclusão |
|---|---|
| `className = "tiptap ProseMirror"` | a classe `tiptap` é **sempre** adicionada (`Editor.ts:697-699`) |
| `contentEditable=true`, `role=textbox`, `tabindex=0` | a11y mínima automática (`Editor.ts:610-630`, `extensions/tabindex.ts:44-56`) |
| `ariaMultiline = null` | **não** vem por padrão; o integrador precisa passar (`editorProps.attributes`) |
| `styleTags = 1`, `styleTagChars = 1329` | há CSS injetado por padrão (`injectCSS:true`) e ele é pequeno/estrutural (`codigo/core-style.ts`) |
| `hasInlineStyleAttr = false` | o elemento raiz **não** recebe estilo inline |
| `toggleBold` ⇒ `<strong>` | o comando produz a marca e o DOM sem UI |
| `getJSON()` com `marks:[{type:"bold"}]` | o JSON é o do ProseMirror, estável para persistir |
| undo/redo via `StarterKit` | `UndoRedo` funciona por comando (`can().undo()` antes de `undo()`) |
| `<table>` ⇒ texto preservado, nó descartado | conteúdo inválido é silencioso sem `enableContentCheck` |
| `getText() = "ok\n\nx"` | separador de bloco default `\n\n` (`Editor.ts:808-830`) |
| schema sem `table`, com `link/strike/underline` | StarterKit 3 inclui underline e link; listas e código também |

Erros de console: o único registro foi do **meu harness** (`tab.waitFor` recebeu uma expressão em
vez de seletor). Nenhuma exceção da página: `error: null`.

---

## 3. `cdn/probe.html` — EXECUTADO (após corrigir o harness)

Objetivo: digitar de verdade (`page.keyboard`), disparar input rule `**negrito**`, `setEditable`,
`unmount`/`mount`, `injectCSS:false`, `aria-label` customizado e um nó `callout` criado com
`Node.create`.

Histórico: as duas primeiras tentativas falharam no **harness**, não no Tiptap
(`tab.type(sel, text)` exige seletor; `page.keyboard.press('Control+End')` → `Unknown key`).
Com `keyboard.down('Control')` + `press('End')` + `up('Control')` funcionou. Saída observada:

```json
{
  "dom": { "className": "tiptap ProseMirror", "role": "textbox", "tabindex": "0",
           "ariaMultiline": null, "ariaLabel": null },
  "initialHtml": "<p>inicio</p>",
  "afterTyping": "<p>inicio fim</p><p><strong>negrito</strong> </p><p></p>",
  "isActiveBoldAfterSelectAll": false,
  "canUndo": true,
  "afterUndo": "<p>inicio fim</p><p><strong>negrito</strong></p>",
  "callout": {
    "html": "<div data-callout=\"\" data-tone=\"warn\">cuidado</div><p>inicio fim</p><p><strong>negrito</strong></p>",
    "json": "{\"type\":\"doc\",\"content\":[{\"type\":\"callout\",\"attrs\":{\"tone\":\"warn\"},\"content\":[{\"type\":\"text\",\"text\":\"cuidado\"}]},{\"type\":\"paragraph\",\"content\":[{\"type\":\"text\",\"text\":\"inicio fim\"}]},{\"type\":\"paragraph\",\"content\":[{\"type\":\"text\",\"marks\":[{\"type\":\"bold\"}],\"text\":\"negrito\"}]}]}"
  },
  "noCss": { "styleTagsBefore": 1, "styleTagsAfter": 1, "className": "tiptap ProseMirror",
             "ariaLabel": "Editor sem CSS", "role": "textbox" },
  "lifecycle": {
    "notEditable": { "isEditable": false, "tabindex": null, "contenteditable": "false" },
    "afterUnmount": { "isDestroyed": true },
    "contentPreserved": true
  },
  "consoleErrors": []
}
```

Leitura do que ficou provado:

| Observação | Conclusão |
|---|---|
| digitar `**negrito** ` (com espaço) virou `<strong>negrito</strong> ` | **input rule de mark funciona** por teclado real (`bold.tsx:138-149` + `InputRule.ts:319-338`) |
| `Ctrl+A` num documento misto ⇒ `isActive('bold') === false` | `isActive` responde "está ativo **em toda** a seleção", não "existe bold" |
| `undo()` removeu o parágrafo vazio criado pelo `Enter` | o histórico agrupa por transação, como no ProseMirror |
| nó `callout` ⇒ `<div data-callout data-tone="warn">` e JSON com `attrs.tone` | `Node.create` + `addAttributes`/`renderHTML` + `parseHTML` **round-trip** completo, sem plugin |
| 2º editor com `injectCSS:false`: `styleTagsBefore === styleTagsAfter === 1` | **inconclusivo** — ver §6. `createStyleTag` reaproveita a tag existente (`createStyleTag.ts:2-8`), então um 2º editor **com** CSS também deixaria a contagem em 1. O opt-out foi re-verificado em documento isolado (§6): 0 tags. |
| `editorProps.attributes['aria-label']` ⇒ atributo aplicado | `attributes` do usuário entram sem perder `role` (`Editor.ts:610-630`) |
| `setEditable(false)` ⇒ `contenteditable="false"`, `tabindex` **removido** | comportamento do `Tabindex` (`extensions/tabindex.ts:44-56`) confirmado |
| `consoleErrors: []` | nenhum aviso/erro durante edição, comando, nó custom e ciclo de vida |
| `afterUnmount.isDestroyed === true` e `isMounted` ausente | **divergência checkout-vs-publicado** (corrigida na revisão, ver §6.8): o CDN entregou **@3.30.3/3.33.0** publicados, e **ambos** retornam `isDestroyed === true` após `unmount()` (`get isDestroyed() { return this.editorView?.isDestroyed ?? true }`). O código de `ref/tiptap` é `main` (não a tag 3.30.3) e tem `return this.destroyed`, que daria `false`. `mount()` funcionou e preservou o conteúdo nos dois builds; não trate `isDestroyed` como "precisa de editor novo". |

**Não verificado** aqui: `Mod-b` (não pressionei `Control+b` nesta rodada), IME real e
colagem (o teste de clipboard não rodou nesta versão da sonda).

---

## 4. `minimal.mjs` — NÃO EXECUTADO

Exemplo mínimo canônico para quem tem o pacote instalado. Não rodado porque `ref/tiptap` não tem
`node_modules` e instalar 61 pacotes não é permitido nesta tarefa; além disso o monorepo exige
Node ≥24 (`ref/tiptap/package.json:84-87`) e o ambiente tem 22.23.2.

```bash
mkdir /tmp/tiptap-mini && cd /tmp/tiptap-mini
npm init -y
npm i @tiptap/core @tiptap/pm @tiptap/starter-kit jsdom
node minimal.mjs
```

Equivalente ao que já foi provado no browser em `cdn/index.html`.

---

## 5. O que **não** foi executado, e por quê

| Item | Motivo |
|---|---|
| `pnpm install`, `pnpm build`, `vp test` no monorepo | sem `node_modules`; instalar 61 pacotes não é permitido nesta tarefa; `engines.node >= 24` vs. 22.23.2 local |
| Testes Playwright do repo (`demos/**/*.spec.ts`) | exigem o monorepo construído (`playwright.config.ts` sobe `vp -C demos run start:e2e`) |
| Colaboração Yjs | exigiria `Y.Doc` + provider; fora do escopo mínimo |
| IME real (composição asiática, teclado virtual) | só foi possível ler o código (`InputRule.ts:324-338`, `view.composing`) |
| Performance | nenhuma medição; ver `avaliacao.md` §2 item 11 |

---

## 6. Revisão independente — sondas adicionais (EXECUTADAS)

Todas rodaram contra o **pacote publicado fixado em `@tiptap/core@3.30.3`** (`esm.sh`), para casar
com a versão declarada em `ref/tiptap`, e não contra o `@3` que resolve hoje em 3.33.0.

### 6.1 `cdn/isolated-nocss.html` — opt-out de CSS em documento limpo

O teste do §3 (`styleTagsBefore === styleTagsAfter === 1`) **não provava nada**: `createStyleTag`
começa com `document.querySelector('style[data-tiptap-style]')` e devolve a tag existente
(`utilities/createStyleTag.ts:2-8`). Ou seja, um segundo editor **com** CSS também daria 1.
Reexecutado com **um único editor num iframe com documento próprio**:

```bash
cd dossie/tiptap/exemplos
python3 -m http.server 8742 --bind 127.0.0.1 &
chromium-browser --headless --disable-gpu --no-sandbox \
  --virtual-time-budget=30000 --dump-dom \
  http://127.0.0.1:8742/cdn/isolated-nocss.html
```

Saída observada (verbatim, de `window.__RESULT__` / `<pre id="out">`):

```json
{
 "error": null,
 "styleNodes": 0,
 "className": "tiptap ProseMirror",
 "role": "textbox",
 "tabindex": "0",
 "toggleBold": true,
 "html": "<p><strong>sem css</strong></p>",
 "outerDocStyleTags": 0
}
```

Conclusão: `injectCSS: false` injeta **zero** `style`/`link` no documento do editor e o editor
continua funcional — mas a classe `tiptap` **permanece**. Não existe opção para removê-la.

### 6.2 Reuso da style tag (o confundidor, medido)

Duas instâncias com o default `injectCSS: true` na mesma página ⇒ `tagCount === 1`. Isso confirma
`querySelector` + reaproveitamento, e é a razão de o teste antigo ser inconclusivo.

### 6.3 Classes na raiz com `editorProps.attributes.class`

Com `class: 'my-wrap'` e `aria-label: 'Demo'`:

```json
{
 "className": "tiptap ProseMirror my-wrap",
 "role": "textbox",
 "tabindex": "0",
 "ariaMultiline": null,
 "ariaLabel": "Demo",
 "tagCount": 1,
 "tagChars": 1329,
 "inlineStyle": false
}
```

`prependClass()` concatena `tiptap` **à frente** (`Editor.ts:698`), então a classe do usuário vem
depois. `aria-multiline` continua ausente; `aria-label` entra sem perder `role`.

### 6.4 `chain()` devolve AND, não "nada mudou"

```json
{
 "focusThenUndoEmptyHistory": false,
 "selectAllBoldUndo": { "result": false, "html": "<p><strong>x</strong></p>" },
 "noopChain": true,
 "chainEmptyRun": true
}
```

`chain().selectAll().toggleBold().undo().run()` retornou **`false`** com o `<strong>` já aplicado
no documento. O retorno de `run()` não é um indicador de "houve mudança".

### 6.5 Estilo inline é default de extensão (não é teoria)

Com `StarterKit + TextAlign + Highlight`, conteúdo de entrada
`<p style="text-align: center">centro</p>`:

```json
{
 "html": "<p style=\"text-align: center;\">centro</p><p><mark>marca</mark></p>",
 "paraAttrs": [{ "textAlign": "center" }, { "textAlign": null }]
}
```

`TextAlign` grava `style="text-align: …"` no HTML **e** o atributo `textAlign` no JSON — as duas
representações. O `Highlight` sem `multicolor` descartou o `data-color` de entrada; com
`multicolor: true` ele emite `background-color` + `data-color` (`highlight.ts:90-99`, lido no
código, não executado).

### 6.6 Code block preserva `language-*`

`<pre><code class="language-js">` entra e sai idêntico — confirma a única classe default de node
(`extension-code-block/src/code-block.ts:153-156`).

### 6.7 `mergeAttributes` re-confirmado

`{class:'prose p-4', style:'color:red'}` + `{class:'prose custom', style:'color:blue; margin:0'}`
⇒ `class: "prose p-4 custom"`, `style: "color: blue; margin: 0"` — igual ao §1, agora também pelo
build publicado.

### 6.8 `isDestroyed` — a divergência é checkout vs. publicado

| Build | `get isDestroyed()` | após `unmount()` |
|---|---|---|
| `@tiptap/core@3.30.3` (esm.sh) | `editorView?.isDestroyed ?? true` | `true` |
| `@3` = 3.33.0 (esm.sh) | idem | `true` |
| código de `ref/tiptap` (main, `5fd5d2f`) | `this.destroyed` (`Editor.ts:901-903`) | `false` |

Como o checkout é `main` com 90 changesets não publicados, o dossiê original atribuiu à diferença
3.30.3→3.33.0 algo que é, na verdade, HEAD→publicado. `mount()` continua funcionando e preserva o
conteúdo em todos os casos testados.
