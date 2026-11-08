# Exemplos e execução — Quill 2.0.3

Código de inicialização e scripts usados para observar comportamento. **Tudo o que está
descrito aqui foi executado**, exceto os itens marcados "não executado". A execução usa o
build publicado **2.0.3** (mesma versão de `ref/quill/packages/quill/package.json:3`), porque
o monorepo em `ref/` não tem `node_modules` e o build local exigiria instalar webpack, Babel,
TypeScript e o restante do devDependencies.

| Arquivo | O que é | Executado? |
|---|---|---|
| `minimal.html` | integração mínima real: toolbar em HTML, editor, init, persistência em Delta | **sim** (Chromium headless) |
| `saida-minimal-dom.html` | DOM após o init (dump completo) | — (saída) |
| `probe.html` | roteiro de API: init, edição, undo, clipboard, sanitização, formato custom, readOnly, CSS computado, benchmark | **sim** |
| `saida-probe.json` | resultados do probe em JSON | — (saída) |
| `interact.cjs` | interação real com Playwright: teclado, seleção, undo, clipboard, IME via CDP, toolbar, readOnly, benchmark | **sim** |
| `saida-interact.json` | resultados da interação em JSON | — (saída) |
| `delta-steps.cjs` | exercício isolado de `insert/delete/retain/compose/transform/invert/diff` do `quill-delta@5.1.0` | **sim** (Node 22) |
| `saida-delta.json` | resultados do exercício de Delta | — (saída) |

Ambiente observado: Linux x86_64, Chromium 156.0.8078.79 (build Playwright
`~/.cache/ms-playwright/chromium-1248/chrome-linux64/chrome`), Node 22.23.2. As saídas são
reproduções no ambiente descrito, não benchmarks oficiais.

---

## 1. Exemplo mínimo (`minimal.html`)

Abre com CSS e JS do CDN, declara a toolbar com classes `ql-*` e inicializa:

```js
const quill = new Quill('#editor', {
  theme: 'snow',
  modules: { toolbar: '#toolbar' },
});
const json = JSON.stringify(quill.getContents()); // persistir
quill.setContents(JSON.parse(json));              // carregar
```

**Comando executado**

```bash
cd dossie/quill/exemplos
/home/one/.cache/ms-playwright/chromium-1248/chrome-linux64/chrome --headless=new --no-sandbox --disable-gpu --virtual-time-budget=6000 \
  --dump-dom file://$PWD/minimal.html > saida-minimal-dom.html
```

**Saída observada** (trecho de `saida-minimal-dom.html`)

```html
<div id="editor" class="ql-container ql-snow">
  <div class="ql-editor" contenteditable="true">
    <p>Hello World!</p>
    <p>Some initial <strong>bold</strong> text</p>
    <p><br></p>
  </div>
  <div class="ql-tooltip ql-hidden">
    <a class="ql-preview" rel="noopener noreferrer" target="_blank" href="about:blank"></a>
    <input type="text" data-formula="e=mc^2" data-link="https://quilljs.com" data-video="Embed URL">
    <a class="ql-action"></a><a class="ql-remove"></a>
  </div>
</div>
```

A toolbar fornecida foi transformada em `class="ql-toolbar ql-snow"`, os botões ganharam
`type="button"`, `aria-pressed="false"` e SVG injetado. O `<div class="ql-tooltip">` é DOM do
tema dentro do container — evidência de que o tema não é só CSS.

---

## 2. Probe de API (`probe.html` → `saida-probe.json`)

Cobre, em uma página:

- Delta inicial derivado do HTML (`loadDeltaOfInitialHTML`);
- classes e atributos do container/raiz (`containerClass`, `rootClass`,
  `rootContentEditable`, `rootRole`, `rootAriaLabel`);
- edição por API (`afterEditsDelta`, `afterEditsHTML`);
- undo/redo (`afterUndo`, `afterRedoLength`);
- `clipboard.convert` de HTML estilo Word/Google Docs (`clipboardConvert`);
- sanitização de link/imagem (`dangerouslyPasteHTML` → `urlSanitizedHTML`);
- formato customizado via `ClassAttributor` (`customFormatDelta`, `customFormatDOM`);
- `readOnly` (`disabledContentEditable`, `disabledReturnsFalseForUserEdits`);
- toolbar (`toolbarClass`, `toolbarRole`, `toolbarButtons`);
- CSS computado (`css.editor`) e lista com CSS (`css.list`);
- benchmark de 1.000 parágrafos e 200 inserts (`bench`).

**Comando executado**

```bash
/home/one/.cache/ms-playwright/chromium-1248/chrome-linux64/chrome --headless=new --no-sandbox --disable-gpu --virtual-time-budget=20000 \
  --dump-dom file://$PWD/probe.html > dom.html
# extração do <pre id="out"> para JSON (Python) → saida-probe.json
```

**Achados que importam** (números e valores exatos em `saida-probe.json`):

- `rootRole: null`, `rootAriaLabel: null` — o nó editável não tem ARIA próprio.
- `css.editor`: `whiteSpace: "pre-wrap"`, `fontFamily: "Helvetica, Arial, sans-serif"`,
  `fontSize: "13px"`, `pMargin: "0px"` — o CSS do Quill define tipografia e espaçamento.
- `css.list.listStyleType: "none"` e `olPaddingLeft: "19.5px"` — a numeração não vem do
  browser, vem de `.ql-ui:before` + counters.
- `clipboardConvert`: um `<b style="font-weight:normal">` do Word envolveu o conteúdo e o
  `bold` vazou para a lista seguinte — a normalização é heurística e tem bordas.
