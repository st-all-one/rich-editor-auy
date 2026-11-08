# Summernote — dossiê de referência

> Estudo verificável de `ref/summernote/` (commit `ac5460e`, 2026-01-04, versão `0.9.1`,
> licença MIT) para o design do `rich-editor-auy`.
> Este documento é sobre **a biblioteca**, não sobre o site/app `summernote.org`.

## 1. O que é, e o que não é

Summernote é uma **biblioteca** jQuery que troca um `<div>` (ou `<textarea>`) por um editor
WYSIWYG montado sobre `contenteditable`. O repositório é a biblioteca e seus exemplos; não há
produto final (não há app de documentos, nem colaboração, nem armazenamento).

`ref/summernote/README.md:7` define o escopo: “a JavaScript library that helps you create WYSIWYG
editors”; `README.md:21` fixa a dependência: “Summernote is built on jQuery”.

O que **não** existe no repositório:

- nenhum modelo de documento, schema, transação ou validador;
- nenhuma camada de persistência (só o `innerHTML` do div, ver §3);
- nenhum backend/connector no core (os “connectors” vivem em `awesome-summernote`, fora do repo);
- colaboração, versionamento ou diff.

## 2. Ficha técnica

| Item | Valor | Evidência |
|---|---|---|
| Versão | `0.9.1` | `ref/summernote/package.json:4` |
| Commit local | `ac5460e` (2026-01-04) | `git log -1` no `ref/` |
| Licença | MIT | `ref/summernote/LICENSE:1-3` |
| Linguagem | JavaScript ES modules (`.js`; em versões antigas era TypeScript `.ts`) | `src/js/**` |
| Dependência de runtime | jQuery (global, não empacotado) | `vite.config.js:72-74` (`external: id === 'jquery'`), `package.json` sem `peerDependencies` |
| Dependência opcional | Bootstrap 3/4/5 (CSS + JS) **ou** a build “lite” | `src/styles/{bs3,bs4,bs5,lite}/` |
| Código-fonte JS | 9.254 linhas em 39 arquivos | `find src/js -name '*.js' \| xargs wc -l` |
| CSS próprio | `common.scss` 624 linhas + 849 linhas de SCSS no tema lite | `src/styles/summernote/common.scss`, `src/styles/lite/**/*.scss` |
| Build “lite” pronta | 186.367 B JS + 30.684 B CSS (min) | `curl` do `dist/` 0.9.1 (ver `exemplos/README.md`) |
| Idiomas | 46 arquivos em `public/lang/`, incl. `pt-BR` | `ls public/lang/*.js \| wc -l` |
| Testes | Vitest + browser (Chrome headless) | `vitest.config.js:5-16`, `test/**` |

## 3. A tese central: o HTML/`contenteditable` é a fonte de verdade

Todo o design decorre de uma decisão: **o documento é o DOM vivo do `div[contenteditable]`**.
Não há árvore intermediária, schema ou AST. As consequências são literais no código:

```js
// ref/summernote/src/js/Context.js:98-111
code(html) {
  const isActivated = this.invoke('codeview.isActivated');
  if (html === undefined) {
    this.invoke('codeview.sync');
    return isActivated ? this.layoutInfo.codable.val() : this.layoutInfo.editable.html();
  } else {
    if (isActivated) { this.invoke('codeview.sync', html); }
    else { this.layoutInfo.editable.html(html); }
    this.$note.val(html);
    this.triggerEvent('change', html, this.layoutInfo.editable);
  }
}
```

- **Ler** = `$editable.html()`; **gravar** = `$editable.html(html)`. Nada é interpretado.
- O undo não guarda operações: guarda o HTML inteiro (`src/js/editing/History.js:12-20`).
- Os comandos são, em boa parte, `document.execCommand` (`src/js/module/Editor.js:58-70`), cuja
  saída é decidida pelo browser e não pelo editor.
- O “schema” é um conjunto de predicados sobre `nodeName` (`src/js/core/dom.js:80-140`:
  `isPara`, `isList`, `isCell`, `isVoid`…). Não há restrição de conteúdo: o HTML carregado entra
  como está.

### 3.1 O que isso ganha (e é muito)

