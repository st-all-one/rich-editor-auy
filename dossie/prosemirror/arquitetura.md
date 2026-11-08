# Arquitetura do ProseMirror

Fontes: tarballs npm oficiais (versões em `codigo/INDEX.md`), extraídos em `/tmp/pm-upstream/`.
Cópias locais em `codigo/` são verbatim, então os números de linha valem para os dois lugares.
Execuções citadas: `exemplos/demo.js` (Node 22, sem DOM) — saída em `exemplos/README.md`.

## 1. Visão em camadas

```
   schema  ── define tipos, attrs, content expressions, toDOM/parseDOM
      │
   model   ── Node / Fragment / Mark / Slice / ResolvedPos   (imutável)
      │
 transform ── Step (apply/invert/map) + Transform + Mapping  (histórico de passos)
      │
   state   ── EditorState (doc, selection, storedMarks, campos de plugin) + Transaction
      │
    view   ── EditorView: DOM ↔ estado, seleção, IME, clipboard, decorações
```

A intenção é que cada camada dependa só das anteriores (`prosemirror-view-1.42.9/package.json`
depende de model, state e transform; `prosemirror-transform` depende de model;
`prosemirror-model` só de `orderedmap`). Mas o grafo **não é estritamente acíclico no npm**:
`prosemirror-state-1.4.4/package.json:26-29` declara `prosemirror-view` em `dependencies`, porque
`plugin.ts:1` e `transaction.ts:3` importam `EditorView`/`EditorProps` **só como tipo**
(`import {type …}`). O efeito prático é que `npm install prosemirror-state` baixa a view (e seu
dist de 250 KB) mesmo para uso sem DOM; e `prosemirror-history` traz `rope-sequence`. A separação
de camadas vale em tempo de execução — model+transform+state rodam em Node sem DOM, como este
dossiê executou — não em tempo de instalação.

## 2. Modelo de dados

### 2.1 `Node` e `Fragment`

`Node` é imutável e persistente: `copy`/`mark`/`cut` retornam `this` quando nada muda
(`prosemirror-model-1.25.12/src/node.ts:138-157`), o que torna barato compartilhar estrutura entre
documentos. Um nó guarda `type`, `attrs`, `content` (um `Fragment`) e `marks`
(`node.ts:22-36`). Texto é `TextNode` (`node.ts:352`), com `nodeSize` = tamanho da string
(`node.ts:371`).

`Fragment` é uma lista de filhos com tamanho total (`fragment.ts:10-27`), com `append` que funde
textos adjacentes de mesma marca (`fragment.ts:73-85`), `cut` que recorta preservando tokens
(`fragment.ts:86-104`), e construtores `fromArray`/`from`/`fromJSON` (`fragment.ts:219-260`).

Consequência: **não há nós-pai**. Toda navegação é descendente, e "onde estou" exige ou um caminho
(`ResolvedPos`) ou recomputação.

### 2.2 Posições

`nodeSize` de um nó não-folha é `2 + content.size` (`node.ts:54`): um token de abertura e um de
fechamento. Logo, uma posição inteira pode cair:

- **antes/depois de um nó** (offset de token);
- **dentro de um textblock** (conteúdo inline "achatado", `resolvedpos.ts:38-40` — texto e inline nodes
  são irmãos em um único `Fragment`).

`ResolvedPos` resolve o inteiro em contexto e é memoizado por documento (`node.ts:211` chama
`ResolvedPos.resolveCached`). Ele expõe `depth`, `parent`, `index`, `start`/`end`, `textOffset`,
`nodeBefore`/`nodeAfter`, `sharedDepth` e `marks()` (`resolvedpos.ts:12-200`). `marks()` já aplica
a regra de `inclusive`: marcas não-inclusivas não "vazam" para o começo do próximo nó
(`resolvedpos.ts:130-152`). Isso é uma decisão de UX embutida no modelo: negrito não continua
depois de um espaço quando a marca é `inclusive: false`.

Verificado em execução:

```
resolve(6) = {pos:6, depth:1, parent:"paragraph", parentOffset:5, start:1, end:12,
              textOffset:5, index:0, nodeAfter:" world"}
```

