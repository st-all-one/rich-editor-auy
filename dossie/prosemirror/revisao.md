# Revisão independente — ProseMirror

Revisor desta rodada. Escopo: `dossie/prosemirror/` conferido contra `ref/prosemirror/`,
`/tmp/pm-upstream/` (tarballs npm) e `/tmp/pm-src/` (clones git de contraste). Nada em `ref/`,
`plan/` ou em outros dossiês foi alterado.

## 1. Fonte: a cópia local é mesmo o meta-repo

Confirmado, não por leitura do README do dossiê, mas pelo repositório:

- `ref/prosemirror` tem 1 commit (`c7f2f1d Add forwarding link`), `README.md:1` aponta para
  `code.haverbeke.berlin`, e `git ls-tree HEAD` lista exatamente 10 entradas (`.github/`,
  `assets/`, `bin/`, `demo/`, `package.json`, `tsconfig.json`, `LICENSE`, `README.md`,
  `CONTRIBUTING.md`, `.gitignore`) — **sem `src/`**.
- `bin/pm.js:9-12` separa `main` (12 módulos) de `mods` (19, incluindo `menu`, `example-setup`,
  `markdown`, `dropcursor`, `test-builder`, `changeset`, `search`); `bin/pm.js:116-127` clona cada
  módulo como repositório irmão. `package.json` do meta-repo é `private: true`, `version 0.0.0`.
- Portanto é correto o dossiê analisar os tarballs npm. **Os 7 sha256 da tabela de
  `codigo/INDEX.md` batem byte a byte com `/tmp/pm-upstream/*.tgz`** (recalculados nesta revisão).

## 2. Cópias em `codigo/`

- `node.ts`, `replace.ts`, `map.ts`, `step.ts`, `state.ts`, `prosemirror.css` são **idênticas** às
  fontes (`sha256` recalculado: `39defcbe…c31f0d2`, `4f2fbf315fba1e70b7802f7c7f5e3478848c5f57f025c2b2169b680153f18b2c`, `ddb1fc54…5624e78`,
  `1080eb08…ab9e1ac`, `081d675b…3f8f7f44`, `1234171efd99cd16d1a6754c0f6586d0d8b69573432aff2e671cb6a26c9eaa57`). Como os números de linha valem
  para cópia e fonte, todas as citações `caminho:linha` dos `.ts` copiados foram checadas em ambos.
- `LICENSE` é byte a byte igual a `prosemirror-model-1.25.12/LICENSE` — MIT, "Copyright (C) 2015-2017
  by Marijn Haverbeke and others". O aviso de origem está no `INDEX.md` e no `README.md`.
- Correção feita: os hashes do `INDEX.md` estavam **truncados de forma inconsistente** e um deles
  estava errado (`…69eaa57`; o real termina em `…c26c9eaa57`). Substituí por sha256 completos.

## 3. Execução

Reexecutei os dois scripts em Node v22.23.2, com `diff` prévio confirmando que
`dossie/prosemirror/exemplos/{demo,extra}.js` são cópias verbatim dos scripts usados em
`/tmp/pm-run`:

- `node demo.js` → saída **idêntica** à documentada (nodeSize 15/13/11, `resolve(6)` =
  `{depth:1, parent:"paragraph", parentOffset:5, start:1, end:12, textOffset:5}`,
  `RangeError: Invalid content…`, `StepMap [1,5,5]`, inverso recuperando `"Hello"`, `undoDepth=2`,
  seleção `1..6 → 2..7`).
- `node extra.js` → idêntica, **exceto** que o `exemplos/README.md` omitia a linha
  `Slice JSON: {"content":[{"type":"text","text":"adef"}]}`. Corrigi o transcript.
- `npm view` foi reexecutado: `prosemirror-model` criado em 2016-09-21, `latest 1.25.12`,
  `time.modified 2026-09-21`; `prosemirror-view 1.42.9`. As datas de maturidade conferem.
- `node_modules` de `/tmp/pm-run`: 9 entradas, `du -sh` 2,5 MB; `dist/index.js` medidos:
  124.855 / 86.426 / 36.285 / 250.622 / 18.498 B (model/transform/state/view/history).

O que **não** rodei: nada em navegador (view, CSS, IME, clipboard, a11y), benchmarks, e a
instanciação de `EditorView` em Node (só testei que `require`/`import` de `prosemirror-view`
funciona sem DOM — o que é um dado novo e favorável ao uso headless).

## 4. Achados e correções

### 4.1 Erros factuais corrigidos

