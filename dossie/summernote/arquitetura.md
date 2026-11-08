# Arquitetura — Summernote 0.9.1

Base: `ref/summernote/` (commit `ac5460e`, MIT). Todas as linhas citadas são desse clone.
Medições marcadas como **medido** vêm de `exemplos/probe.html`, `exemplos/probe2.html` e
`exemplos/drive.mjs` (Chromium headless, dist 0.9.1 via CDN); detalhes em `exemplos/README.md`.

## 1. Visão em camadas

```
                 $(el).summernote(opts | 'api.nome', args)
                                 │
                    src/js/summernote.js  (plugin jQuery)
                                 │  um Context por elemento
                                 ▼
                    src/js/Context.js     (ciclo de vida, invoke, code())
                    ┌────────────┴─────────────┐
                    ▼                          ▼
     ui_template (bs3/bs4/bs5/lite)      modules[] (Editor, Clipboard,
     renderer → DOM do editor            Codeview, Toolbar, Buttons, …)
                    │                          │
                    └──────────► DOM vivo ◄────┘
                                  (div.note-editable[contenteditable])
```

- O plugin registra `$.fn.summernote` (`src/js/summernote.js:5-45`). O primeiro argumento pode ser
  objeto (init) ou string (API externa): `src/js/summernote.js:12-15`.
- `Context` guarda `layoutInfo` (note/editor/toolbar/editable/codable/statusbar),
  `options`, `modules` e `memos` (`src/js/Context.js:15-24`).
- Os “estilos” (bs3/bs4/bs5/lite) **não são temas CSS**: são implementações da `ui_template`,
  isto é, o *markup* e o comportamento de botão/dropdown/modal/popover, além do SCSS
  (`src/styles/bs4/summernote-bs4.js:5-16`, `:328-330`).

## 2. Ciclo de vida

```js
// src/js/Context.js:30-36
initialize() {
  this.layoutInfo = this.ui.createLayout(this.$note); // insere .note-editor depois do elemento
  this._initialize();                                  // registra e inicializa módulos
  this.$note.hide();                                   // o <textarea>/<div> original some
  return this;
}
```

- `createLayout` monta `editor([toolbar, editingArea([codable, editable]), statusbar])` na ordem,
  ou `airEditor([editingArea([codable, airEditable])])` no air mode
  (`src/styles/bs4/summernote-bs4.js:296-315`).
- O elemento original **não é removido**: permanece escondido e é o alvo de `AutoSync`
  (`src/js/module/AutoSync.js:5-15`) e de `code()` (`src/js/Context.js:98-113`).
- `_initialize` cria id único (`options.id`), resolve o container de popovers e registra
  `options.modules` + `$.summernote.plugins` (`src/js/Context.js:59-81`).
- `destroy` remove módulos em ordem reversa, limpa `memos`, remove o layout e, no
  `removeLayout`, copia o HTML de volta para o elemento e o reexibe
  (`src/styles/bs4/summernote-bs4.js:318-323`, `src/styles/lite/summernote-lite.js:765-770`). Medido:
  após `destroy`, `editorsLeft:1` (restou o da segunda instância), `noteVisible:true`, `noteValue`
  com o HTML editado. Diferença real entre builds: só a `lite` faz `$note.off('summernote')` dentro
  de `removeLayout` (`summernote-lite.js:768`); `bs3/bs4/bs5` deixam os handlers da instância morta
  ligados ao elemento.
- `reset()` limpa o conteúdo (`dom.emptyPara`) e reinicializa (`src/js/Context.js:48-57`), sem
  remover `$note.data('summernote')`.
- Reinicializar sem `destroy` é **no-op**: o plugin só cria `Context` se não houver
  `el.data('summernote')` (`src/js/summernote.js:26-32`).

Consequência: **o ciclo de vida é do DOM**. Não existe “documento” a serializar/desserializar; existe
um layout que encapsula o elemento.

## 3. Módulos e `invoke` (o “sistema de plugins”)

`options.modules` é um mapa nome→classe (`src/js/settings.js:41-70`): `editor`, `clipboard`,
`dropzone`, `codeview`, `statusbar`, `fullscreen`, `handle`, `hintPopover`, `autoLink`, `autoSync`,
`autoReplace`, `placeholder`, `buttons`, `toolbar`, `linkDialog`, `linkPopover`, `imageDialog`,
`imagePopover`, `tablePopover`, `videoDialog`, `helpDialog`, `airPopover`.