`sharedDepth` (`resolvedpos.ts:173`) é a base de comparação de duas posições — usado por
`Node.slice`, `findDiff` da view e pelo `Fitter`.

### 2.3 `Slice`: corte com lados abertos

Um `Slice` é `Fragment + openStart + openEnd` (`replace.ts:13-41`). `openStart` não é "quantos
níveis cortar": é **quantos nós da borda esquerda foram atravessados e devem permanecer abertos**
para o conteúdo se juntar ao destino. `size` desconta os lados abertos (`replace.ts:34-36`).

É a peça que faz "selecionar o meio de dois parágrafos e apagar" virar uma substituição estrutural
bem definida, em vez de um caso especial. `Node.slice(from, to, includeParents)`
(`node.ts:159-166`) calcula `openStart/openEnd` a partir do `sharedDepth`, e `Node.replace`
(`node.ts:175-177`) chama o algoritmo de `replace.ts:113`, que pode lançar `ReplaceError`
(`replace.ts:8`) quando os lados não encaixam.

O algoritmo (`replace.ts:121-216`) trata três casos — substituição em uma posição "três vias"
(conteúdo antes, slice, conteúdo depois), "duas vias" e remoção — sempre reconstruindo a árvore.
`insertInto` valida `parent.canReplace(...)` antes de aceitar (`replace.ts:101-111`).

### 2.4 `Mark`

`Mark` é tipo + attrs (`mark.ts:10-18`). O conjunto é uma **lista ordenada por `rank`**, com
exclusões: `MarkType.excludes` é derivado de `spec.excludes` no `Schema`
(`schema.ts:610-612`), e `addToSet` remove marcas mutuamente exclusivas e insere pela ordem de rank
(`mark.ts:22-47`). `Mark.none` é a lista vazia compartilhada (`mark.ts:109`).

Isso evita a "sopa de spans": duas marcas incompatíveis não coexistem, e a serialização
(`to_dom.ts:46-75`) pode emitir tags aninhadas de forma estável.

## 3. Schema e expressões de conteúdo

`Schema` compila os specs em `NodeType`/`MarkType` (`schema.ts:558-620`). Cada `NodeType` recebe:

- `content: string` → `contentMatch`, um autômato (`schema.ts:596-598`);
- `marks` → conjunto permitido (`schema.ts:604-607`);
- `inlineContent`, `isLeaf` derivados de `contentMatch` (`schema.ts:90-110`).

A compilação passa por `ContentMatch.parse` (`content.ts:23`), que tokeniza a expressão, constrói
NFA (`content.ts:289`), converte em DFA (`content.ts:376`) e checa becos sem saída
(`content.ts:402`). O DFA é o que permite responder em O(1)-ish:

- `matchType`/`matchFragment` (`content.ts:31-50`) — "este filho pode entrar aqui?";
- `fillBefore` (`content.ts:79`) — "que nós preciso inserir para caber?";
- `findWrapping` (`content.ts:104`) — "em que nós preciso envolver para caber?".

Essas três operações são a razão pela qual `setBlockType`, `wrapInList`, colar dentro de listas e
"Enter no meio de um heading" funcionam sem código ad-hoc.

`NodeSpec`/`MarkSpec` carregam semântica que a view respeita:

| campo | efeito | evidência |
|---|---|---|
| `defining`/`definingAsContext`/`definingForContent` | quais pais sobrevivem a um replace | `schema.ts:415-427` |
| `isolating` | bordas que backspace/lift não atravessam (ex.: célula) | `schema.ts:431-432` |
| `code` | whitespace `pre`, comandos mudam de comportamento | `schema.ts:398-400` |
| `whitespace` | parsing colapsa ou preserva espaços | `schema.ts:409-413` |
| `toDOM`/`parseDOM` | ponte com HTML | `schema.ts:445,453,519,524` |
| `atom`, `selectable`, `draggable` | comportamento de seleção/arraste | `schema.ts:382-396` |