1. **Integração de 3 linhas.** Um `<div>`/`<textarea>`, jQuery e `$('#x').summernote()`. O exemplo
   do README (`README.md:29-38`, `:44-46`, `:52-56`) e `ref/summernote/index.html:13-17` são exatamente isso.
2. **Compatível com qualquer backend** que produza/consuma HTML — inclusive HTML “sujo” que já
   está no banco.
3. **Formulário “de graça”.** Se a origem é um `<textarea>`, o módulo `AutoSync` copia o HTML para
   o `value` a cada `summernote.change` (`src/js/module/AutoSync.js:5-15`); verificado: após editar,
   `$('#note').val()` já contém o HTML (probe2, passo `persist.textareaValue`).
4. **Formatação nativa e barata.** Negrito, itálico, listas, links, tabelas e desfazer do browser
   funcionam sem reimplementar edição de texto.
5. **Interoperabilidade de colagem.** O conteúdo colado do Word/Google Docs costuma “funcionar”,
   porque o browser faz o parsing e o editor só mexe em casos específicos.
6. **Zero build no consumidor.** `dist/summernote-lite.min.js` é um IIFE que só precisa de
   `window.jQuery`; não exige bundler nem framework.

### 3.2 O que isso custa (e é caro)

1. **O HTML de saída é do browser, não seu.** O mesmo comando produz tags diferentes conforme a
   configuração e o browser. Medido em Chromium 0.9.1: `bold` com `styleWithCSS:false` gera
   `<b>beta</b>`; com `styleWithCSS:true` gera `<span style="font-weight: bold;">beta</span>`
   (probe1, `format.boldClick` e `format.styleWithCSS`). Não há normalização na saída.
2. **Conteúdo inválido é “consertado” pelo parser, silenciosamente.** Gravar
   `<p>a<div>b</div>c</p><ul><p>li?</p></ul>` e ler de volta devolve
   `<p>a</p><div>b</div>c<p></p><ul><p>li?</p></ul>` (probe1, `code.normalization`). O `<p>` vazio
   aparece e o `<p>` dentro do `<ul>` permanece inválido.
3. **Undo grosseiro e caro.** Cada snapshot é o HTML completo + bookmark. Digitar 3 caracteres
   reais levou a pilha de 4 snapshots (CDP `ime.composing`, `stack:4`). Desfazer um documento de
   17 KB custou ~14 ms em headless (CDP `perf.undoOne`, 13,5–13,9 ms em duas execuções), contra
   ~0,25 ms para ler o código (`perf.readCode`). `historyLimit` é 200 (`src/js/settings.js:142`),
   ou seja, até 200 cópias do documento em memória. Pior: **todo `keydown` serializa o documento**
   — `this.snapshot = this.history.makeSnapshot()` roda antes do keymap
   (`src/js/module/Editor.js:346-347`), independentemente de `recordEveryKeystroke`; essa cópia só
   serve ao limite de texto e é descartada na tecla seguinte.
4. **`code()` não é uma operação desfazível.** Trocar o conteúdo pela API não empilha undo; o
   primeiro `undo` depois de `code()` volta ao estado em que o editor foi inicializado
   (probe1, `history.undo`, que retornou o HTML da instância anterior). Importar/salvar via API e
   esperar undo coerente é uma armadilha real.
5. **Não há sanitização.** `purify()` é um `String.replace` com regex que remove *tags*
   (`src/js/module/Codeview.js:72-95`; é aplicado no `deactivate` do code view e no
   `editor.pasteHTML`, **nunca** em `code()`). Medido: `<script>` é removido mas **o corpo do script vira
   texto** (`<p>window.__a=1</p>`); `<style>` idem (`<p>xbody{display:none}</p>`);
   `onerror=`, `onclick=` e `href="javascript:…"` **passam intactos** e o `onerror` **executa** quando
   o HTML é recarregado via `code()` (CDP `xss.reloadOnerror` → `{"pwned":"yes"}`). O próprio README
   avisa: “filter/sanitize the HTML on the server” (`README.md:71`).