Cada módulo pode ter:

- `initialize()` / `destroy()`;
- `shouldInitialize()` (ex.: `Toolbar` não existe em air mode, `src/js/module/Toolbar.js:21-23`);
- `events` — objeto `{'summernote.mousedown': fn, …}` anexado ao elemento por `dom.attachEvents`
  (`src/js/Context.js:150-165`).

A chamada é textual:

```js
// src/js/Context.js:227-243 (resumo)
invoke() {
  const namespace = lists.head(arguments);            // 'editor' | 'toolbar' | 'code'
  const splits = namespace.split('.');
  const moduleName = splits.length > 1 && splits[0];   // 'editor'
  const methodName = splits.length > 1 ? splits.at(-1) : splits[0];
  const module = this.modules[moduleName || 'editor'];
  if (!moduleName && this[methodName]) return this[methodName].apply(this, args); // code, reset, destroy
  else if (module && module[methodName] && module.shouldInitialize())
    return module[methodName].apply(module, args);
}
```

Características e riscos:

- **Sem tipos**: `summernote('codez')` devolve `undefined` sem erro; o mesmo para módulo inexistente.
- **Superfície enorme**: `invoke('editor.<qualquer método público>')` é API. Métodos “internos” como
  `removed`, `resizeTo`, `saveTarget` são acessíveis.
- `memo('button.<nome>', fn)` é o registro de botões (`src/js/Context.js:194-201`), consumido por
  `Buttons.build` (`src/js/module/Buttons.js:801-822`). É o principal ponto de extensão de UI.

## 4. `Editor`: eventos, keymap, comandos

### 4.1 Listeners (o “input loop”)

`Editor.initialize` (`src/js/module/Editor.js:339-445`) liga no `.note-editable`:

| Evento | O que faz |
|---|---|
| `keydown` | dispara callback; **salva snapshot** (`this.snapshot = history.makeSnapshot()`); aplica keymap; se não for keymap, `afterCommand()` (que grava undo) |
| `keyup` | `setLastRange()`, callback |
| `mousedown` | callback |
| `mouseup` | `setLastRange()`, `history.recordUndo()`, callback |
| `paste` | `setLastRange()`, callback (o módulo `Clipboard` trata arquivos) |
| `copy` | callback apenas |
| `input` | se `isLimited(0)`, restaura snapshot (limite de caracteres durante composição) |
| `focus/blur/focusin/focusout/scroll` | callbacks |

Trecho decisivo (`src/js/module/Editor.js:341-375`): o undo é registrado no **keydown**, não no
`input`; e `recordEveryKeystroke` (padrão `false`, `settings.js:141`) controla se teclas que não são
atalho também empilham.

### 4.2 Keymap

`options.keyMap` tem duas tabelas, `pc` e `mac` (`src/js/settings.js:257-319`), com nomes de tecla
mapeados por `keyCode` numérico (`src/js/core/key.js:4-55`). `handleKeyMap`
(`src/js/module/Editor.js:450-480`) monta a string (`CTRL+SHIFT+NUM7`) e invoca o método.

Duas observações verificadas no código:

1. **TAB é caso especial e não executa `tab()`**:
   ```js
   // src/js/module/Editor.js:465-468
   if (keyName === 'TAB' && !this.options.tabDisable) {
     this.afterCommand();
   } else if (eventName) { … }
   ```
   Logo `tabSize` só vale para `invoke('tab')`/`typing.insertTab`, não para a tecla. Medido com
   teclado real: Tab **sai** do editor e o HTML não ganha espaços (CDP `key.tab`); Shift+Tab vai
   para o `BODY` (CDP `key.shiftTab`).
2. **Teclas de edição sem atalho** caem em `key.isEdit` → `afterCommand()` (`:474-479`), que por sua
   vez chama `history.recordUndo()` — daí o undo por tecla.

`keyCode` é legado (não usa `event.key`), o que torna o keymap dependente de layout de teclado e
frágil em IMEs.

### 4.3 Comandos

Dois mecanismos convivem:

1. **`document.execCommand`** para o que o browser implementa (`src/js/module/Editor.js:58-70`):
   `bold`, `italic`, `underline`, `strikethrough`, `superscript`, `subscript`, `justify*`,
   `formatBlock`, `removeFormat`, `backColor`; mais `foreColor`, `styleWithCSS`, `unlink`
   (`:269-279`, `:676-687`, `:908-910`).