Validação em execução: `schema.node("paragraph", null, [schema.node("paragraph")])` lança
`RangeError: Invalid content for node paragraph: <paragraph>`; e
`schema.topNodeType.createAndFill()` gera `{"type":"doc","content":[{"type":"paragraph"}]}`.

## 4. Transformações

### 4.1 `Step`

`Step` é abstrato e exige `apply`, `invert`, `map`, `toJSON` (`step.ts:16-69`). `getMap()` tem
default `StepMap.empty` (`step.ts:26`) — passos de marca não mudam tamanho. `jsonID` registra a
classe para serialização (`step.ts:61-67`), e `Step.fromJSON` procura no registro
(`step.ts:50-58`).

`StepResult` separa "falhou" de "exceção" (`step.ts:71-91`), o que permite tentar passos
especulativamente (`Transform.maybeStep`, `transform.ts:56-60`).

`ReplaceStep` (`replace_step.ts:7-91`) é o passo universal: guarda `from`, `to`, `slice`,
`structure`. `getMap` devolve `new StepMap([from, to-from, slice.size])` (`replace_step.ts:34-36`);
`invert` usa `doc.slice(from, to)` (`replace_step.ts:38-40`); `map` mapeia `to` com assoc `-1` e
`from` com `+1` e devolve `null` se ambos foram apagados (`replace_step.ts:42-47`); `merge` funde
substituições adjacentes (`replace_step.ts:49-63`). Há ainda `ReplaceAroundStep` para "mover
conteúdo para dentro/fora" (`replace_step.ts:93+`), usado por listas.

### 4.2 `Transform`

`Transform` guarda `steps`, `docs` (documento antes de cada passo) e um `Mapping`
(`transform.ts:28-46`). `step()` aplica e lança em falha; `maybeStep()` ignora falha
(`transform.ts:48-60`). `addStep` empilha doc/step e faz `mapping.appendMap(step.getMap())`
(`transform.ts:89-95`) — ou seja, **o mapa é derivado dos passos, não armazenado à parte**.

`replaceRange` (`transform.ts:137` → `replace.ts:340`) é o "replace amigável a WYSIWYG": usa
`from/to` como dicas e cresce/encolhe a região conforme `defining`/`openStart` para "encaixar" o
slice (`replace.ts:340-410`). É o que se usa para colar; `replace` cru é para precisão cirúrgica.

### 4.3 `StepMap` e `Mapping` — o coração do mapeamento

`StepMap` codifica mudanças como triplas `[start, oldSize, newSize]` (`map.ts:72-91`). O `_map`
(`map.ts:98-134`) percorre as triplas acumulando `diff` e escolhe o lado por `assoc`:

- `assoc > 0` (padrão): posição em uma borda fica **depois** do conteúdo inserido;
- `assoc < 0`: fica **antes**.

`MapResult` carrega `recover` (índice+offset codificados em um número, `map.ts:24-34`) e flags
`deletedBefore/After/Across/deleted` (`map.ts:40-70`). `recover` permite reconstruir a posição
original mesmo depois de várias operações — é o mecanismo que mantém widgets/decorações e
bookmarks de seleção no lugar.

`Mapping` é uma lista de `StepMap` com `from` (início da janela) e `mirror` (`map.ts:175-250`).
`mirror` liga um passo ao passo inverso, o que evita que uma posição "atravesse" duas vezes a mesma
edição ao rebasear — a base de undo/colaboração (`map.ts:199-250`). `slice(from,to)`
(`map.ts:199`) cria um `Mapping` parcial usado por `Transaction.selection`
(`transaction.ts:71-79`) e por `Selection.replace` (`selection.ts:72-88`).

### 4.4 `Fitter` — por que inserir "só funciona"

`replaceStep` (`replace.ts:12-19`) tenta o caminho trivial e, se falhar, usa `Fitter`. O `Fitter`
(`replace.ts:54-120`) mantém uma "fronteira" de `{type, match}` desde `$from` até `$to`, coloca o
conteúdo não colocado (`unplaced`) pedaço a pedaço e, quando não cabe, **abre mais** o slice ou
**descarta** nós até caber (`fit()`, `replace.ts:77-120`). No fim, normaliza os lados abertos e
emite `ReplaceStep` ou `ReplaceAroundStep`.

