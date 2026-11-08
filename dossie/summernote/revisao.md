# Revisão independente — Summernote (`ref/summernote/`, commit `ac5460e`, 0.9.1, MIT)

Revisor independente do explorador. Escopo: `dossie/summernote/` + leitura de `ref/summernote/`.
Nada em `ref/`, `plan/`, `AGENTS.md` ou outros dossiês foi alterado.

## 1. Método

1. Li `dossie/_GUIA.md`, `dossie/_RUBRICA.md`, `dossie/_REVISAO-CHECKS.md` e o dossiê inteiro
   (`README.md`, `arquitetura.md`, `avaliacao.md`, `codigo/INDEX.md`, `exemplos/README.md`).
2. Confirmei cada afirmação central contra `ref/summernote/` (caminho:linha), recalculando os
   números derivados (`wc -l`, `find`, `git log`, `grep`).
3. **Reexecutei as três sondas** com a dist publicada `0.9.1` (chromium-browser headless, rede
   liberada): `probe.html` (`--dump-dom`), `probe2.html` (`--dump-dom`) e `drive.mjs` (CDP com
   teclado real), duas vezes cada para checar variação.
4. Corrigi o que estava errado e registrei o que não pude verificar.

## 2. Números centrais — reexecutados

| Alegação do dossiê | Resultado da revisão | Status |
|---|---|---|
| `undo()` ≈ 13,9 ms (doc de 17 KB) | 13,9 ms; 13,5 ms; 13,8 ms em três execuções | **confirmado** (13,5–13,9 ms) |
| 150 classes injetadas | `F2.classesInjected {"count":150}` | **confirmado** |
| 4 de 171 botões da toolbar tabuláveis | `a11y.toolbarTabOrder {"toolbarButtons":171,"tabbable":4}` | **confirmado** |
| 342 botões no layout / 334 com `tabindex="-1"` | idêntico na reexecução | **confirmado** (população diferente da toolbar; ver §3.1) |
| doc de 16.990 B; 30 teclas = 31–34 ms; `stackLen:1` | 16.990 B; 31,3/34,3 ms; `stackLen:1` | **confirmado** |
| `code()` leitura ≈ 0,25 ms | 0,24–0,26 ms | **confirmado** |
| 3 caracteres → pilha 4 (`ime.composing`) | `stack:4` em todas as execuções | **confirmado** |
| `onerror` executa ao recarregar via `code()` | `{"pwned":"yes"}` | **confirmado** |
| `historyLimit` 200 | `settings.js:142` e `perf.doc.historyLimit:200` | **confirmado** |
| 46 idiomas, 39 arquivos/9.254 linhas JS, lite 777 linhas, 849 SCSS | `ls`, `find`, `wc` | **confirmado** |
| cópias de `codigo/` correspondem ao original | 4 arquivos "completos" batem linha a linha (+8 de cabeçalho) | **confirmado** |
| MIT, commit `ac5460e` (2026-01-04) | `git log -1`, `LICENSE:1-3`, `package.json:5` | **confirmado** |

## 3. Achados

### 3.1 O que estava certo (e agora tem número reexecutado)

- **HTML/`contenteditable` como fonte de verdade**: `Context.code()` é `innerHTML` puro
  (`Context.js:98-113`), sem schema, transação ou normalização. Correto.
- **`execCommand` no núcleo**: `Editor.js:58-70` gera os métodos nativos; `beforeCommand`/
  `afterCommand` envolvem tudo (`Editor.js:676-695`). Correto.
- **Undo por snapshot**: `History.makeSnapshot()` guarda `$editable.html()` + bookmark
  (`History.js:12-20`); limite 200. Correto.
- **XSS**: `purify` é regex de tags (`Codeview.js:72-95`), roda no `deactivate` do code view e em
  `Editor.pasteHTML` (`Editor.js:153`), **nunca** em `code()`. `onerror` executa; `javascript:` e
  `onclick` sobrevivem; `<iframe>` arbitrário entra por `code()`. Correto.
- **A11y fraca**: sem `aria-pressed`/`aria-expanded` em `src/` (grep vazio); `tabindex="-1"` no
  botão base (`lite/summernote-lite.js:40`; `bs4:222`,`:247`); Tab real sai do editor. Correto.
- **Acoplamento visual**: 150 classes, `padding:10px`, `font-family:sans-serif` herdado do
  `.note-frame` (`lite/scss/common.scss:1-6` + `variables.scss:11`). Correto.
- **API**: dispatch textual sem tipos (`Context.js:227-243`); erro de nome → `undefined`. Correto.

### 3.2 O que o explorador não viu (adicionado ao dossiê)

1. **Todo `keydown` serializa o documento.** `this.snapshot = this.history.makeSnapshot()` roda
   antes do keymap (`Editor.js:346-347`), com `$editable.html()` + bookmark, e é descartado na
   tecla seguinte. O custo é O(documento) **por tecla**, mesmo com `recordEveryKeystroke:false`.
   É mais grave que o snapshot de undo, que só ocorre em comandos.
2. **Reinicializar é no-op silencioso.** `summernote.js:26-32` só cria `Context` se não existir
   `el.data('summernote')`; `.summernote({...})` de novo ignora as opções sem erro.
3. **Opções desconhecidas são aceitas em silêncio.** O próprio dossiê usava `tabsize` (inexistente);
   o nome é `tabSize` (`settings.js:128`). Corrigido nos exemplos e no texto.
4. **`destroy` diverge entre builds.** Só a `lite` faz `$note.off('summernote')`
   (`summernote-lite.js:768`); `bs3/bs4/bs5` (`bs4:318-323`) deixam handlers da instância morta
   ligados ao elemento.