2. **Manipulação de DOM própria** para o que o `execCommand` faz mal: listas (`src/js/editing/Bullet.js`),
   tabelas (`src/js/editing/Table.js`), parágrafos (`src/js/editing/Typing.js`), links
   (`Editor.createLink`, `:197-265`), imagens (`:768-793`), formatação por span
   (`src/js/editing/Style.js:70-116`).

O envelope é sempre o mesmo:

```js
// src/js/module/Editor.js:676-699
beforeCommand() {
  this.context.triggerEvent('before.command', this.$editable.html());
  document.execCommand('styleWithCSS', false, this.options.styleWithCSS);
  this.focus();
}
afterCommand(isPreventTrigger) {
  this.normalizeContent();          // .normalize() do DOM: funde nós de texto
  this.history.recordUndo();        // snapshot completo
  if (!isPreventTrigger) this.context.triggerEvent('change', this.$editable.html(), this.$editable);
}
wrapCommand(fn) {
  return function() { this.beforeCommand(); fn.apply(this, arguments); this.afterCommand(); };
}
```

Consequência: **cada comando grava um snapshot do documento inteiro**. Medido: doc de 17 KB, 30
inserções por `execCommand` custaram 32 ms *sem* snapshots (o `input` não empilha); digitar de
verdade empilha 1 snapshot por tecla (3 teclas → pilha 4, CDP `ime.composing`), e um `undo` nesse
doc custou ~14 ms (CDP `perf.undoOne`).

### 4.4 Seleção: `lastRange`

O editor mantém **uma** seleção “lógica” em `this.lastRange`:

- `setLastRange(rng)` sem argumento cria `range.create(this.editable)`; se o resultado não estiver
  dentro de `.note-editable`, usa `createFromBodyElement` (`src/js/module/Editor.js:558-576`);
- `getLastRange()` devolve a última ou cria (`:578-583`);
- `saveRange`/`restoreRange` são usados por diálogos (`LinkDialog.show`,
  `src/js/module/LinkDialog.js:184-195`): ao abrir um modal o foco sai do editor, então a seleção
  precisa ser guardada em memória.

Isso explica por que os botões usam `tabindex="-1"` (não roubar o foco/seleção) e por que
“clicar no botão e voltar” nem sempre preserva a seleção como o usuário espera.

Comportamento medido de comandos dependentes de range:

- `insertText` sem caret funciona (insere no início do primeiro parágrafo): `<p>Xabc</p>`;
- `pasteHTML` sem caret **não faz nada** para conteúdo inline (`<b>Y</b>`); com caret, funciona
  `<p>abc<b>Y</b></p>` (probe2 `A2`/`A4`).

## 5. Ranges (`core/range.js`, 929 linhas)

`WrappedRange` guarda 4 campos (`sc, so, ec, eo` — `src/js/core/range.js:119-138`) e oferece:

| Grupo | Métodos | Uso |
|---|---|---|
| Conversão | `nativeRange()`, `select()`, `getPoints()`, `getClientRects()` | falar com `document.createRange`/`Selection` |
| Normalização | `normalize()` (`:219-298`) | mover o ponto para um nó **visível** (idiossincrasias Chrome/FF) |
| Travessia | `nodes(pred)`, `ancestor`, `expand`, `isOnCell/List/Anchor` | saber “onde estou” |
| Edição | `splitText`, `deleteContents` (`:425-465`), `wrapBodyInlineWithPara` (`:498-546`), `insertNode` (`:548-570`), `pasteHTML` (`:571-599`) | colar, inserir mídia, formatar |
| Persistência de seleção | `bookmark(editable)` (`:718-735`) → `{s:{path,offset}, e:{…}}`; `dom.fromOffsetPath` reconstrói | undo e re-render |
| Palavras | `getWordRange`, `getWordsRange`, `getWordsMatchRange` | autolink e hint/emoji |

Há fallback para IE (`textRangeToPoint`/`pointToTextRange`, `:16-118`), sinal de que a base é antiga.

`pasteHTML` é o ponto mais delicado: ele **parseia o markup com o próprio browser**
(`$('<div>').html(markup)`), insere os filhos em ordem (reversa quando `so >= 0`) e devolve os nós
inseridos (`:571-599`). Ou seja: sanitização = o que o browser aceita + o regex de `purify`.

## 6. Undo/redo = snapshots de HTML