Esse é o componente mais difícil de reimplementar bem e o que mais impacta UX (colar dentro de
listas, Enter no fim de bloco, mover itens). Para o `rich-editor-auy`, é referência obrigatória —
mesmo que a implementação Rust seja outra.

## 5. Estado

`EditorState` é imutável e guarda apenas `config` (`state.ts:90-116`); os valores vivem em campos:

- base: `doc`, `selection`, `storedMarks`, `scrollToSelection` (`state.ts:21-43`);
- de plugins: um `FieldDesc` por `plugin.spec.state`, com `init/apply` (`state.ts:44-62`).

`create` inicializa os campos na ordem (`state.ts:185-195`); `applyInner` rejeita transação cujo
`tr.before` não bate com o doc atual e reconstrói os campos (`state.ts:171-180`).
`applyTransaction` (`state.ts:137-170`) implementa o laço de `filterTransaction` +
`appendTransaction`: um plugin pode vetar ou **acrescentar** transações, e as transações
acrescentadas são passadas de novo aos plugins que ainda não as viram. É assim que o
`inputrules`/`history` reagem sem acoplamento.

`reconfigure` troca plugins mantendo campos homônimos (`state.ts:199-215`); `toJSON/fromJSON`
serializam doc, seleção, storedMarks e campos que implementem os hooks (`state.ts:217-260`).

`Transaction` estende `Transform` (`transaction.ts:42-60`) e adiciona:

- seleção mapeada + `setSelection` (`transaction.ts:71-90`);
- `storedMarks` e `ensureMarks` (`transaction.ts:95-127`);
- `addStep` que invalida storedMarks (`transaction.ts:128-132`);
- `replaceSelection`, `replaceSelectionWith`, `deleteSelection` (`transaction.ts:141-152`);
- `setMeta/getMeta` com chave `string | Plugin | PluginKey` (`transaction.ts:187-200`) — o canal de
  metadados usado por `"addToHistory"`, `"composition"`, `"pointer"`, `"uiEvent"`.

## 6. Seleção

`Selection` é abstrata e trabalha com `ResolvedPos` (`selection.ts:9-40`), com `ranges` para
seleções múltiplas (usadas em tabelas). Tipos: `TextSelection` (cursor/intervalo em texto),
`NodeSelection` (nó atômico), `AllSelection` (tudo), e `GapCursor` fica em pacote separado.

Operações-chave: `map(doc, mapping)` (cada subclasse), `content()` que devolve um `Slice` com
`includeParents=true` (`selection.ts:66-68`), `replace(tr, slice)` que ajusta a seleção para depois
do conteúdo inserido (`selection.ts:66-95`), e `getBookmark()`/`Bookmark.resolve()` para guardar a
seleção fora do documento (usado pelo history, `history.ts:221-245`).

`Selection.atStart` e `findFrom` garantem o invariante "sempre há uma seleção válida"
(`selection.ts:118-190`), inclusive pulando nós sem conteúdo inline.

## 7. View

### 7.1 Renderização incremental

`EditorView` mantém `docView`, uma árvore paralela de `ViewDesc` (`viewdesc.ts:143`) com um
`NodeViewDesc` por nó (`viewdesc.ts:680`). Em `updateState` (`index.ts:149-260`):

1. calcula decorações internas/externas;
2. pergunta `docView.matchesNode(state.doc, …)` (`viewdesc.ts:773`) — compara `node.eq`, decorações
   e `dirty`;
3. se não bate, tenta `docView.update(...)` (`viewdesc.ts:878`), que **reusa** subárvores por
   identidade/posição e só recria o que mudou (`updateChildren`, `viewdesc.ts:786-870`);
4. se `update` retorna false, destrói e recria a subárvore.

`renderDescs` (`viewdesc.ts:1060`) e `computeOuterDeco` (`viewdesc.ts:1090`) aplicam decorações sem
tocar no documento. O resultado é O(mudança) em vez de O(documento) na maioria das edições — o que
importa em documentos grandes.

### 7.2 Entrada e reconciliação