5. **Modal/Dropdown/Tooltip da "lite" não estão em `summernote-lite.js`.** Ficam em
   `src/styles/lite/js/{Modal,Dropdown,Tooltip}UI.js` (30/72/101 linhas); os 777 linhas do
   `summernote-lite.js` são o `ui_template`.
6. **`perf.recordUndo50`** (0,04 ms por snapshot em doc pequeno) mostra que a serialização é barata
   por si — o custo vem de multiplicá-la pelo tamanho do documento e pela frequência.

### 3.3 Citações corrigidas

| Antes | Depois | Motivo |
|---|---|---|
| `settings.js:30-56` (módulos) | `:41-70` | bloco real dos módulos |
| `settings.js:99-108` (toolbar padrão) | `:78-112` | matriz completa |
| `settings.js:318-352` (keyMap) | `:257-319` | `pc`/`mac` reais |
| `package.json:3` (versão) | `:4` | `version` está na linha 4 |
| `Codeview.js:72-88` (purify) | `:72-95` | regex + allowlist de iframe |
| `Context.js:98-111` | `:98-113` | fim real do método |
| `Context.js:40-50` (reset) | `:48-57` | `reset()` real |
| `common.scss:92-99` (com `outline`) | `:92-100` + `:114` | `outline: none` está em 114 |
| "lite reimplementa … 777 linhas" | 777 (`ui_template`) + 203 (UI) + 849 SCSS | arquivos separados |
| "726 de SCSS" | 849 de SCSS | contagem real (`wc -l`) |
| `probe1 F1/F2` | `probe2 F1/F2` | passos pertencem à sonda 2 |

## 4. Correções aplicadas nos documentos

- `README.md`: ficha técnica (`package.json:4`); veredito explícito; item de undo com o custo de
  `keydown`; `purify` com escopo e linha corretos; "lite" descrita pelas três partes reais;
  `destroy` com a diferença entre builds; §5 com reinit no-op e opções ignoradas; `tabSize` no
  exemplo; atribuição `probe2` para `F1`/`F2`; tabela de prós/contras alinhada ao SCSS real.
- `arquitetura.md`: citações de `settings.js` corrigidas; `purify` (72-95) e code view; "lite" com
  `lite/js/*`; CSS com `outline` e `$font-family`; §2 com reinit no-op e `off('summernote')` por
  build; §14 com `keydown` serializando o documento e faixas de tempo (13,5–13,9 ms); §15 com
  três armadilhas novas (reinit, opção grafada errado, handlers duplicados).
- `avaliacao.md`: notas ajustadas para **usabilidade 4, API 3, neutralidade 1, modelo 1,
  extensibilidade 3, performance 3, acessibilidade 2, documentação 3, manutenção 3, adequação 1**,
  cada uma com evidência reescrita para o novo valor; dois achados novos; confiança para 86%.
- `exemplos/README.md`: saída do driver re-executada (13,5 ms / 0,24–0,26 ms / 1,04–1,14 ms) com
  aviso de variação; erros de sonda `B5` **e** `G3` documentados; `perf.recordUndo50` e
  `a11y.tabSequence` documentados; nota sobre `tabSize`.
- `exemplos/*.html` e `drive.mjs`: `tabsize` → `tabSize` (opção real). Reexecutado depois da
  mudança: nenhum valor observado mudou (Tab continua saindo do editor, 150 classes, 171/4).
- `codigo/INDEX.md`: `package.json:4`.

## 5. Confiança final

**86%.** Justificativa: todas as afirmações centrais têm caminho:linha; as três sondas foram
**reexecutadas** e os números caros (undo, classes, a11y, bytes, snapshots) se reproduziram dentro
da variação de medição; as cópias de `codigo/` foram conferidas contra o original. Não chega a 90%
porque:

- `yarn test`/`yarn build` **não foram executados** (clone sem `node_modules`); a suíte (17 specs)
  foi lida, não rodada;
- um único browser (Chromium headless), sem Firefox/Safari/mobile;
- colagem real de HTML do SO e IME de teclado não testados (só CDP `Input.imeSetComposition`);
- leitor de tela não testado (apenas atributos e ordem de foco);
- performance é amostra headless, não benchmark (sem doc >1 MB nem imagens grandes).

## 6. Não verificado (herdado e explicitamente fora desta rodada)

| Item | Por quê | Como fechar |
|---|---|---|
| suíte `vitest` | sem dependências instaladas | `cd ref/summernote && yarn && yarn test` |
| build a partir de `src/` | idem | `yarn build` e repetir as sondas contra `dist/` local |
| Firefox/Safari/touch | um browser no ambiente | repetir `probe*.html` com outro binário |
| colagem de Word/Docs | exige clipboard do SO | teste manual + `onPaste` |
| IME real (japonês/coreano) | só composição sintética | teste manual |
| leitor de tela | ambiente headless | NVDA/VoiceOver em `.note-editable` e toolbar |
| XSS por `onerror` em outros vetores (`<svg onload>`, `<iframe srcdoc>`) | não sondado | nova sonda com allowlist estrutural como referência |

## 7. Veredito

**Fácil de integrar, caro de evoluir; HTML como verdade é dívida, não atalho.** O Summernote
entrega em uma chamada o que um time levaria semanas para escrever — e cobra depois: a saída é do
browser, o undo é o documento inteiro (e cada tecla o serializa), a sanitização é regex, o teclado
não alcança a toolbar e o CSS do editor vira o CSS do conteúdo. Para o `rich-editor-auy`, serve como
catálogo de casos de borda de DOM/seleção e como contraexemplo de modelo — não como base.