`src/js/editing/History.js` (copiado em `codigo/03-History.js`):

```js
makeSnapshot() {                       // :12-20
  return {
    contents: this.$editable.html(),   // documento inteiro, como string
    bookmark: range.create(this.editable).bookmark(this.editable),
  };
}
applySnapshot(snapshot) {              // :22-30
  this.$editable.html(snapshot.contents);                       // innerHTML
  range.createFromBookmark(this.editable, snapshot.bookmark).select(); // restaura cursor
}
recordUndo() {                         // :110-126
  this.stackOffset++;
  if (this.stack.length > this.stackOffset) this.stack = this.stack.slice(0, this.stackOffset);
  this.stack.push(this.makeSnapshot());
  if (this.stack.length > this.context.options.historyLimit) { this.stack.shift(); this.stackOffset -= 1; }
}
```

Propriedades:

- **Sem intenção, sem merge, sem operação inversa.** Digitar 3 teclas = 3 snapshots completos.
- Limite padrão 200 (`settings.js:142`) → até 200 × tamanho do HTML (17 KB ⇒ ~3,4 MB só de strings,
  sem contar o bookmark e o custo de GC).
- `code()` **não** empilha nada; medido no probe1: aplicar `code('<p>one</p>')`, `code('<p>one two</p>')`
  e `code('<p>one two three</p>')` e chamar `undo` retornou o HTML do estado **anterior** à instância
  (o snapshot inicial), não o passo intermediário (`history.undo`).
- `undo()` chama `applySnapshot` → `innerHTML` → **recria todo o DOM** e re-seleciona via bookmark.
  Isso invalida referências (`$target` de imagem, por exemplo) e força reflow.

## 7. Toolbar, botões, popovers e o overlay de mídia

### 7.1 Toolbar

- O padrão é uma matriz de grupos/nomes (`src/js/settings.js:78-112`):
  `['style', ['style']], ['font', ['bold','underline','clear']], …`
- `Toolbar.initialize` chama `buttons.build(toolbar, options.toolbar)`, esconde a toolbar se a lista
  estiver vazia, reanexa em `toolbarContainer` e assina `keyup/mouseup/change` para atualizar estado
  (`src/js/module/Toolbar.js:25-56`).
- `followingToolbar` implementa uma toolbar “sticky” calculando offsets no scroll e aplicando
  `position: fixed` **inline** (`src/js/module/Toolbar.js:58-105`) — JS de layout dentro do editor.

### 7.2 Botões

- Cada botão é registrado em `memo('button.<nome>', factory)` (`src/js/module/Buttons.js:230+`) e
  construído por `ui.button(...)` → `renderer` → jQuery (`src/js/renderer.js:11-46`).
- O clique usa `Context.createInvokeHandlerAndUpdateState('editor.bold')`, que faz
  `preventDefault()` e invoca o método (`src/js/Context.js:212-224`).
- Estado visual: `updateCurrentStyle` pergunta ao browser
  (`document.queryCommandState('bold')` etc.) e ao estilo computado
  (`src/js/editing/Style.js:124-160`) e aplica classes `active`
  (`src/js/module/Buttons.js:824-898`). **Não** há `aria-pressed`.
- `Style.current()` depende de `queryCommandState`, que reflete o estado do **browser**, não de um
  modelo — outro sintoma de “o DOM é a verdade”.

### 7.3 Popovers e diálogos

- Popovers de imagem/link/tabela são posicionados por cálculo de offset e reagem a
  `mousedown/keyup/scroll/change` (`src/js/module/ImagePopover.js`, `LinkPopover.js`,
  `TablePopover.js`). O `bs4` delega o modal ao JS do Bootstrap
  (`onDialogShown`/`showDialog` → `$dialog.modal('show')`, `src/styles/bs4/summernote-bs4.js:280-296`).
- A build `lite` implementa `ModalUI` (30 linhas), `DropdownUI` (72) e `TooltipUI` (101) do zero em
  `src/styles/lite/js/`; `summernote-lite.js` é o `ui_template` (777 linhas) que os usa.
- **`Handle`** (135 linhas) é o overlay de redimensionamento de imagem: cria
  `.note-handle > .note-control-selection` com 4 alças, posiciona com `getBoundingClientRect()` e
  durante o drag aplica `editor.resizeTo` + `afterCommand` (`src/js/module/Handle.js:30-120`).
  Medido: a seleção usa coordenadas absolutas dentro de `.note-editing-area`, então qualquer CSS do
  host que mude posicionamento afeta o overlay.

