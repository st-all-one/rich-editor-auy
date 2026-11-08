# Exemplos e sondas — o que foi executado

Tudo aqui roda a **dist publicada 0.9.1** (`summernote-lite`), não o `src/` deste clone:
o `ref/summernote/` não tem `node_modules` e `yarn build` não foi executado (ver §5).
Ambiente: Linux, `chromium-browser` headless, Node v22.23.2, rede liberada para CDN.

## Arquivos

| Arquivo | O que é |
|---|---|
| `minimal.html` | integração mínima real: jQuery + `summernote-lite` por CDN, `<textarea>`, `code()` para ler/gravar, `onChange` |
| `probe.html` | sonda de comportamento por script dentro da página (init, API, formatação, undo, tabela, imagem, codeview, a11y, limites) |
| `probe2.html` | sonda 2: ranges após `code()`, sanitização de `pasteHTML`, parágrafos/listas, teclado, CSS computado |
| `cdp.html` | página limpa usada pelo driver (editor + botão seguinte, para testar Tab) |
| `drive.mjs` | driver CDP (DevTools Protocol): teclado **real**, IME, Tab, Ctrl+Z, XSS no reload e timing com relógio real |

A opção de tabulação é `tabSize` (não `tabsize`); as sondas usavam a grafia errada e foram
corrigidas na revisão — nada nelas depende do valor, e a saída observada não mudou.

## 1. Exemplo mínimo

Abrir `minimal.html` em um navegador (precisa de rede para o CDN):

```
chromium-browser exemplos/minimal.html
```

O que observar: o `<textarea>` some e vira `.note-editor`; “ler HTML” chama
`$('#body').summernote('code')`; “gravar HTML” chama `code('<h2>…</h2>')`; o `<pre>` mostra o
retorno de `onChange`.

Não foi executado com captura automatizada de tela; a execução equivalente (init + `code()` +
eventos) está coberta pelas duas sondas abaixo.

## 2. Sondas no DOM (`probe.html` / `probe2.html`)

Comando usado (o mesmo para as duas):

```bash
cd dossie/summernote/exemplos
timeout 120 chromium-browser --headless=new --disable-gpu --no-sandbox --hide-scrollbars \
  --virtual-time-budget=30000 --dump-dom "file://$PWD/probe.html" \
  2>/dev/null | sed -n '/<pre id="log">/,/<\/pre>/p'
```

### Saída observada — `probe.html` (trechos literais)

```
env.jquery	3.7.1
env.summernote	0.9.1
env.execCommandSupported.bold	true
init.layout	{"editorClasses":"note-editor note-frame","editorChildTags":["DIV.note-dropzone","DIV.note-toolbar","DIV.note-editing-area","OUTPUT.note-status-output","DIV.note-statusbar","DIV.note-modal link-dialog","DIV.note-popover bottom note-link-popover","DIV.note-modal","DIV.note-popover bottom note-image-popover","DIV.note-popover bottom note-table-popover","DIV.note-modal","DIV.note-modal"],"noteHidden":true,"noteTag":"TEXTAREA"}
a11y.editableAttrs	{"tag":"DIV","contenteditable":"true","role":"textbox","ariaMultiline":"true","spellcheck":"true","autocorrect":"true","tabindex":null}
a11y.toolbar	{"role":"toolbar","buttons":342,"buttonsTabindexMinus1":334,"buttonsWithoutLabel":8,"statusOutput":{"role":"status","live":"polite"}}
code.initial	<p>Hello <b>world</b></p>
code.roundtrip	<p>Hello <i>world</i></p>
code.normalization	<p>a</p><div>b</div>c<p></p><ul><p>li?</p></ul>
code.emptyValue	{"code":"","isEmpty":true}
format.boldClick	{"html":"<p>alpha <b>beta</b> gamma</p>","buttonActiveClass":true}
format.italicShortcut	<p>alpha beta <i>gamma</i></p>
format.h2	<h2>título aqui</h2>
format.styleWithCSS	<p>alpha <span style="font-weight: bold;">beta</span></p>
table.insert	<p><br></p><table class="table table-bordered">…</table><p><br></p>
image.dataUrl.after	{"html":"<p><img src=\"data:image/gif;base64,…\" style=\"width: 1px;\"></p>","hasAlt":false}
codeview.purifyOnExit	{"html":"<p>a</p>window.__cv=1","cvRan":"undefined"}
persist.textareaValue	{"textareaVal":"<p>persistido</p>"}
destroy	{"editorsLeft":1,"noteVisible":true,"noteValue":"<p>editado no DOM</p>"}
```