6. **Acoplamento visual.** A build `bs4` injeta `card`, `card-header`, `btn btn-light btn-sm`,
   `dropdown-menu`, e delega modal/tooltip ao JS do Bootstrap
   (`src/styles/bs4/summernote-bs4.js:5-16`, `:231-262` tooltip, `:289-293` modal). A “lite”
   reimplementa dropdown, modal e tooltip em classes próprias (203 linhas em
   `src/styles/lite/js/{DropdownUI,ModalUI,TooltipUI}.js`) e mantém um `ui_template` de 777 linhas
   (`src/styles/lite/summernote-lite.js`) — prova de que “sem Bootstrap” não significa “sem CSS”.
7. **Acessibilidade fraca.** Há `role="toolbar"`, `role="textbox"`, `aria-multiline`,
   `aria-live="polite"` no status (`src/styles/bs4/summernote-bs4.js:6-16`), mas os botões nascem
   com `tabindex="-1"` para não roubar a seleção. Medido: **4 de 171** botões da toolbar são
   alcançáveis por Tab (CDP `a11y.toolbarTabOrder`), e o Tab real **sai** do editor (CDP `key.tab`).
   Não há `aria-pressed` nos toggles (`toggleBtnActive` só adiciona classe `active`,
   `src/styles/bs4/summernote-bs4.js` função `toggleBtnActive`), e imagens são inseridas sem `alt`.
8. **Colagem/cópia só parcialmente controladas.** O módulo `Clipboard` trata **arquivos** e limite de
   texto, não HTML (`src/js/module/Clipboard.js:24-44`); `copy` apenas dispara callback
   (`src/js/module/Editor.js:390-392`), o clipboard é o nativo do browser.

## 4. Início mínimo real (jQuery + init)

Arquivo completo em `exemplos/minimal.html` (execução equivalente nas sondas; ver `exemplos/README.md`):

```html
<script src="https://code.jquery.com/jquery-3.7.1.min.js"></script>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/summernote@0.9.1/dist/summernote-lite.min.css">
<script src="https://cdn.jsdelivr.net/npm/summernote@0.9.1/dist/summernote-lite.min.js"></script>

<textarea id="body" name="body"><p>Olá, <b>mundo</b>.</p></textarea>
<script>
  $(function() {
    $('#body').summernote({ height: 220, tabSize: 2 });
    $('#body').summernote('code');          // ler HTML
    $('#body').summernote('code', '<p>x</p>'); // gravar HTML
  });
</script>
```

Observações de execução (Chromium headless, 0.9.1 via CDN):

- o `<textarea>` original é **escondido**, não removido: `noteHidden:true`, `noteTag:"TEXTAREA"`
  (probe1, `init.layout`);
- o editor criado tem as classes `note-editor note-frame` e contém, nesta ordem,
  `note-dropzone`, `note-toolbar`, `note-editing-area`, `note-status-output`, `note-statusbar` e
  **quatro modais + três popovers** já no DOM (probe1, `init.layout`);
- `destroy` restaura o HTML no `textarea` e o mostra de novo
  (`src/styles/lite/summernote-lite.js:765-770`; nas builds `bs3/bs4/bs5` o mesmo método
  **não** faz `$note.off('summernote')`, então handlers de uma instância anterior sobrevivem —
  `src/styles/bs4/summernote-bs4.js:318-323`).

## 5. API pública (superfície real)

Um único método jQuery com dois modos (`src/js/summernote.js:10-15`):

```js
$('#note').summernote(options)              // inicializa / devolve jQuery
$('#note').summernote('code')               // chama Context.code()
$('#note').summernote('editor.formatH2')    // namespace.method
$('#note').summernote('insertText', 'x')    // método do módulo 'editor'
```

O dispatch é textual em `Context.invoke` (`src/js/Context.js:227-243`): `'a.b'` acha o módulo `a`
e chama `b`; sem ponto, tenta `Context` e depois o módulo `editor`. **Não há tipos, nem validação**:
um nome errado retorna `undefined` silenciosamente. Duas consequências de integração verificadas no
código (`src/js/summernote.js:26-32`, `:20-22`):

- chamar `.summernote(opts)` num elemento já inicializado é **no-op silencioso** — as novas opções
  são ignoradas até `destroy` (`if (!$note.data('summernote'))`); não há erro nem aviso;