## 8. Code view e “purify”

- `activate()` copia `dom.html($editable)` para o `<textarea class="note-codable">`, adiciona
  `codeview` ao `.note-editor` e, se houver `window.CodeMirror`, instancia o editor
  (`src/js/module/Codeview.js:100-145`).
- `deactivate()` lê o textarea, aplica `purify` e **substitui** o HTML do editable
  (`:153-174`).
- `purify` (`:72-95`) remove tags por regex (`codeviewFilterRegex`, `settings.js:241`) e remove
  `<iframe>` cujo `src` não esteja na allowlist de domínios (`settings.js:244-256`).

Medições (probe2):

| Entrada | Saída |
|---|---|
| `<p>ok</p><script>window.__a=1</script>` | `<p>x</p><p>ok</p><p>window.__a=1</p>` (tag some, código vira texto) |
| `<style>body{display:none}</style>` | `<p>xbody{display:none}</p>` |
| `<img src="nope.png" onerror="window.__b=1">` | **atributo preservado** |
| `<a href="javascript:window.__c=1">` | **preservado** |
| `<iframe src="https://evil.example/x">` | preservado quando vem por `code()` (purify só roda no codeview/pasteHTML) |
| `<iframe src="https://www.youtube.com/embed/abc123">` | preservado (whitelist) |

E o teste decisivo (CDP): gravar `<img src="nope.png" onerror="window.__pwned='yes'">` via `code()`
faz o handler **executar** (`{"pwned":"yes"}`). O README do projeto reconhece o problema e empurra a
responsabilidade para o servidor (`README.md:71`).

## 9. Clipboard, imagem e drag-and-drop

- `Clipboard` só intercepta `paste` para: (a) arquivos de imagem → `insertImagesOrCallback`;
  (b) exceder `maxTextLength` → `preventDefault` (`src/js/module/Clipboard.js:24-44`). HTML colado
  segue o fluxo nativo do browser; não há filtro de colagem.
- `copy` é apenas um evento para callback (`src/js/module/Editor.js:390-392`); medido: 1 evento
  disparado no Ctrl+C do browser.
- Imagens: `readFileAsDataURL` + `createImage` (`src/js/core/async.js:11-47`), inseridas via
  `range.insertNode` com `width` em px calculado de `$editable.width()`
  (`src/js/module/Editor.js:768-793`). Medido: `<img src="data:image/gif;base64,…" style="width: 1px;">`
  — **sem `alt`**, sem `srcset`, sem lazy.
- `Dropzone` cobre o editor com `.note-dropzone` e aceita arquivos por drag (`src/js/module/Dropzone.js`).

## 10. CSS e acoplamento visual

Três artefatos distintos:

1. `src/styles/summernote/{common,elements,font}.scss` — base compartilhada: layout, dropzone,
   statusbar, toolbar, popover, e a fonte de ícones `summernote.woff` (classe `.note-icon-*`).
2. `src/styles/{bs3,bs4,bs5}/summernote-*.{js,scss}` — markup que **usa classes do Bootstrap**
   (`card`, `card-header`, `btn`, `dropdown-menu`, `modal`) e delega comportamento ao
   `bootstrap.bundle.js`.
3. `src/styles/lite/` — mesmo papel sem Bootstrap: SCSS próprio (849 linhas), `ui_template` de 777
   linhas (`summernote-lite.js`) e 203 linhas de UI própria (`lite/js/{Modal,Dropdown,Tooltip}UI.js`).

O que o editor impõe ao redor do conteúdo (medido, probe2 `F1`/`F2`):

- 150 nomes de classe distintos injetados no DOM (`.note-*`, `.dropdown-*`, `.btn-*`);
- `.note-editable { padding: 10px; overflow: auto; word-wrap: break-word }`
  (`src/styles/summernote/common.scss:92-100`) com `outline: none` em `:114`; no CSS minificado:
  `.note-editable{overflow:auto;padding:10px;…}`;
- `.note-frame { border-radius: 4px; box-sizing: border-box; color:#000; font-family: sans-serif }`
  (`src/styles/lite/scss/common.scss:1-6`; `$font-family: sans-serif` em `lite/scss/variables.scss:11`) → o conteúdo herda **`sans-serif`** mesmo que o `body` use
  outra fonte (medido: `body` = `"Times New Roman"`, editable = `sans-serif`);