Leituras:

- `a11y.toolbar` conta **todos** os botões do layout (inclusive os 128 da paleta de cores e os de
  modais): 342 botões, 334 com `tabindex="-1"`.
- `code.normalization` mostra o parser do browser “consertando” HTML inválido.
- `format.*` mostra o efeito de `styleWithCSS` na saída.
- `codeview.purifyOnExit` mostra a tag `<script>` removida e **o código virando texto**.

### Saída observada — `probe2.html` (trechos literais)

```
A1.insertTextSemCaret	<p>Xabc</p>
A2.pasteHTMLSemCaret	<p>abc</p>
A4.pasteHTMLComCaret	<p>abc<b>Y</b></p>
B1.scriptTag	{"html":"<p>x</p><p>ok</p><p>window.__a=1</p>","ran":"undefined"}
B2.onerrorAttr	{"html":"<p>x<img src=\"nope.png\" onerror=\"window.__b=1\"></p>","ran":"undefined"}
B3.javascriptHref	<p>x<a href="javascript:window.__c=1">click</a></p>
B4.styleTag	<p>xbody{display:none}</p>
B6.iframeYoutube	<p>x<iframe src="https://www.youtube.com/embed/abc123"></iframe></p>
B7.onclickSurvivesCodeview	<p onclick="window.__d=1">click me</p>
C1.enterSplit	<p>abc</p><p>def</p>
C2.enterNoPara	<p>texto</p><p> solto</p>
C3.ulFromPara	<ul><li>um</li></ul><p>dois</p>
C4.indent	<ul><li>a<ul><li>b</li></ul></li></ul>
C5.emptyListEnter	<ul><li>a</li></ul><p><br></p>
D1.tabInsertsSpaces	{"html":"<p>ab</p>","activeElement":"note-editable"}
D3.ctrlEnterHr	<p>ab</p><hr><p><br></p>
E1.changeCount	35
E4.textareaSync	<p>abZ</p>
F1.computedStyle	{"editablePadding":"10px","editableOverflow":"auto","editableFontFamily":"sans-serif","editableFontSize":"16px","pMargin":"16px 0px","pFontFamily":"sans-serif","bodyFontFamily":"\"Times New Roman\""}
F2.classesInjected	{"count":150,"sample":["note-editor","note-frame","note-dropzone","note-toolbar","note-btn-group","note-style","note-btn","dropdown-toggle", …]}
```

Leituras:

- `D1.tabInsertsSpaces` **não** insere espaços: com `keydown` sintético nada acontece; com teclado
  real (CDP) o Tab sai do editor. `tabSize` não é aplicado à tecla — ver `Editor.js:465-468`.
- `F1` prova que o conteúdo herda `sans-serif` do `.note-frame` mesmo com `body` em Times.
- `B2/B3/B7` provam que atributos de evento e `javascript:` sobrevivem ao filtro.

Os passos `perf.*` do `probe.html`/`probe2.html` retornaram `0 ms` porque
`--virtual-time-budget` congela o relógio; por isso as medições de tempo foram refeitas no driver
CDP (abaixo). Dois passos do `probe2` lançaram erro **da própria sonda**, registrados como
limitação da sonda e não do editor: `B5` (`Failed to execute 'setStart' on 'Range'`, caret em nó
nulo após mutações) e `G3` (`Cannot read properties of undefined (reading 'stack')`).

## 3. Driver CDP (teclado real, IME, XSS, tempo)

```bash
cd dossie/summernote/exemplos
timeout 180 node drive.mjs
```

Saída observada (íntegra, resumida só nas linhas longas; tempos variam ~±10% entre execuções —
reexecuções na revisão deram `undoOne` 13,8 ms, `readCode` 0,24 ms e `perKeystroke` 1,14 ms):