A view escuta eventos DOM e mutações:

- `DOMObserver` (`domobserver.ts:39-130`) acumula `MutationRecord`s e faz `flush`/`flushSoon`
  (20 ms). Há desvios explícitos para IE11 e Safari (`domobserver.ts:54-70`);
- `readDOMChange` (`domchange.ts:82-200`) lê a região alterada, parseia com o `DOMParser` do schema,
  faz diff contra o documento atual (`findDiff`, `domchange.ts:362`) e emite uma `Transaction`;
- `dispatchKey` (`input.ts:108-120`) dá prioridade aos `handleKeyDown` dos props/plugins — é assim
  que `keymap` funciona;
- composição (`input.ts:527-620`) mantém `composing`, `compositionNode`, `compositionPendingChanges`,
  e adia o flush até `compositionend`; há tratamento específico para Safari/Android
  (`badSafariComposition`, `domobserver.ts:377`);
- colar/copiar (`clipboard.ts:6-121`) serializa com `DOMSerializer` e um marcador
  `data-pm-slice` (`clipboard.ts:35`) que preserva `openStart/openEnd`; `parseFromClipboard`
  (`clipboard.ts:44-121`) tenta HTML, cai para texto e aplica `transformPasted*`.

Isso confirma o custo: **metade da view é compatibilidade de navegador**. Um núcleo headless não
tem esse problema, mas também não ganha essa experiência de graça — o front-end precisará de uma
estratégia equivalente.

### 7.3 Atributos do elemento editável

`computeDocDeco` (`index.ts:517-535`) define `class="ProseMirror"`, `contenteditable` conforme
`editable`, mescla `attributes` (`index.ts:791`) e força `translate="no"` (`index.ts:533`) — este
último evita que o navegador traduza o conteúdo e quebre a sincronização.

**Nenhum `role`/`aria-*` é emitido.** Quem integra precisa adicionar `role="textbox"`,
`aria-multiline`, `aria-label` etc. via `attributes` — e o próprio ProseMirror documenta o
`contenteditable` como suficiente apenas para edição, não para leitores de tela.

A mesclagem de `attributes` tem regras que limitam a personalização (`index.ts:523-531`):
`class` é **concatenada** à classe global `ProseMirror` (não substituída), `style` é concatenado,
e um atributo já definido pelo core **não** é sobrescrito; `contenteditable` e `nodeName` são
explicitamente ignorados. Ou seja, `role`/`aria-*` entram, mas trocar a classe raiz ou o
`contenteditable` via `attributes` não funciona — `contenteditable` só muda pelo prop `editable`.

### 7.4 Acessibilidade: o que existe e o que não existe

| item | estado | evidência |
|---|---|---|
| elemento focável/editável | sim, `contenteditable="true"` | `index.ts:520` |
| `translate="no"` | sim | `index.ts:533` |
| classes de estado (`.ProseMirror-selectednode`) | sim | `viewdesc.ts:908-923`, `prosemirror.css:30-48` |
| `role`, `aria-*`, labels | **não** | `grep -rn "aria"` em `view/src` só acha "variants"/"variant" |
| navegação por teclado | comandos existem (`selectParentNode`, `selectAll`), mas o keymap padrão é mínimo | `commands.ts:426` (`selectParentNode`), `commands.ts:436` (`selectAll`), `commands.ts:752-760` (keymap) |
| foco/scroll | `focus()` na view, `handleScrollToSelection` como prop; `scrollIntoView()` é método da `Transaction`, não da view | `index.ts:338` (`focus`), `index.ts:241`/`:678` (`handleScrollToSelection`), `transaction.ts:206` (`scrollIntoView`) |
| leitor de tela | não há árvore acessível própria; depende do DOM + `contenteditable` | ausência de `aria` e de `role` no core |

Ou seja: acessibilidade é **responsabilidade do integrador** — um ponto em que nossa proposta
precisa ser melhor, porque um core headless que não define o contrato de a11y empurra o problema
para cada front-end.

## 8. Extensibilidade

Há quatro mecanismos, todos fora do núcleo:

