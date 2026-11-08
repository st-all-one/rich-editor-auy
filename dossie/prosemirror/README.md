# ProseMirror — dossiê de referência

> **Veredito curto:** ProseMirror é a **referência canônica** de arquitetura para editores ricos
> baseados em `contenteditable`, mas **não é um editor**: é um conjunto de 12 pacotes de núcleo
> (19 contando os periféricos, `ref/prosemirror/bin/pm.js:9-12`) que fornecem modelo de documento,
> transformações, estado e camada DOM. Para o `rich-editor-auy`, ele é a melhor
> fonte para **copiar o modelo de dados e o sistema de posições/mapeamento** e o pior exemplo a copiar
> em **acoplamento a DOM, CSS obrigatório e acessibilidade delegada ao integrador**.

Este dossiê cobre a versão publicada de cada pacote (ver `codigo/INDEX.md`), com evidência em
`caminho:linha` e execução real registrada em `exemplos/`.

## 1. O que está em `ref/prosemirror/` (e o que não está)

`ref/prosemirror/` é o **meta-repositório histórico**, não o código. Evidências:

- `ref/prosemirror/README.md:1` — "This repository has moved to code.haverbeke.berlin/…".
- `ref/prosemirror/README.md:11-14` — "This repository just serves as a central issue tracker, and
  holds a script to help easily check out all the core modules".
- `ref/prosemirror/bin/pm.js:9-13` — a lista de módulos (`model`, `transform`, `state`, `view`,
  `keymap`, `inputrules`, `history`, `collab`, `commands`, `gapcursor`, `schema-basic`,
  `schema-list` + `menu`, `example-setup`, `markdown`, `dropcursor`, `test-builder`, `changeset`,
  `search`).
- `ref/prosemirror/bin/pm.js:116-131` (`install`) — clona cada módulo como **repositório irmão**
  dentro do diretório; por isso não existe `src/` versionado no stub.
- `git ls-tree HEAD` em `ref/prosemirror/` lista apenas `.github/`, `assets/`, `bin/`, `demo/`,
  `package.json`, `tsconfig.json`, `LICENSE`, `README.md`, `CONTRIBUTING.md`, `.gitignore`.

Portanto a análise de código usa os **tarballs oficiais do npm** extraídos em `/tmp/pm-upstream/`
(versões e sha256 na tabela de `codigo/INDEX.md`): model **1.25.12**, transform **1.12.5**,
state **1.4.4**, view **1.42.9**, schema-basic **1.2.5**, history **1.5.1**, collab **1.3.1**.
Os clones git usados só como contraste (`prosemirror-commands` 1.7.2, `prosemirror-keymap` 1.2.3,
`prosemirror-inputrules` 1.5.1, `prosemirror-example-setup` 1.2.3, todos em `/tmp/pm-src/`) estão
identificados no texto; nenhum trecho deles foi copiado.

Licença: **MIT** (`codigo/LICENSE`, copiado de `prosemirror-model-1.25.12/LICENSE`). Data de criação
no npm de `prosemirror-model`: **2016-09-21** (`npm view prosemirror-model time.created`), com
release 1.25.12 em **2026-09-21** (`prosemirror-model-1.25.12/CHANGELOG.md:1`) — dez anos de
manutenção contínua, ainda em série `1.x`.

## 2. Por que é a referência canônica

ProseMirror separa quatro responsabilidades que quase todo editor rico mistura, e essa separação é
o que o torna "o" modelo mental do domínio:

| camada | pacote | o que garante | evidência |
|---|---|---|---|
| documento imutável | `prosemirror-model` | árvore `Node`/`Fragment`, marcas, `Slice`, validação por schema | `node.ts:22`, `fragment.ts:10`, `mark.ts:10`, `replace.ts:13` |
| transformação por passos | `prosemirror-transform` | `Step` atômico, invertível e mapeável; `Transform` encadeia | `step.ts:16`, `transform.ts:28`, `map.ts:72` |
| estado + plugins | `prosemirror-state` | `EditorState` imutável, `Transaction`, seleção, campos de plugin | `state.ts:90`, `transaction.ts:42`, `plugin.ts:7` |
| DOM/entrada | `prosemirror-view` | renderização incremental, seleção, IME, colar/copiar, eventos | `index.ts:30`, `viewdesc.ts:680`, `input.ts:527` |