- opções desconhecidas também são ignoradas: `tabsize` (minúsculo) não existe, o nome real é
  `tabSize` (`src/js/settings.js:128`), e nada denuncia o engano.

Callbacks disponíveis em `src/js/settings.js:213-232` (`onChange`, `onInit`, `onKeydown`, `onPaste`,
`onImageUpload`, `onImageUploadError`, …), disparados por `Context.triggerEvent`, que também emite
`summernote.<nome>` no elemento (`src/js/Context.js:138-147`).

## 6. Prós e contras para o nosso caso

| Prós | Contras |
|---|---|
| Init trivial, funciona com jQuery e `<form>` | O documento é DOM/string: sem schema, sem transação, sem diff |
| Formatação nativa completa (listas, tabelas, links) | Saída de HTML varia por browser/opção; sem normalização |
| 46 idiomas prontos, inclusive `pt-BR` | CSS e classes próprias + dependência opcional de Bootstrap |
| Air mode, code view, upload de imagem, DnD | Sanitização inexistente; XSS no HTML salvo |
| Ecossistema de plugins por `context` | Plugins não declaram nós/marks; não há schema de extensão |
| MIT, ativo, CI com browser | A11y limitada (toolbar fora do Tab; sem aria-pressed) |
| `lite` não exige Bootstrap | `lite` reimplementa 777 linhas de `ui_template` + 203 linhas de UI (Modal/Dropdown/Tooltip) + 849 de SCSS |

**Veredito:** fácil de integrar, caro de evoluir; HTML como verdade é dívida, não atalho. O ganho é
o caminho curto até um WYSIWYG que funciona (um `<div>`, jQuery, uma chamada); o custo aparece
quando o produto precisa de conteúdo previsível, undo barato, XSS contido e teclado funcional —
exatamente os eixos em que o Summernote delega ao browser.

## 7. O que copiar, adaptar ou evitar

**Copiar (ideias, não código):**

- o **contrato de integração**: “um elemento entra, um valor HTML sai”, com leitura/escrita
  explícitas (`code()`) e evento de mudança com debounce de 10 ms
  (`src/js/module/Editor.js:410-412`);
- o **`WrappedRange`** como álgebra de seleção *se* um dia precisarmos operar sobre DOM real
  (normalização de pontos visíveis, `insertNode`, `deleteContents` — `src/js/core/range.js:219-298`,
  `:425-465`, `:548-600`);
- o **bookmark** de seleção para preservar cursor em re-render/undo (`range.js:718-737`);
- a lista de **callbacks** como checklist de eventos que um host espera (change, focus, blur,
  paste, image upload, dialog).

**Adaptar:**

- o padrão de **módulos + `invoke` por string** — trocando string por comandos tipados;
- o **plugin** `function(context)` com `memo('button.x')` e `events`
  (`public/plugin/hello/summernote-ext-hello.js:16-70`) — bom modelo mental para extensão de UI;
- a **code view** como escape hatch, mas com parser/serializer de verdade em vez de regex.

**Evitar:**

- `contenteditable` como fonte de verdade: nossa decisão é Rust headless, então o modelo tem de
  ser canônico e serializável, não o DOM vivo;
- `execCommand` (obsoleto, sem garantia de output);
- snapshots de HTML para undo (O(n) por tecla, sem merge, sem intenção);
- `purify` por regex: sanitização precisa de allowlist estrutural (nós/atributos), feita no
  servidor e/ou no core;
- CSS que impõe `padding`, `font-family` e 150 classes ao redor do conteúdo
  (probe2 `F1.computedStyle`/`F2.classesInjected`).

## 8. Mapa do dossiê

| Arquivo | Conteúdo |
|---|---|
| `README.md` | este texto: propósito, tese, ganhos/custos, API, veredito |
| `arquitetura.md` | camadas, ciclo de vida, ranges, undo, toolbar, CSS, a11y, plugins, performance |
| `avaliacao.md` | matriz de usabilidade (13 eixos), notas 0–5 com evidência, limites do estudo |
| `codigo/INDEX.md` | origem/licença das cópias e como os arquivos se encaixam |
| `exemplos/README.md` | comandos executados, saídas observadas e o que **não** foi executado |