- margens de `<p>` **não** são resetadas: `margin: 16px 0px` (padrão do UA) no conteúdo;
- `.note-frame` aplica `border: 1px solid` e o `.note-editor` é `position: relative` — o host não
  escolhe a moldura.

Ou seja: “lite” reduz a dependência de Bootstrap, mas **não** é “zero CSS”. O conteúdo é estilizado
por CSS global do editor e por classes estruturais.

## 11. Acessibilidade

O que existe:

- `role="textbox" aria-multiline="true"` no editable, `role="toolbar"` na toolbar, `<output
  role="status" aria-live="polite">` no status (`src/styles/bs4/summernote-bs4.js:5-16`);
- `aria-label`/`title` nos botões que têm tooltip (`src/styles/lite/summernote-lite.js`, função
  `button`);
- foco real no `contenteditable`, Tab nativo para sair.

O que falta (e foi medido):

| Problema | Evidência |
|---|---|
| Toolbar quase inalcançável por teclado | 171 botões na toolbar, **4** com `tabIndex >= 0` (todos `.note-color-reset`/`.note-color-select`); os demais têm `tabindex="-1"` (CDP `a11y.toolbarTabOrder`; markup em `src/styles/bs4/summernote-bs4.js:222`, `:247`) |
| Sem `aria-pressed`/`aria-expanded` nos toggles | `toggleBtnActive` só faz `toggleClass('active')` (`src/styles/bs4/summernote-bs4.js:276-278`) |
| Imagens sem `alt` | medido `<img src="data:…" style="width: 1px;">` (probe1 `image.dataUrl.after`) |
| Overlay de redimensionamento não é acessível | `.note-control-*` são `div` com mousedown, sem equivalente de teclado (`src/js/module/Handle.js:30-76`) |
| Sem região viva para mudanças de conteúdo | `aria-live` só no status output (mensagens de “sem seleção”, etc.) |

O `tabindex="-1"` é uma decisão consciente para **preservar a seleção** (um botão focável tiraria o
foco do editable ao clicar), mas o custo é que o teclado não opera a formatação.

## 12. Extensibilidade real

```js
// public/plugin/hello/summernote-ext-hello.js:16-70 (resumo)
$.extend($.summernote.plugins, {
  hello: function(context) {
    const ui = $.summernote.ui;
    context.memo('button.hello', function() {
      return ui.button({ contents: 'Hello', tooltip: 'hello',
        click: () => context.invoke('editor.insertText', 'hello') }).render();
    });
    this.events = { 'summernote.init': fn, 'summernote.keyup': fn };
    this.initialize = function() { /* cria DOM do plugin */ };
    this.destroy = function() { /* limpa */ };
  },
});
```

Pontos fortes:

- qualquer plugin pode adicionar botão, evento e comando;
- `options.buttons` permite sobrescrever/adicionar botões sem plugin (`src/js/Context.js:66-73`);
- `options.modules` permite substituir módulos inteiros (ex.: trocar `imageDialog`).

Limites (para o nosso caso):

- **não há schema**: o plugin não declara “este nó é X”; a validação é sempre ad hoc sobre o DOM;
- os plugins vivem em `$.summernote.plugins` global, então dois editores na página compartilham o
  registro (a instância vem no `context`);
- não há versionamento de plugin, nem API de migração de conteúdo.

## 13. Serialização e persistência

- `code()` lê `editable.html()` — o **innerHTML serializado pelo browser**, com:
  - whitespace e comentários preservados (medido: `'<p>  espaços  </p>\n\n<!-- comentário -->'`
    volta idêntico);
  - normalização implícita de HTML inválido (ver §3.2 do README);
  - atributos de estilo inline gerados por `execCommand` (`style="font-weight: bold;"`).
- Quando a origem é `<textarea>`, `AutoSync` mantém `value` atualizado a cada `summernote.change`
  (`src/js/module/AutoSync.js:5-15`). Medido: após `code('<p>persistido</p>')`, `val()` já é o HTML.
- **Não existe formato canônico**: salvar e recarregar o mesmo conteúdo pode produzir strings
  diferentes se o browser/opções mudarem (ex.: `<b>` vs `<span style="font-weight:bold">`).

## 14. Performance

Estrutura:

- um único DOM grande, sem virtualização e sem modelo paralelo → edição é “grátis”, mas cada
  comando pode varrer/normalizar o DOM;