A consequência arquitetural é forte: **a fonte da verdade é a árvore, não o DOM**. Toda edição vira
`Step` aplicado a um documento novo (`transform.ts:48-60`), e o DOM é reconciliado depois
(`index.ts:149`, `viewdesc.ts:773`). É exatamente o inverso de um `contenteditable` tradicional, onde
o DOM é a verdade e o modelo é reconstruído.

O nome do projeto resume a intenção: "Structured **WYSIWYM** editor" (`ref/prosemirror/package.json:4`)
— *what you see is what you mean*: o schema e as marcas são semânticos, não estilos. `toDOM` decide
a aparência inicial, mas não há no core nenhuma noção de "negrito visual"; há a marca `strong`
(`prosemirror-schema-basic-1.2.5/src/schema-basic.ts:138`).

## 3. O que o usuário precisa implementar (o custo real)

ProseMirror **não entrega um editor utilizável**; ele entrega as peças. Para a primeira edição
formatada, o integrador precisa escrever/prover:

1. **Schema** — `doc` com `content: "block+"` (`schema-basic.ts:10-11`), tipos de bloco inline
   (`paragraph` com `content: "inline*"`) e marcas. As expressões de conteúdo são validadas por um
   autômato (`content.ts:10`, `content.ts:23`, `content.ts:376`) e conteúdo inválido lança
   `RangeError` — verificado em execução: `Invalid content for node paragraph: <paragraph>`.
2. **Comandos e keymap** — o `baseKeymap` do core tem **oito** bindings no PC
   (`prosemirror-commands@1.7.2/src/commands.ts:752-760`): Enter, Mod-Enter, Backspace,
   Mod-Backspace, Shift-Backspace, Delete, Mod-Delete, Mod-a. **Não há Mod-b, Mod-i, Mod-z**.
   Negrito/itálico/desfazer exigem
   `toggleMark` (`commands.ts:614`), `undo/redo` (`prosemirror-history-1.5.1/src/history.ts:437,440`)
   e um `keymap` (`prosemirror-keymap@1.2.3/src/keymap.ts:76`). O pacote `example-setup` existe só
   para montar isso (`prosemirror-example-setup/src/keymap.ts:60-65`).
3. **View** — `new EditorView(place, {state})` (`view/src/index.ts:69`) e um laço de dispatch:
   sem `dispatchTransaction`, a view aplica a transação sozinha (`index.ts:511-515`); com ele, o
   integrador controla estado (necessário para colaboração/undo externo).
4. **CSS** — o pacote de view **exige** um stylesheet: "Make sure you load
   `style/prosemirror.css` as a stylesheet when using this module"
   (`prosemirror-view-1.42.9/src/README.md:5-6`), exportado em `package.json:17`
   (`"style": "style/prosemirror.css"`). O arquivo (`codigo/prosemirror.css`) fixa
   `position: relative`, `white-space: pre-wrap/break-spaces`, desliga ligaduras e desenha
   `.ProseMirror-selectednode`. Sem ele, quebras de linha e seleção de nós ficam erradas.
   O demo ainda carrega **cinco** folhas de estilo (`ref/prosemirror/demo/index.html:6-10`).
5. **Acessibilidade** — o core não emite `role`/`aria-*`: `grep -rn "aria"` em
   `prosemirror-view-1.42.9/src/` encontra apenas as palavras "variants" e "cheap variant"
   (`decoration.ts:122`, `viewdesc.ts:1499`). O que a view define é `class="ProseMirror"`,
   `contenteditable` e `translate="no"` (`index.ts:519-533`). `role="textbox"`,
   `aria-multiline`, labels e contraste são responsabilidade do integrador.