- `customFormatDelta`: `formatText(..., 'highlight', true)` produz
  `{"highlight":"true"}` no Delta e `class="ql-highlight-true"` no DOM.
- `bench`: `setContents` de 1.000 parágrafos (~48.890 caracteres) ≈ 37 ms; `getContents`
  ≈ 0,1 ms; 200 inserts ≈ 335 ms; 1.000 nós no editor.
- `urlSanitizedHTML`: `href="javascript:..."` → `href="about:blank"`;
  `onerror` descartado; `src` inválido → `//:0`.

**Limite deste probe**: `noCss.editor` não mede ausência de CSS — a página carrega
`quill.snow.css` e o segundo editor herda `.ql-editor`. A conclusão de "CSS obrigatório" vem
de `src/assets/core.styl` e da medição de `css.editor`, não deste campo. Ver
`avaliacao.md` §4 e §5.

---

## 3. Interação real (`interact.cjs` → `saida-interact.json`)

Playwright dirigindo o Chromium instalado, contra a página
[`interactive.html`](interactive.html) (mesmo conteúdo do `minimal.html`, com hooks
`window.quill` e `window.getState`), via `python3 -m http.server 8765`.

**Comandos executados**

```bash
cd /tmp/deltademo   # diretório de trabalho da execução; scripts copiados para este dossier
python3 -m http.server 8765 &
node interact.cjs > saida-interact.json
```

O script usa `playwright-core@1.54.1` (instalado apenas localmente, sem browsers próprios) e
`executablePath` apontando para o Chromium do Playwright. Pré-requisito: `npm i playwright-core`
e o binário do Chromium; não faz parte do repositório.

**Resultados observados** (resumo; JSON completo no arquivo):

| Cenário | Resultado |
|---|---|
| digitar "Olá mundo" | Delta `[{insert:"Olá mundo\n"}]`, caret em 9, ~27 ms (varia entre execuções) |
| Enter + "segundo" | `<p>Olá mundo</p><p>segundo</p>`, Delta com dois `\n` |
| Shift+← ×7 + `Ctrl+B` | `segundo` com `bold:true` no Delta e `<strong>` no HTML |
| `Ctrl+Z` | volta ao documento anterior (`initial`) |
| `Ctrl+Shift+Z` | refaz |
| digitar `- ` + "item" | autofill vira `list: bullet` (`<ul><li>item</li>`) |
| `Tab` em item de lista | insere `\t` no texto (indent é `format('indent','+1')`, não recuo automático de lista) |
| colar texto do clipboard | `colado do clipboard` no ponto do caret |
| colar HTML `<b>Bold</b>…<ul><li>` | Delta com `bold`, `italic`, `list:bullet`; `<ul><li>` no HTML semântico |
| IME `にほん` → commit `日本` | durante composição o Delta fica vazio (`[{"insert":"\n"}]`); após o commit, `[{"insert":"日本\n"}]` — o script limpa o editor antes do cenário |
| `disable()` + digitar | Delta inalterado (`readOnlyTyping`); `contenteditable` vira `"false"` (`disabledContentEditable`, no probe) |
| API com `disable()` | ainda insere (retorna Delta) |
| clique em `ql-bold` | formata a seleção (`<strong>cliqu</strong>e aqui`) |
| 120 caracteres via `keyboard.type` | ~247 ms, comprimento 121 |

Notas: o `Tab` em lista **não** indentou o item (o resultado foi `outro\t`), porque a binding
de `Tab`/`indent` só atua quando `format` inclui `indent`/`list` e a seleção estava no texto;
não é um bug observado, é o contrato do binding (`keyboard.ts:384-404`). O teste de autofill
foi refeito com o documento vazio e caret em 0.

---

## 4. Delta isolado (`delta-steps.cjs` → `saida-delta.json`)

Usa `quill-delta@5.1.0` (tarball npm extraído) e as dependências `fast-diff`,
`lodash.clonedeep`, `lodash.isequal`. Demonstra:

- documento `"Olá" + " mundo"{bold} + "\n"{header:1}` com `length() = 10`;
- edição `retain 4, insert "!", delete 1, retain 5 {italic}`;
- `compose` aplicando a edição, `diff` derivando a mesma edição de volta;
- `invert(edit, base)` com `insert "m"{bold}` e `italic:null` para desfazer;
- `transform` de mudanças concorrentes e `transformPosition` para cursor;
- `push` canônico (`insert` antes de `delete`), `chop` removendo `retain` final;
- `registerEmbed('img')` para `compose` de retains em objetos.

**Comando executado**

```bash
cd /tmp/deltademo
NODE_PATH=/tmp/deltademo/node_modules node delta-steps.cjs
```

O script importa o Delta de um caminho absoluto (`/tmp/qd/package/dist/Delta.js`, tarball
5.1.0 extraído) e usa `NODE_PATH` para `fast-diff`/`lodash.*`. **Reverificado na revisão de
2026-11-07**: reexecutado em Node 22.23.2 e a saída foi idêntica a `saida-delta.json`
(comparação por JSON, não visual). Para reproduzir fora deste ambiente, extraia o tarball e
ajuste o `require`.

**Não executado**: build do monorepo (`npm run build`), lint, Vitest, E2E e fuzz do Quill —
exigiriam instalar todo o devDependencies e browsers/Playwright do projeto. Para reproduzir:
`cd ref/quill && npm ci && npm run build -w quill && npm test -w quill` (os testes unitários
usam browser via Vitest/Playwright, `test/unit/vitest.config.ts:22-27`).