| # | achado | correção |
|---|---|---|
| 1 | `baseKeymap` do PC descrito como "**nove** bindings". O objeto em `commands.ts:752-760` tem **oito** (`Enter`, `Mod-Enter`, `Backspace`, `Mod-Backspace`, `Shift-Backspace`, `Delete`, `Mod-Delete`, `Mod-a`). | README §3.2 e avaliacao corrigidos para oito, com a lista completa. |
| 2 | "conjunto de ~12 pacotes" no veredito vs. "~19 pacotes" na arquitetura §11. | Veredito agora diz 12 de núcleo / 19 com periféricos, citando `bin/pm.js:9-12`. |
| 3 | "Os **dois commits** git usados como contraste (commands, keymap, inputrules, example-setup)" — são **quatro** clones. | Reescrito, com versões conferidas nos `package.json` (`1.7.2`, `1.2.3`, `1.5.1`, `1.2.3`). |
| 4 | Tamanhos não minificados "88 KB (transform), 248 KB (view), 20 KB (history)" não batiam com `dist/index.js`. | Substituídos por bytes medidos: 86.426 / 250.622 / 18.498. |
| 5 | `exemplos/README.md` §3 não reproduzia a última linha de `extra.js` (`Slice JSON`). | Linha adicionada. |
| 6 | `prosemirror` no npm descrito como "meta-pacote de desenvolvimento". Na verdade o pacote `prosemirror` é a **0.11.1** (2015-09-03), `deprecated` — "split up into separate modules… no longer maintained". | Corrigido em `avaliacao.md` §3 e acrescentado ao README §5 como a armadilha de instalação mais provável. |
| 7 | "8.932 linhas somadas em `prosemirror-view/test/`" (avaliacao §4 e INDEX). 8.932 é o total de **todos** os pacotes clonados; a view tem **3.271** (3.216 `webtest-*.ts` + 55 `view.ts`). | Corrigido nos dois lugares; `exemplos/README.md` ganhou os dois números. |
| 8 | `commands.ts:786` citado como `selectParentNode`. `:786` é outro trecho do keymap; `selectParentNode` está em `:426` e `selectAll` em `:436`. | Linha corrigida na tabela de a11y. |
| 9 | `scrollIntoView` atribuído à view (`index.ts:461+`). É método de `Transaction` (`transaction.ts:206`); a view expõe `focus()` (`index.ts:338`) e a prop `handleScrollToSelection` (`:241`, `:678`). | Linha da tabela reescrita. |
| 10 | `content.ts:290` para o NFA; a função `nfa()` começa em `:289`. | Ajustado (off-by-one). |
| 11 | `Selection.replace` citado como `selection.ts:66-95`; `:66` é `content()`, `replace()` vem em `:72-88`. | Ajustado. |
| 12 | Citação de dependência `prosemirror-state-1.4.4/package.json:21` (linha de `maintainers`). | Corrigida para `:26-29`, onde `prosemirror-view` é declarada. |

### 4.2 Custo escondido — encontrado e documentado

O dossiê original afirmava "cada camada depende só das anteriores" e que `prosemirror-model` depende
"só de `orderedmap`/`rope-sequence`". O `package.json` mostra outra coisa:

- `prosemirror-model` depende **apenas** de `orderedmap` (não de `rope-sequence`);
- `prosemirror-state` declara `prosemirror-view` em `dependencies` (import **só de tipos** em
  `plugin.ts:1` e `transaction.ts:3`), então `npm i prosemirror-state` baixa a view e seus 250 KB;
- `prosemirror-history` declara `state`, `transform`, `view` e `rope-sequence`.

Por isso o `node_modules` do setup "headless" tem 9 pacotes: o headless não é gratuito em bytes. A
arquitetura §1 e o "custo medido" do README foram corrigidos; a distinção "acíclico em runtime, não
em instalação" agora está explícita. Isto é o achado mais relevante da revisão, porque afeta
diretamente quem só quer model+transform+state.

Também acrescentei o limite da mesclagem de `attributes` (`index.ts:523-531`): `class` é
concatenada à classe global, atributo já existente não é sobrescrito, `contenteditable`/`nodeName`
são ignorados — `role`/`aria-*` entram, mas a classe raiz e o `contenteditable` não são
controláveis por ali.

### 4.3 Citações que conferem (amostra grande, não exaustiva)