6. **Extras de produto** — menu, drop cursor, gap cursor, tabelas, markdown: pacotes separados
   (`bin/pm.js:12`), cada um com CSS e semântica próprios.

Custo medido: um setup headless mínimo (model+transform+state+history+schema-basic+commands)
instalou **9 pacotes / 2,5 MB** em `node_modules` — e isso inclui `prosemirror-view` e
`rope-sequence` como transitivas, porque `prosemirror-state` declara `prosemirror-view` em
`dependencies` (`prosemirror-state-1.4.4/package.json`, import só de tipos em
`plugin.ts:1`/`transaction.ts:3`) e `prosemirror-history` declara `rope-sequence`; o "headless"
não é gratuito em bytes. Os `dist/index.js` não minificados somam 124.855 B (model), 86.426 B
(transform), 36.285 B (state), 250.622 B (view), 18.498 B (history) — em gzip,
**29,9 KB + 20,4 KB + 9,4 KB + 61,2 KB** para model/transform/state/view. Não é um editor
"leve", é um toolkit com superfície grande.

## 4. Como o modelo lida com posições e mapeamento

Esta é a parte mais valiosa para nós.

**Posições são inteiros em uma travessia de tokens.** Cada nó não-folha contribui com 2 (abertura e
fechamento); texto contribui com o número de caracteres; folha não-texto contribui com 1:
`node.ts:54` — `get nodeSize(): number { return this.isLeaf ? 1 : 2 + this.content.size }`.
Verificado em execução com o schema básico:

```
doc.nodeSize = 15 | paragraph.nodeSize = 13 | text.nodeSize = 11   # "Hello world"
```

Ou seja, o cursor entre `<p>` e o texto é a posição 1; `</p>` fica em 12; o fim do doc em 15 —
"tokens" ocupam espaço como no DOM.

**`ResolvedPos`** transforma um inteiro em contexto: `depth`, `parent`, `index`, `start/end`,
`textOffset`, `sharedDepth` (`resolvedpos.ts:12,63,95,173`). Em execução, `doc.resolve(6)` devolveu
`{depth:1, parent:"paragraph", parentOffset:5, start:1, end:12, textOffset:5, nodeAfter:" world"}`.
Isso substitui a pergunta "em que nó estou?" por "que caminho de índices me levou até aqui?".

**`StepMap` é a resposta de mapeamento.** Um passo guarda ranges `[start, oldSize, newSize]`
(`map.ts:72-91`); um `ReplaceStep` de "Hello"→"Howdy" produz `[1,5,5]` (verificado:
`StepMap ranges = "[1,5,5]"`). `map(pos, assoc)` decide o lado quando a posição cai na borda
(`assoc = -1|1`, `map.ts:98-121`), e `mapResult` informa `deleted`, `deletedBefore`,
`deletedAfter`, `deletedAcross` (`map.ts:40-70`). Execução real de `delete(2,5)`:

```
mapResult(2) -> {pos:2, deleted:true,  deletedBefore:false, deletedAfter:true,  deletedAcross:false}
mapResult(3) -> {pos:2, deleted:true,  deletedBefore:true,  deletedAfter:true,  deletedAcross:true}
mapResult(5) -> {pos:2, deleted:false, deletedBefore:true,  deletedAfter:false, deletedAcross:false}
```

**`Mapping` é o pipeline.** `Transform` acumula um `Mapping` com um `StepMap` por passo
(`transform.ts:89-95`), e `Mapping` suporta `mirrors` para desfazer/rebasear sem perder a
"identidade" das posições (`map.ts:175-250`, `appendMap` em `map.ts:206`). É essa peça que permite
undo e colaboração: `rebaseSteps` inverte, aplica passos remotos e reaplica localmente
(`prosemirror-collab-1.3.1/src/collab.ts:14-32`).