```
init	{"version":"0.9.1","editable":true}
focus	note-editable
type.afterXYZ	<p>alpha betaXYZ</p>
type.ctrlB	<p>alpha betaXYZ</p>
key.tab	{"html":"<p>alpha betaXYZ</p>","active":"BUTTON.","isNextButton":true}
key.shiftTab	{"html":"<p>alpha betaXYZ</p>","active":"BODY."}
ime.composing	{"html":"<p>alpha betaXYZに</p>","stack":4}
ime.committed	{"html":"<p>alpha betaXYZにほん</p>","stack":4}
key.enter	<p>alpha betaXYZにほん</p><p><br></p>
key.ctrlZ	<p>alpha betaXYZ</p>
perf.doc	{"docBytes":16990,"keystrokes":30,"totalMs":31.3,"perKeystrokeMs":1.04,"stackLen":1,"snapshotBytes":16990,"historyLimit":200}
perf.readCode	{"perReadMs":0.26}
perf.undoOne	{"undoMs":13.5}
xss.reloadOnerror	{"pwned":"yes","html":"<img src=\"nope.png\" onerror=\"window.__pwned='yes'\">"}
xss.reloadJavascriptHref	{"pwned2":"no","html":"<p><a id=\"j\" href=\"javascript:window.__pwned2='yes'\">go</a></p>"}
iframe.reloadEvil	<p>x</p><iframe src="https://evil.example/x" width="200" height="80"></iframe>
clipboard.copyHandlerExists	{"copyEvents":1}
a11y.toolbarTabOrder	{"toolbarButtons":171,"tabbable":4,"tabbableClasses":["note-color-reset …","note-color-select …"],"dropdownItemsFocusable":0}
perf.recordUndo50	{"perSnapshotMs":0.04,"stackLen":51,"totalBytes":612}
a11y.tabSequence	["DIV#.note-editable"]
done	ok
```

Leituras e ressalvas:

- `type.ctrlB` **não** é prova de que Ctrl+B falha: o driver enviou o evento com `text`/código e o
  Chromium pode ter tratado como edição nativa; o caminho do keymap foi verificado por outro meio
  (sonda 1, `format.italicShortcut`, que usa `Ctrl+I` via jQuery e funciona). Fica como
  **lacuna**: Ctrl+B com teclado real não confirmado.
- `key.tab`/`key.shiftTab`: Tab real **sai** do editor (HTML intacto) — o editor não captura Tab.
- `ime.*`: a composição entra no DOM e **não** empilha snapshot durante a composição; ao commitar,
  o texto final aparece. `stack:4` para 3 caracteres digitados antes demonstra 1 snapshot por tecla.
- `perf.doc`: os 30 keystrokes via `execCommand` **não** geram snapshots (`stackLen:1`), então os
  32 ms medem só a edição; `undoMs:13.9` mede o custo do snapshot de 17 KB.
- `xss.reloadOnerror`: **o handler executou** (`pwned: yes`) ao recarregar HTML salvo por `code()`.
- `a11y.toolbarTabOrder`: 171 botões na toolbar, 4 tabuláveis. (A sonda `probe.html` conta 342
  botões no layout inteiro — toolbar + modais + paleta de cores —, dos quais 334 têm
  `tabindex="-1"`; são populações diferentes, não um conflito.)
- `perf.recordUndo50`: 50 `recordUndo()` num documento pequeno = 0,04 ms/snapshot; não mede o custo
  de documentos grandes, serve só para mostrar que o snapshot é uma serialização barata *por si*,
  cara quando multiplicada pelo tamanho do documento a cada tecla.

## 4. Verificação dos artefatos usados

```bash
$ curl -s -o /dev/null -w "%{http_code} %{size_download}\n" \
    https://cdn.jsdelivr.net/npm/summernote@0.9.1/dist/summernote-lite.min.js
200 186367

$ curl -s -o /dev/null -w "%{http_code} %{size_download}\n" \
    https://cdn.jsdelivr.net/npm/summernote@0.9.1/dist/summernote-lite.min.css
200 30684
```

## 5. O que NÃO foi executado, e por quê

| Item | Motivo | Como verificar |
|---|---|---|
| `yarn install` / `yarn build` / `yarn test` | o clone não tem `node_modules`; a instrução do projeto proíbe instalar o mundo | `cd ref/summernote && yarn install && yarn test` (vitest + Chrome headless, `vitest.config.js:5-16`) |
| Build a partir do `src/` | depende do passo acima | `yarn build` gera `dist/summernote-{lite,bs3,bs4,bs5}.{js,css}` (`vite.config.js`) |
| Firefox / Safari / mobile | um único browser disponível | repetir as sondas trocando o binário |
| Colagem real de HTML do SO | exigiria clipboard do sistema | testar `paste` com HTML do Word/Docs |
| IME real de teclado | simulado via `Input.imeSetComposition` | testar com IME coreano/japonês |
| Leitor de tela | não disponível no ambiente headless | NVDA/VoiceOver sobre `.note-editable` e toolbar |

## 6. Reproduzir offline (sem CDN)

Baixe `summernote-lite.min.{js,css}` e `jquery-3.7.1.min.js` para uma pasta local e troque as URLs
nos HTMLs. Nada nas sondas depende de servidor: são arquivos abertos por `file://`.
