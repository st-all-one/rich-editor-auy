# Avaliação — Summernote 0.9.1 (`ref/summernote/`, commit `ac5460e`, MIT)

Notas na rubrica de `dossie/_RUBRICA.md`, sempre com evidência. “Medido” = executado em
Chromium headless pela sonda deste dossiê (`exemplos/probe.html`, `probe2.html`, `drive.mjs`);
comandos e saídas em `exemplos/README.md`.

## 1. Matriz de cobertura de usabilidade (meta ≥85%)

| # | Eixo | Status | Evidência |
|---|---|---|---|
| 1 | Instalação e pré-requisitos | **verificado em doc + execução** | `README.md:21-38` (jQuery + CSS/JS); dist `summernote-lite.min.{js,css}` carregada por CDN e usada nas sondas; `package.json` **sem `peerDependencies`** → dependência de jQuery não é declarada ao npm |
| 2 | Inicialização mínima | **verificado em execução** | `exemplos/minimal.html` e `cdp.html`: `$('#note').summernote({...})` cria `.note-editor note-frame`; `noteHidden:true` (probe1 `init.layout`) |
| 3 | Edição básica (texto, parágrafo, B/I/link) | **verificado em execução** | digitação real → `<p>alpha betaXYZ</p>`; Enter → `<p>…</p><p><br></p>`; bold → `<b>beta</b>`; itálico por Ctrl+I → `<i>gamma</i>`; link → `<a href="http://example.com" target="_blank">` |
| 4 | Seleção, teclado, undo/redo, clipboard, IME | **verificado em execução (parcial)** | seleção por `Range` + `keyup` funciona; Tab real **sai** do editor; Ctrl+Z restaura após digitação; undo por snapshot; IME (`Input.imeSetComposition`) insere “に”/“にほん” sem snapshot intermediário; *clipboard*: só o evento `copy` foi verificado (1 disparo); **colagem real de HTML do SO não verificada** |
| 5 | Schema e restrições de conteúdo | **verificado no código + execução** | não há schema: `code()` só faz `innerHTML` (`Context.js:98-111`); único limite é `maxTextLength` (`Editor.js:489-508`, `Clipboard.js:24-44`); HTML inválido é “consertado” pelo browser (probe1 `code.normalization`) |
| 6 | Custom node/mark/plugin | **verificado em doc + código** | `public/plugin/hello/summernote-ext-hello.js:16-70`; `options.modules` (`settings.js:41-70`); `memo('button.x')` (`Context.js:194-199`); **não existe** extensão de nó/mark com validação |
| 7 | Persistência e serialização | **verificado em execução** | `code()` lê/escreve `innerHTML`; `<textarea>` sincronizado por `AutoSync` (`AutoSync.js:5-15`); medido: `val()` = HTML após editar; comentários/whitespace preservados |
| 8 | Renderização e não interferência no CSS | **verificado em execução** | 150 classes `note-*`/`btn-*`/`dropdown-*`; `.note-editable{padding:10px;overflow:auto}`; `.note-frame{font-family:sans-serif}` (medido: `body` = Times, conteúdo = sans-serif); nenhum shadow DOM/iframe de isolamento |
| 9 | Acessibilidade | **verificado em execução + código** | `role="textbox"/"toolbar"`, `aria-multiline`, `<output role="status" aria-live="polite">`; medido **4 de 171** botões tabuláveis; sem `aria-pressed`; imagem sem `alt` |
| 10 | Colaboração | **verificado no código (ausente)** | nenhum módulo/opção de colaboração em `settings.js`; sem OT/CRDT/diff; o repo não tem servidor |
| 11 | Performance e limites | **medido parcialmente** | doc 17 KB/300 `<p>`: `code()` 0,24–0,26 ms; `undo()` 13,5–13,9 ms; 30 inserções `execCommand` 31–34 ms (sem snapshot). 1 snapshot de undo por tecla + **1 `makeSnapshot()` em todo `keydown`** (`Editor.js:346-347`), limite 200. **Sem medição** em >1 MB, mobile, Firefox/Safari |
| 12 | Erros, migração e diagnóstico | **verificado em execução + código** | API textual devolve `undefined` para nome errado (`Context.js:227-243`); `pasteHTML` sem caret é no-op; erro `Cannot read properties of null (reading 'appendChild')` reproduzido em `pasteHTML` após `destroy`/re-init (probe1); `CHANGES.md` (27,8 KB) documenta mudanças, não há guia de migração |
| 13 | Licença e maturidade | **verificado em doc/git** | MIT (`LICENSE:1-3`); 0.9.1; commit de 2026-01-04; `.github/` com CI; `vitest` com browser (`vitest.config.js:5-16`); `public/lang` com 46 idiomas |