**Invertibilidade é contrato do passo.** `Step.invert(doc)` devolve o passo que desfaz
(`step.ts:31-33`), e `ReplaceStep.invert` reusa o slice antigo
(`replace_step.ts:38-40`). Verificado: o inverso do replace foi
`{"stepType":"replace","from":1,"to":6,"slice":{"content":[{"type":"text","text":"Hello"}]}}`.

**Seleção também mapeia.** `Transaction.selection` é a seleção anterior mapeada pelos passos
(`transaction.ts:71-79`); executado: seleção `1..6` virou `2..7` após inserir um caractere em 1.

## 5. O que o usuário vê na prática (UX real)

- **Instalação:** vários pacotes npm `prosemirror-*`; não existe `prosemirror` "tudo-em-um"
  utilizável. Pior: `npm install prosemirror` **funciona** e instala a **0.11.1** (criada em
  2015-09-03, marcada `deprecated`: "This library has been split up into separate modules… no longer
  maintained"), com a API antiga — o erro de instalação mais provável de um iniciante. O meta-repo
  tem `bin/pm install` apenas para *desenvolver* o ProseMirror.
- **Primeira edição:** só funciona se schema + view + keymap estiverem ligados. O demo oficial faz
  isso em 16 linhas de TS, das quais 6 só imports (`ref/prosemirror/demo/demo.ts:1-16`), mas importa
  seis pacotes.
  Digitar texto funciona porque a view lê mutações do DOM (`domchange.ts:82`) e as converte em
  transações; formatar, não: depende de comando.
- **Formatação:** `toggleMark`/`setBlockType` produzem passos; o botão/menu é do integrador
  (`prosemirror-menu`, fora do core).
- **Seleção e teclado:** a view reconcilia a seleção do DOM com a do estado
  (`view/src/selection.ts`), com correções específicas por navegador (Chrome/IE/Safari/Firefox
  aparecem nominalmente em `index.ts:189-215` e `domobserver.ts:54-70`).
- **Undo/redo:** plugin opcional; agrupa mudanças adjacentes dentro de 500 ms
  (`history.ts:377-394`), com `depth` 100. Verificado: duas digitações adjacentes → `undoDepth = 1`
  e um único undo restaurou o texto.
- **Clipboard:** HTML + texto, com marcador `data-pm-slice` para preservar lados abertos do slice
  (`clipboard.ts:6-42`, `clipboard.ts:35`); sem esse marcador, o HTML até parseia, mas os lados
  abertos (colar metade de uma lista, por exemplo) se perdem — `parseFromClipboard` lê o atributo em
  `clipboard.ts:74-75`.
- **IME/composição:** tratada com estado próprio (`input.ts:19`, `527`, `572`), inclusive
  "badSafariComposition" e Android. Não é trivial e é onde editores caseiros quebram.
- **Persistência:** `doc.toJSON()` (executado) e `EditorState.toJSON/fromJSON`
  (`state.ts:217,234`), com campos de plugin serializáveis. O formato é o mesmo do schema — não há
  HTML canônico.
- **Erros:** `RangeError("Invalid content for node …")` na construção; `ReplaceError`
  (`replace.ts:8`) quando um slice não encaixa; falhas de passo viram `StepResult.fail`
  (`step.ts:71-91`) em vez de exceção.
- **Aparência:** o core não opina sobre tipografia; a view só garante o mínimo funcional do
  `prosemirror.css`. Negrito é `<strong>` porque o schema básico manda
  (`schema-basic.ts:138`), não porque o motor "saiba" o que é negrito.

## 6. Prós e contras para nós

**Copiar/adaptar (alto valor):**

- A divisão **model / transform / state / view** e a regra "DOM nunca é a verdade".
- `Slice` com `openStart`/`openEnd` (`replace.ts:13-41`) e o `Fitter` que ajusta inserções
  (`prosemirror-transform-1.12.5/src/replace.ts:54-120`): é o que faz colar/entrar em listas
  funcionar sem casos especiais.
- `StepMap` + `Mapping` + `assoc` + `deleted*` + `mirrors`: base limpa para undo, colaboração e
  âncoras/comentários (nosso caso tem referências entre publicações — posições que precisam
  sobreviver a edições).
- Schema declarativo com expressões de conteúdo compiladas para autômato (`content.ts`): validação
  estrutural de graça, mensagens de erro boas.
- `EditorState` com **campos de plugin** (`plugin.ts:95-127`) e `appendTransaction`
  (`state.ts:137-170`): forma comprovada de estender sem tocar no core.
- Serialização JSON de documento e de estado (`state.ts:217`), com `parseDOM`/`toDOM` por tipo.

**Evitar (custo que não queremos):**

- `contenteditable` como superfície de entrada: obriga `DOMObserver`, `domchange`, correções por
  navegador, hack de composição, dois modelos de seleção. Nosso editor é headless; o front-end
  decide a superfície.
- **CSS obrigatório** e classes globais `.ProseMirror*` (`prosemirror.css`, `index.ts:519`):
  contraria nossa proposta de zero CSS e de não interferir no CSS do host.
- **A11y fora do core**: nenhum `role`/`aria` emitido; o integrador precisa saber que
  `contenteditable` não é suficiente.
- Comandos e keymap mínimos: cada produto reescreve o "editor padrão". Um core nosso deve nascer
  com um conjunto semântico padrão (negrito/itálico/link/desfazer), mesmo que o front-end o
  reestilize.
- Acoplamento da view ao navegador (detecção de Safari/Chrome/Android em tempo de execução) e
  dependência de `MutationObserver`/`Selection` implicam que a camada de DOM é grande e difícil de
  testar sem browser (a suíte `webtest-*.ts` só roda em navegador real).

## 7. Contraste com `rich-editor-auy`

| dimensão | ProseMirror | nossa proposta |
|---|---|---|
| núcleo | Rust headless, sem DOM | Rust headless, sem DOM — **igual**, e é a razão de estudá-lo |
| verdade | árvore + passos | árvore + operações; posições precisam de mapa para referências entre publicações |
| CSS | stylesheet obrigatório + classes globais | **zero CSS**; o host estiliza |
| entrada | `contenteditable` no core | front-end TS/PHP escolhe a superfície; IME/selection não vazam para o core |
| a11y | delegada | precisa ser contrato do core/DSL (roles, labels, foco, leitura) |
| extensão | schema + plugin com campos + `appendTransaction` | schema declarativo + plugins; copiar o padrão de campos e hooks |
| serialização | JSON de doc/estado; HTML via `toDOM` | JSON canônico + renderer separado (sem `toDOM` no modelo) |
| posições | inteiros com tokens `+2`; `ResolvedPos` | podemos copiar o esquema de índices, mas com API explícita de âncoras estáveis |

O que **não** deve ser copiado literalmente: o `+2` por nó é elegante para cursor mas torna
"posição de conteúdo" e "posição de token" fáceis de confundir (o próprio guia do ProseMirror
gasta páginas nisso); para referências entre publicações, queremos identificadores estáveis +
intervalos, não offsets globais frágeis.

## 8. Como navegar este dossiê

- `arquitetura.md` — modelo, transformações, estado, view, schema, IME, clipboard, a11y,
  performance e extensibilidade, com evidência linha a linha.
- `avaliacao.md` — rubrica 0–5 por dimensão + matriz de usabilidade (>=85%) com o que foi
  verificado no código, em teste, em execução e o que ficou não verificado.
- `codigo/` — seis arquivos reais (model, transform, state, view) + `INDEX.md` com origem,
  versão, sha256 e licença.
- `exemplos/` — schema/editor mínimo, script headless **executado** e comandos reproduzíveis.