- `afterCommand` chama `normalizeContent()` (`this.$editable[0].normalize()`) e
  `history.recordUndo()` (`src/js/module/Editor.js:690-695`): **todo comando custa O(documento)**;
- **todo `keydown` serializa o documento**: `this.snapshot = this.history.makeSnapshot()` roda antes
  de qualquer decisão (`src/js/module/Editor.js:346-347`), com `$editable.html()` + bookmark, e é
  descartado na tecla seguinte. Esse custo não aparece no undo (`recordEveryKeystroke: false`) mas
  é pago em toda digitação;
- imagens em base64 dentro do próprio HTML: um documento com 10 imagens de 200 KB = 2,6 MB de
  string, duplicada em cada snapshot de undo;
- quatro modais e três popovers são construídos no init, mesmo sem uso (probe1 `init.layout`);
- `followingToolbar` recalcula layout no scroll (`src/js/module/Toolbar.js:58-105`).

Medições (Chromium headless, uma amostra; **não** é benchmark):

| Operação | Documento | Tempo |
|---|---|---|
| `code()` (leitura) | ~17 KB, 300 `<p>` | 0,25–0,26 ms/leitura |
| `undo()` (aplica snapshot) | ~17 KB | **13,5–13,9 ms** (2 execuções) |
| 30 inserções via `execCommand` (sem snapshots) | ~17 KB | 31,3–32,1 ms (~1,0–1,1 ms cada) |
| `recordUndo()` x50 (`perf.recordUndo50`) | doc pequeno | 0,04 ms/snapshot (não escala com o doc) |

Sem números medidos para: documentos de 1 MB, imagens grandes, mobile, Safari/Firefox.

## 15. Erros, diagnóstico e armadilhas observadas

| Sintoma | Causa no código | Como diagnosticar |
|---|---|---|
| `undefined` ao chamar a API | `invoke` textual não valida nomes (`src/js/Context.js:227-243`) | conferir o nome do módulo/método |
| Conteúdo “some” ao chamar `code('')` | `emptyPara` e bookmark antigos apontando para nós removidos | reler o código após cada `code()` |
| Undo volta “errado” após importar conteúdo | `code()` não empilha snapshot | usar `history.commit()`/`reset()` conscientemente |
| `pasteHTML` não insere inline | sem caret válido (`A2` no probe2) | posicionar caret/`focus` antes |
| Erro `Cannot read properties of null (reading 'appendChild')` em `pasteHTML` | range apontando para nó destacado após `destroy`/re-init (observado no probe1, passo `paste.scriptTag`) | não reutilizar instâncias; recriar a seleção |
| XSS no conteúdo salvo | sem sanitização real (`purify` é regex) | sanitizar no servidor e/ou allowlist |
| HTML diferente do esperado | `execCommand`/parser do browser | fixar `styleWithCSS`, normalizar na saída |
| Reinit silencioso | `if (!$note.data('summernote'))` ignora as novas opções (`src/js/summernote.js:26-32`) | chamar `destroy` antes de reconfigurar |
| Opção “não faz nada” | opções desconhecidas são ignoradas; o nome é `tabSize`, não `tabsize` (`src/js/settings.js:128`) | conferir a grafia na doc; não há validação |
| Handlers duplicados após `destroy` nas builds `bs*` | `removeLayout` não faz `$note.off('summernote')` (existe só na lite, `summernote-lite.js:768`) | usar a build `lite` ou limpar eventos manualmente |

## 16. O que isso significa para o `rich-editor-auy`

1. **Não copiar o modelo**: o “modelo” aqui é o DOM vivo. Nosso core Rust precisa de árvore
   canônica, validação e serialização determinística — exatamente o que o Summernote delega ao
   browser.
2. **Copiar a ergonomia de integração**: init de 1 chamada, leitura/escrita explícitas, evento de
   mudança com debounce e callbacks cobrindo foco/blur/paste/imagem/diálogo.
3. **Copiar (com cautela) a álgebra de ranges**: `normalize`/`insertNode`/`deleteContents` e o
   `bookmark` são um catálogo de casos de borda reais (Chrome vs Firefox, bordas de bloco, nós
   vazios) que qualquer editor DOM enfrenta.
4. **Evitar**: snapshots de HTML para undo; regex como sanitizador; `execCommand`; CSS que impõe
   tipografia e 150 classes; toolbar inacessível por teclado; imagens sem `alt`.