**Cobertura:** 13/13 eixos com evidência; 10 plenamente verificados e 3 parciais
(#4 colagem de HTML do SO, #11 escala, #6 ausência de schema é ela mesma a evidência).
Adotando 1,0 para pleno e 0,5 para parcial: `(10 + 1,5) / 13 = 88,5%` ≥ 85%.
O que **não** foi feito: rodar `yarn test`/`yarn build` (sem `node_modules`, ver §5).

## 2. Notas por dimensão (0–5)

| Dimensão | Nota | Evidência e justificativa |
|---|---|---|
| **Usabilidade de integração** | **4** | Init de uma chamada e `<textarea>` funcional como formulário (`README.md:52-56`; medido). 46 idiomas prontos. Desconta: API jQuery/string, CSS próprio, e a build `bs*` exigir Bootstrap CSS+JS (`bs4 js:280-296` delega modal ao Bootstrap) → 5 exigiria “erros claros/ótima DX”, que não há (nomes errados falham em silêncio). |
| **API** | **3** | Superfície única (`$.fn.summernote`) com dispatch textual por namespace (`Context.js:227-243`) e JSDoc nos módulos; cobre init, `code()`, `invoke`, `isEmpty`, `destroy`, `reset` e callbacks (`settings.js:213-232`). É **utilizável**, não “bem tipada”: sem `.d.ts`, sem validação (nome errado → `undefined`), opção grafada errado é ignorada, reinit é no-op, e métodos internos (`removed`, `resizeTo`) ficam expostos. |
| **Neutralidade de estilo** | **1** | Exige o pacote de CSS/tema para ser usável e injeta 150 nomes de classe, `padding:10px`, `font-family:sans-serif` e borda (medido; `common.scss:92-100`, `:114`, `lite/scss/common.scss:1-6`). Não há modo “sem estilo”: a “lite” substitui Bootstrap por 849 linhas de SCSS + 203 de UI própria, mas continua impondo markup e tipografia ao conteúdo. |
| **Modelo de dados** | **1** | Não há modelo: o documento é o DOM/innerHTML (`Context.js:98-113`; `History.js:12-20`). Predicados de `nodeName` (`dom.js:80-140`) não formam schema. Sem normalização, sem validação, sem transação. Fica acima de 0 apenas porque a álgebra de `WrappedRange` (`range.js`) dá alguma estrutura operacional. |
| **Extensibilidade** | **3** | Plugins de primeira classe para **UI/comando** (`$.summernote.plugins`, `memo`, `events`, `options.modules`, `options.buttons`) com exemplo real (`public/plugin/hello`). Falta o essencial para nós/marks: não há declaração de nó, schema, parser/serializer por nó, nem negociação de conteúdo. |
| **Performance** | **3** | Aceitável em documentos pequenos (0,25 ms para ler 17 KB), mas todo comando é O(documento): `normalize()` + snapshot de HTML (`Editor.js:690-695`); `keydown` já serializa o documento inteiro (`Editor.js:346-347`); `undo` recria o DOM (13,5–13,9 ms/17 KB); imagens base64 duplicam payload em cada snapshot; 4 modais + 3 popovers criados no init. Sem virtualização, sem benchmark no repo. |
| **Acessibilidade** | **2** | Base ARIA mínima presente (`role`, `aria-multiline`, `aria-live`), mas toolbar inacessível por teclado (4/171 tabuláveis, por `tabindex="-1"`), toggles sem `aria-pressed`, imagens sem `alt`, overlay de redimensionamento só com mouse. Tab sai do editor (bom), mas não há atalho para alcançar comandos. |
| **Documentação** | **3** | README curto + site + 30 arquivos em `examples/` + `CHANGES.md` detalhado; `src/styles/README.md` explica as builds. Faltam: referência de API no repo, guia de migração, limites/segurança documentados além do aviso de sanitização (`README.md:71`). |
| **Manutenção** | **3** | Projeto **ativo** (commit de 2026-01-04), CI (`.github/workflows/{build,codeql-analysis,npm-publish}.yml`), 17 specs em browser (`test/**`, `vitest.config.js`), 900 linhas de `CHANGES.md` e 46 locales. Desconta muito: `execCommand` obsoleto, `keyCode` legado, sem tipos, jQuery obrigatório e não declarado, placeholders (`@@VERSION@@`) e TODO/FIXME no core; a suíte não cobre a11y, XSS nem undo. |
| **Adequação ao `rich-editor-auy`** | **1** | Quase nada é transportável: o oposto arquitetural do alvo (DOM como verdade vs. core Rust headless; CSS imposto vs. zero CSS; `execCommand` vs. comandos determinísticos; strings vs. árvore canônica). O valor é **conceitual e negativo** — catálogo de casos de borda de DOM/seleção e contraexemplo do que não fazer. Não é referência de implementação nem de modelo. |

## 3. O que o usuário realmente experiencia

| Momento | Experiência observada |
|---|---|
| Instalar | Copiar 2 URLs (jQuery + dist) ou `npm i summernote` e usar `dist/*`; o pacote **não avisa** que jQuery é obrigatório (sem `peerDependencies`). |
| Inicializar | `$('#x').summernote()`; o elemento original é escondido, o layout aparece depois dele; 4 modais já existem no DOM. |
| Primeira edição | Digitação normal; Enter cria `<p>`; listas e tabelas funcionam; formatação usa `execCommand`. |
| Formatar | Botões mudam de estado via `queryCommandState`; `<b>` ou `<span style>` conforme `styleWithCSS`; sem `aria-pressed`. |
| Selecionar/teclado | Tab sai do editor; atalhos Ctrl+B/I/U/K/Z/Y; toolbar não é alcançável por Tab; `Host+click` de imagem mostra overlay de resize. |
| Undo | Por snapshot: funciona para digitação, mas **não** desfaz `code()`; em documento grande, cada undo recria o DOM. |
| Persistir | `code()`/`textarea.value` = innerHTML; sem formato canônico; XSS do conteúdo salvo é responsabilidade do servidor. |
| Estilizar | Precisa sobrescrever `.note-*`; conteúdo herda `sans-serif` do `.note-frame`; 150 classes no DOM. |
| Erro | Chamada inválida → `undefined`; opção grafada errado (`tabsize`) é ignorada; reinit sem `destroy` é no-op silencioso; `pasteHTML` sem caret → nada; re-init + `pasteHTML` → `TypeError`. |
| Performance | OK até dezenas de KB; a partir daí, comandos e undo custam proporcionalmente ao documento. |

## 4. Achados que mudam decisão de design

1. **`code()` invalida o histórico** (medido): importar conteúdo por API e depois `undo` volta ao
   estado de init. Qualquer host que sincronize conteúdo precisa de `history.commit()` explícito.
2. **Sanitização é regex e não cobre atributos** (medido): `onerror` executa ao recarregar HTML.
   Isso é decisão de arquitetura, não bug isolado — não existe allowlist estrutural.
3. **Undo por snapshot é O(n) por tecla** (medido): 3 teclas → 3 snapshots; 17 KB → ~14 ms por undo.
4. **A “lite” não é zero-CSS**: 150 classes + `padding`/`font-family` impostos.
5. **Tab é nativo, mas a toolbar não é alcançável**: 4/171 tabuláveis — a decisão de preservar
   seleção custou o teclado.
6. **`insertText` e `pasteHTML` têm pré-condições diferentes de seleção** (um funciona sem caret,
   outro não): API sem contrato explícito.
7. **Não há validação de opções nem de reinit** (verificado no código): `.summernote(opts)` num
   elemento já inicializado não faz nada (`summernote.js:26-32`) e nomes de opção errados passam
   calados — inclusive o `tabsize` que o próprio exemplo deste dossiê usou antes da revisão
   (correto: `tabSize`, `settings.js:128`).
8. **Todo `keydown` serializa o HTML** (`Editor.js:346-347`), mesmo com `recordEveryKeystroke:false`:
   o custo por tecla é O(documento) antes de qualquer decisão de undo — o pior lugar possível.

## 5. Limites deste estudo (o que não foi verificado)

- **Não rodei `yarn test` nem `yarn build`**: o clone não tem `node_modules` e não instalei
  dependências (instrução do projeto). Portanto: nenhuma asserção sobre a suíte passar; nenhuma
  verificação da build a partir do `src/` — usei a dist publicada `0.9.1`, que corresponde à
  versão do `package.json`.
- **Não verifiquei outros browsers** (Firefox/Safari) nem mobile/touch: os caminhos
  `env.isFF`, `env.isSafari`, `isSupportTouch` (`src/js/core/env.js:1-95`) existem, mas não foram
  exercitados.
- **Colagem real do sistema operacional** (HTML do Word/Docs) não foi testada; só o handler e o
  caminho de arquivos.
- **IME real de teclado** (coreano/japonês) não foi usado; simulei composição via CDP
  (`Input.imeSetComposition`), o que cobre o fluxo DOM, não o comportamento de cada IME.
- **Medições de performance** são uma única amostra em headless, não benchmark; servem para ordem de
  grandeza (undo ≫ leitura).
- **Acessibilidade com leitor de tela** não foi testada; apenas atributos ARIA e ordem de foco.
- **`revisao.md`**: escrito pelo revisor independente; todos os números centrais foram reexecutados
  (3 sondas), com os mesmos resultados dentro da variação de medição.

## 6. Confiança final

**86%.** Base: leitura de código com caminho:linha nos pontos centrais (Context, Editor, History,
range, Codeview, CSS, plugins), execução real da dist 0.9.1 com três sondas **reexecutadas pelo
revisor** (init, API, teclado real, Tab/Shift+Tab, IME, Ctrl+Z, XSS, performance, a11y) e 13/13
eixos da matriz com evidência. Os números centrais se reproduziram: `undo` 13,5–13,9 ms, 150
classes, 171 botões / 4 tabuláveis, doc de 16.990 B, 4 modais + 3 popovers no init. O que impede
90%+: suíte de testes não executada, um único browser, colagem de SO e IME reais não testados, e
amostras de performance em headless (não benchmark).