1. **Schema**: novos `NodeSpec`/`MarkSpec` com `content`, `attrs`, `toDOM`, `parseDOM`
   (`schema.ts:358-530`). Nada no core precisa mudar.
2. **Plugins** (`plugin.ts:7-47`): `props` (eventos/handlers), `state` (campo), `view` (PluginView),
   `filterTransaction`, `appendTransaction`, `key`. `PluginKey` dá acesso sem instância
   (`plugin.ts:129-142`). Exemplo real: `history` (`history.ts:392-435`) e `collab`
   (`collab.ts:70-100`).
3. **Steps** customizados: basta estender `Step` e registrar `jsonID` (`step.ts:16-60`); o
   `Transform` não precisa saber do tipo concreto.
4. **Decorações** (`decoration.ts:123-300`): widgets, inline e node decorations com `spec`
   (`side`, `marks`, `stopEvent`, `ignoreSelection`, `key`), agrupadas em `DecorationSet` que
   também é mapeável (`decoration.ts:301-420`). É o caminho para comentários, cursores remotos,
   realce de busca — sem alterar o documento.

Limite: não há herança de schema nem merge genérico de specs; a composição é feita por funções
específicas e destrutivas (`addListNodes` reescreve o grupo `block`, ver
`ref/prosemirror/demo/demo.ts:8-11`), e cada produto remonta o schema inteiro — inclusive as marcas.

## 9. Performance

Evidência disponível:

- **Estruturas persistentes** com compartilhamento (`node.ts:138-157`, `fragment.ts:73-104`):
  editar um texto não copia o documento inteiro.
- **Mapeamento O(nº de passos)** com `Mapping` (`map.ts:255-286`); passos de marca têm mapa vazio.
- **Reconciliação incremental** de DOM (`viewdesc.ts:773-930`) e `MutationObserver` com flush
  agrupado (`domobserver.ts:83-97`).
- **Benchmarks oficiais** existem em `ref/prosemirror/demo/bench/` (`example.js`, `mutate.js`,
  `type.js`), rodando no browser (`demo/bench/index.js:1-30`). **Não executados** aqui: exigem
  browser + build do meta-repo.
- Medidas de tamanho feitas aqui (gzip, `dist/index.js`): model 29,9 KB, transform 20,4 KB,
  state 9,4 KB, view 61,2 KB.

Limites conhecidos (do código, não medidos): `DOMObserver.flush` reprocessa a região alterada e
parseia; documentos com milhares de nós por bloco e decorações densas são o caso patológico;
`EditorState.applyTransaction` roda todos os plugins e pode reexecutar `appendTransaction`
(`state.ts:137-170`), então plugins caros amplificam custo por transação.

## 10. Segurança e robustez

- Conteúdo HTML de colagem passa por `DOMParser` do schema (`from_dom.ts:182-340`), que só cria
  nós/marcas conhecidos — a allowlist é o schema, não um sanitizador de HTML. Atributos arbitrários
  não entram no documento (exceto via `getAttrs` do próprio schema).
- `ReplaceError`/`TransformError`/`RangeError` são os erros esperados; falhas de passo são
  resultados (`StepResult.fail`), não exceções.
- `Node.check()` (`node.ts:304`) valida conteúdo/attrs/marcas e é usado em testes/`createChecked`
  (`schema.ts:153`).

## 11. O que isso significa para o `rich-editor-auy`

1. Copiar o **vocabulário** (Step/StepMap/Mapping/Slice/ResolvedPos) evita reinventar semântica de
   posições; adaptar a representação para identificadores estáveis de referência.
2. Copiar o **modelo de validação por schema** (expressões de conteúdo + autômato) e o padrão de
   `defining`/`isolating`.
3. Copiar o padrão de **plugin com campos + appendTransaction**, que é extensível sem herança.
4. **Não** copiar: dependência de `contenteditable`, CSS obrigatório, a11y delegada, keymap mínimo
   e a dispersão em ~19 pacotes.
5. Onde somos diferentes: serialização canônica JSON no core + renderer separado; referências entre
   publicações como cidadãs de primeira classe (não offsets frágeis); zero CSS por contrato.