Checadas linha a linha e corretas: `node.ts:22/54/138/159/175/211/304/352/371`,
`replace.ts:8/13/34/101/113/121`, `mark.ts:10/24/109`, `fragment.ts:10/73/219`,
`resolvedpos.ts:12/38/130/173`, `schema.ts:90/110/153/382-453/558/596/604/610`, `content.ts:23/289/376/402`,
`map.ts:72/98/175/199/206/255`, `step.ts:16/26/31`, `replace_step.ts:7/34/38/42/93`,
`transform.ts:28/48/56/89/137`, `state.ts:21/44/90/137/171/185/199/217/234`,
`transaction.ts:42/71/128/187/206`, `selection.ts:66/72/118/143`, `plugin.ts:7/95/129`,
`view/index.ts:69/149/338/511/519-533/791`, `viewdesc.ts:143/680/773/786/878/910/1499`,
`domobserver.ts:39/54/83/377` (flush de 20 ms confirmado), `domchange.ts:82/362`,
`input.ts:19/39/108/527/572`, `clipboard.ts:6/35/44/74`, `decoration.ts:122/123/301`,
`history.ts:221/278/367/393/437/440`, `collab.ts:14/70/95/162`, `schema-basic.ts:10/109/138`,
`commands.ts:426/436/614/752-760`, `keymap.ts:76`, `example-setup/src/keymap.ts:59-65/86`.
`ref/prosemirror/demo/index.html:6-10` tem exatamente **5** `<link rel=stylesheet>`;
`demo/demo.ts` tem 16 linhas (8 de imports) e importa 6 pacotes; `demo/bench/` contém
`example.js`, `mutate.js`, `type.js`.

## 5. Auditoria da matriz de 85% e das notas

- A matriz tem **18** linhas, não 16 (título corrigido). Todas as 18 têm evidência declarada;
  **10** têm execução real (itens 1, 2, 3, 4, 5, 7, 10, 11, 12, 17) — o dossiê dizia 9 (corrigido).
  Portanto: cobertura de **evidência** ≥85%, cobertura de **execução** 56%, com os eixos de DOM
  declaradamente não executados. Isso é honesto; a meta de usabilidade não pode ser "provada" sem
  browser, e o texto diz isso.
- **modelo 5**: sustentado (imutabilidade com `return this`, `Slice` com lados abertos, autômato de
  conteúdo, `Mark` com rank/exclusão, `ResolvedPos`). Mantido.
- **a11y 1**: sustentado — `grep -rn "aria"` em `view/src` só encontra "variants"/"variant"; nenhum
  `role` é emitido; `attributes` permite suprir, e o dossiê agora explicita até onde.
- **estilo 2**: mantido, com ressalva registrada. O descritor de 2 na rubrica é "traz tema pesado",
  que **não** é o caso (o CSS é funcional, 54 linhas, sem tema). A nota 2 se justifica pela
  dependência **obrigatória** de stylesheet + seletores globais `.ProseMirror*`, não pelo peso
  visual; lida só pelo peso, 3 seria defensável. Deixei a ressalva no `avaliacao.md` em vez de
  mudar a nota, porque o efeito de integração (precisa de CSS global) é real.
- **performance 4** marcada como estrutural, sem medição — condizente com o texto.
- A distinção "biblioteca de peças, não editor" está preservada e reforçada: veredito, §3 (custo
  real), §5 (UX), e a comparação de instalação com a `prosemirror` 0.11.1.

## 6. O que continua não verificado

1. Comportamento em navegador: renderização, seleção DOM, IME/composição, clipboard, foco e leitor
   de tela. Tudo o que o dossiê afirma nesses eixos é leitura de código/testes, não execução.
2. Benchmarks (`ref/prosemirror/demo/bench/`) — nenhum número de throughput.
3. `minimal-editor.html` — não executado (exige navegador e rede para `esm.sh`/`jsdelivr`); a
   descrição do que se espera ver é explicitamente "inferida", o que é correto.
4. `prosemirror-tables` — não obtido. Tentei reexecutar o clone nesta revisão e o espelho pediu
   credenciais; não confirmei a causa original, então a nota em `avaliacao.md` §4.3 foi suavizada
   para "não obtido", em vez de atribuir o motivo ao repositório.
5. Comandos/keymap/inputrules/example-setup vêm de clone git (HEAD), não de tarball: as versões dos
   `package.json` batem com as citadas, mas não há hash de tarball para eles — está declarado no
   `INDEX.md`.
6. `prosemirror-view` importável em Node foi testado; **instanciar** `EditorView` sem `document` não
   foi executado (o texto apenas diz que exige DOM, o que é evidente do código).

## 7. Confiança final

**88%.**

Sustentação: fonte correta e verificada por hash, cópias verbatim confirmadas, execuções headless
reproduzidas com saída idêntica, mais de 80 citações de linha conferidas pontualmente, metadados npm
reconsultados, e os números de tamanho/contagem refeitos. O que impede 90+ é exatamente o que o
dossiê já declara: nenhuma execução em navegador e nenhuma medição de performance — os eixos de
view/CSS/a11y/IME não podem subir de "verificado no código" para "observado" neste ambiente.
