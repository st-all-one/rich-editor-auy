# taino-edit — arquitetura

> Baseado em leitura do código em `ref/taino-edit/` (v0.7.0, commit `90ba5ed`,
> 2026-09-05) e em execuções registradas em `README.md`/`avaliacao.md`.
> Citações são `caminho:linha` relativas a `ref/taino-edit/` salvo quando a
> cópia está em `dossie/taino-edit/codigo/`.

## 1. Forma geral

Quatro camadas, com a dependência apontando só para baixo
(`DESIGN_NOTES.md:23-51`):

```
extensions  →  leptos / dioxus  →  dom  →  core
```

`core` é "sacrossanto": não pode depender de `web-sys`, Leptos ou Dioxus
(`DESIGN_NOTES.md:28-38`, `crates/taino-edit-core/Cargo.toml:14-18`). É a
decisão que faz o exemplo headless existir e que nós devemos copiar.

Tamanho real (excluindo testes, contagem `wc -l` / `find … | xargs cat | wc -l`):

| Camada | LOC |
| --- | --- |
| `taino-edit-core/src` | 6 667 |
| `taino-edit-dom/src` | 1 685 |
| `taino-edit-extensions/src` | 3 050 (tabela sozinha = 1 469) |
| `taino-edit-leptos/src` | 496 |
| `taino-edit-dioxus/src` | 571 |
| `taino-edit-table-view/src` | 146 |
| **Soma de `src/`** | **12 659** |
| **Total com testes (`*.rs` do repo)** | **23 732** |

O orçamento declarado era 11–15k LOC para o MVP e 20–26k para v1
(`DESIGN_NOTES.md:62-68`); está dentro — e a soma de `src/` (12 659) está na
faixa do MVP.

## 2. Modelo de documento

### 2.1 Árvore tipada

`Node` é um handle `Arc<NodeInner>` (`codigo/node.rs:102`) com
`type_: NodeType`, `attrs: Attrs`, `content: Fragment`, `marks: Vec<Mark>` e
`text: Option<String>` (`codigo/node.rs:85-100`). Texto é um nó — não há
representação separada de "text run".

- `NodeType` também é `Arc<NodeTypeInner>` com `id` estável por schema
  (`codigo/node.rs:20-30`); igualdade é por `id` (`node.rs:71-76`).
- `Fragment` é `Arc<FragmentInner>` com `content: Vec<Node>` e `size` **cacheado**
  (`ref/taino-edit/crates/taino-edit-core/src/fragment.rs:16-45`).
- `Attrs = BTreeMap<String, serde_json::Value>` — ordenado de propósito, para
  serialização determinística
  (`ref/taino-edit/crates/taino-edit-core/src/attrs.rs:1-16`).
- `Mark` é `(MarkType, Attrs)`; `same_mark_set` decide inclusividade
  (`crates/taino-edit-core/src/mark.rs`).

### 2.2 Ownership e copy-on-write

Não há mutação: cada operação devolve uma árvore nova, com `Arc` compartilhando
tudo o que não mudou. `copy_content`, `with_text`, `with_marks`, `with_attrs`
criam `NodeInner` novos (`codigo/node.rs:212-270`). `Fragment::replace_child`
clona o `Vec<Node>` e substitui um índice
(`fragment.rs:145-155`).

Isso é **copy-on-write por caminho**, não árvore persistente balanceada: cada
step que toca o documento reconstrói os `Fragment` no caminho da raiz até o
ponto editado. Para documentos grandes, o custo é O(filhos do pai) por nível, e
`Node::node_size()` de texto é **O(n) por chamada** porque faz
`t.chars().count()` (`codigo/node.rs:189-197`) — o tamanho do texto não é
cacheado, ao contrário do `size` do `Fragment`.

`PartialEq` de `Node` atalha por `Arc::ptr_eq` e depois compara campo a campo,
inclusive conteúdo (`codigo/node.rs:301-312`); `Fragment::eq` idem
(`fragment.rs:186-190`). É o que permite ao renderer dizer "subárvore igual,
não toque no DOM" (`codigo/view.rs:929-932`).

### 2.3 Posições

Modelo ProseMirror: tamanho de texto = quantidade de **scalars Unicode**
(`chars().count()`), folha não-texto = 1, qualquer outro nó =
`content.size + 2` (`codigo/node.rs:189-197`).

`ResolvedPos::resolve` produz um caminho de ancestrais com `index`, `start`,
`parent_offset` e helpers `before/after/start/end/text_offset/node_after/node_before`
(`ref/taino-edit/crates/taino-edit-core/src/pos.rs:1-190`).

**Divergência deliberada do ProseMirror**: posições são em scalars, não em
unidades UTF-16; a ponte DOM converte (`codigo/node.rs:92-97`). Isso é bom para
nós (Rust fala `char`), mas significa que todo cálculo de offset DOM precisa de
um mapa de conversão — que existe em `position_map.rs`.

## 3. Schema e autômato de conteúdo

`SchemaBuilder` declara nós e marks em ordem, atribui ids e compila cada
expressão de conteúdo (`codigo/schema.rs:105-235`).

A expressão é um subconjunto de regex — nomes, grupos, `|`, parênteses, `+`, `*`,
`?`, sem `{n,m}` —, tokenizada, parseada e compilada de NFA para DFA
(`crates/taino-edit-core/src/content.rs:1-12`). O resultado é um
`ContentMatch` que avança por ids de tipo e responde `valid_end()`,
`matches_complete()`, `match_type()` e `compatible()`
(`content.rs:29-110`).

Consequências práticas:

- `schema.node(name, attrs, children, marks)` **valida** e preenche defaults
  (`codigo/schema.rs:278-305`); `create_node` é a escotilha sem validação, usada
  por `ReplaceAroundStep` e depois revalidada pelo replace
  (`codigo/schema.rs:307-336`).
- `is_leaf` não vem de flag: é derivado de "o autômato não aceita nenhum filho"
  (`codigo/schema.rs:154-157`) — elegante e evita inconsistência.
- `marks` por nó existe no spec (`NodeSpec::marks`), mas `core` **não interpreta**
  a expressão de marks (`codigo/schema.rs:29-40`); aceitação de marks por tipo é
  só declarativa. É uma lacuna real de restrição de conteúdo.
- `MarkSpec::inclusive` é armazenado mas "not interpreted by core in v0.1"
  (`codigo/schema.rs:56-63`).

## 4. Steps, transações e mapeamento

### 4.1 O trait

```rust
pub trait Step: fmt::Debug + Send + Sync {
    fn apply(&self, doc: &Node, schema: &Schema) -> Result<Node, StepError>;
    fn get_map(&self) -> StepMap;
    fn invert(&self, doc: &Node) -> Result<Box<dyn Step>, StepError>;
    fn map(&self, mapping: &Mapping) -> Option<Box<dyn Step>>;
    fn to_json(&self) -> Value;
    fn clone_box(&self) -> Box<dyn Step>;
}
```

(`codigo/step.rs:35-55`.) Os cinco concretos: `ReplaceStep`, `ReplaceAroundStep`,
`AddMarkStep`, `RemoveMarkStep`, `AttrStep`. Cada um implementa invert (contra o
documento em que aplicou), map e JSON `{stepType: …}`
(`codigo/step.rs:112-205` e seguintes).

O ponto de extensão para CRDT/OT é `map_against(&Step)`, **documentado mas
ausente** — o `COLLAB_DESIGN.md:112-127` conclui que nem foi necessário para a
estratégia escolhida (loro faz o merge).

### 4.2 `Replace` — o algoritmo difícil

`Node::replace(from, to, slice, schema)` resolve as duas posições e chama
`replace_outer`, que escolhe entre:

1. descer recursivamente quando o índice é o mesmo e a profundidade permite;
2. `replace_two_way` quando o slice é vazio (remoção);
3. caminho rápido quando `open_start == open_end == 0` e as profundidades batem
   (concatenação de fragmentos no mesmo pai);
4. `replace_three_way` (inserção com conteúdo aberto) nos demais casos.

(`ref/taino-edit/crates/taino-edit-core/src/replace.rs:114-316`.)

É um port fiel de `replace.ts`, com validação de schema em cada `close()`
(`replace.rs:126-134`) e verificação de junção em `check_join`
(`replace.rs:136-147`). `choose the boring port` aqui foi a decisão certa: é
código correto e testado que não vale reinventar.

### 4.3 `Transform` e `Transaction`

`Transform` mantém `doc`, `steps`, `docs` (documento **antes** de cada step) e
`mapping` (`ref/taino-edit/crates/taino-edit-core/src/transform.rs:19-40`).
`docs` é o que permite `invert_steps()` sem reaplicar (`transform.rs:175-182`).

`Transaction` = `Transform` + `selection`, flags `selection_set`,
`add_to_history`, `join`, `history_intent` (`codigo/state.rs:263-272`).

`EditorState::apply` (`codigo/state.rs:176-217`):

1. se há `HistoryIntent`, desvia para `undo`/`redo` e **não** empilha nada;
2. seleção explícita vence; senão a seleção anterior é mapeada pelo `Mapping`;
3. se `add_to_history` e o doc mudou, **inverte todos os steps** e grava um
   `HistEntry` com `undo`, `redo`, `selection_before`, `selection_after`;
4. dobra os plugins;
5. devolve um `EditorState` novo (o antigo é imutável).

**Custo escondido:** `let mut history = self.history.clone()` a cada transação
(`codigo/state.rs:190`), com `Box<dyn Step>` sendo clonado profundamente. Em
100 grupos, são ~100 boxes novos por tecla. É o principal suspeito (não
perfilado) do crescimento do custo por tecla medido em `README.md` §5.

## 5. Histórico, seleção, comandos

### 5.1 Histórico

Linear, com limite default 100 (`codigo/state.rs:17`), duas pilhas
`done`/`undone`, `record(entry, join)` que limpa o redo e, quando `join`, funde
com a entrada anterior (`codigo/state.rs:74-102`). Agrupamento é **do chamador**:
`join_history()` existe (`codigo/state.rs:299-302`) mas só é usado num teste
(`crates/taino-edit-core/tests/state.rs:195`) — na prática cada tecla vira um
grupo de undo. Undo/redo restauram a seleção gravada
(`codigo/state.rs:219-258`).

Interessante: `History` **não** é um `Plugin`. O documento explica por quê
(`crates/taino-edit-core/src/plugin.rs:11-25`): plugins são observadores e não
podem reescrever o doc, e um "HistoryPlugin" foi avaliado e rejeitado.

### 5.2 Seleção

Enum `Copy` com `Text{anchor,head}`, `Node{pos}`, `Cell{anchor,head}`, `All`
(`codigo/selection.rs:12-30`). `map` mapeia endpoints com assoc=1, clampa no
tamanho do doc e **degrada para caret** quando o nó/célula foi tocado
(`codigo/selection.rs:75-110`).

Limitação honesta declarada: o mapeamento é puramente posicional, sem "procurar
uma seleção válida por perto" (`codigo/selection.rs:1-4`).

### 5.3 Comandos e keymap

`Command = Box<dyn Fn(&EditorState, Option<&mut Dispatch>) -> bool>`; sem
dispatch, o comando só **prova** que se aplica, com dispatch ele o faz
(`crates/taino-edit-core/src/commands.rs:24`). `chain` testa em ordem sem
meio-aplicar (`commands.rs:28-41`).

`Keymap` canonicaliza modificadores (`Mod` = Cmd no macOS, Ctrl fora dele),
faz duas passadas de lookup (com shift implícito para símbolos/maiúsculas) e
oferece `add` (substitui) e **`add_chained`** (o novo comando tenta primeiro, o
antigo vira fallback) — é assim que `Tab` funciona em tabela *e* lista
(`crates/taino-edit-core/src/keymap.rs:94-215`).

`base_keymap` tem **oito** bindings: Enter, Backspace, Delete, Mod-a, ArrowLeft,
ArrowRight, Home, End (`keymap.rs:218-242`). Não há Shift+Setas, Ctrl+Setas,
PageUp/Down, seleção por palavra — extensão de seleção fica com o browser nativo
(não é prevenida; ver §7.2).

## 6. Extensão: três eixos distintos

Isto é a parte conceitualmente mais valiosa do projeto.

1. **`Extension`** — contribui **schema + keymap** (`extensions/src/lib.rs:71-86`).
   `build_schema_with` empilha nós/marks (`lib.rs:88-105`);
   `build_keymap_with` parte de `base_keymap` e faz `add_chained` em cada
   binding (`lib.rs:107-128`). Uma extensão típica tem < 60 linhas
   (`extensions/src/bold.rs:1-36`).
2. **`Plugin`** — estado tipado por `EditorState`, dobrado a cada transação,
   **sem poder mudar o documento**; lookup por chave zero-sized
   (`crates/taino-edit-core/src/plugin.rs:71-130`). O estado é guardado como
   `Arc<dyn Any>` e `downcast` na leitura (`plugin.rs:210-256`).
3. **`ViewPlugin`** — reage a eventos DOM crus e contribui `Decoration`s,
   recebendo o `EditorView` (doc, schema, `pos_at_point`, `node_dom_at`)
   (`codigo/view.rs:20-60`). É o que permite a `TableView` fazer arrasto e
   redimensionamento sem contaminar o adapter genérico
   (`crates/taino-edit-table-view/src/lib.rs:1-40`).

Para o nosso caso: **copiar a separação em três eixos**. Ela resolve exatamente
o problema que vamos ter (schema estático vs. estado dinâmico vs. interação
específica de renderer).

## 7. Renderer e `contenteditable`

### 7.1 Estrutura

`EditorView` guarda `root`, `schema`, `doc`, `children: Vec<ViewDesc>`, flag
`composing`, `decorations`, `plugins` e um `overlay` (irmão do root)
(`codigo/view.rs:63-95`).

`ViewDesc` espelha a árvore: `Element{node, dom, children}` ou
`Text{node, text, wrapper}` (`crates/taino-edit-dom/src/desc.rs:13-45`). Para
texto com marks, o `wrapper` é o elemento mais externo e o nó de texto real fica
dentro (`codigo/view.rs:659-684`).

`mount` marca `contenteditable="true"`, garante `tabindex="0"`, esvazia o root
e renderiza cada filho (`codigo/view.rs:96-136`).

### 7.2 Eventos e o sequestro de teclas estruturais

Os adapters registram: `input`, `keydown`, `compositionstart`,
`compositionend`, `paste`, `mousedown/mousemove/mouseup`, `selectionchange`
(no `document`), `scroll` (capture) e `resize`
(`crates/taino-edit-leptos/src/lib.rs:229-448`). **Não** há `copy`, `cut`,
`beforeinput`, `dragstart`, `dragover` nem `drop` — confirmei por
`grep -n 'register("|push_listener'`.

Divergência de doc a notar: o rustdoc de `read_dom_changes` diz que os adapters
o ligam "behind a `MutationObserver`" (`dom/src/view.rs:456-457`), mas os dois
adapters usam o evento `input`
(`leptos/src/lib.rs:277-287`) — não há `MutationObserver` no wiring. É mais um
sinal de doc que não acompanhou o código.

O `keydown` é o núcleo do design "o editor é dono do teclado":

1. lê a seleção **viva** do DOM (`view.read_selection()`);
2. se ela difere da do modelo, aplica uma transação `no_history` só de seleção;
3. roda o keymap;
4. se algo aplicou, atualiza DOM + seleção **sincronamente**;
5. **sempre** faz `prevent_default` para Enter/Backspace/Delete.

(`crates/taino-edit-leptos/src/lib.rs:300-340`.)

Isso elimina a classe de bugs "caret atrasado", mas cobra o preço descrito no
`README.md` §3.1: sem keymap, essas teclas ficam mortas.

### 7.3 Leitura do DOM (o elo frágil)

`read_dom_changes` percorre os descs de texto, compara `text.data()` com
`node.text()` e, no **primeiro** divergente, calcula um diff de prefixo/sufixo
(`find_diff`, `codigo/view.rs:987-1016`) e devolve um `Transform`
(`codigo/view.rs:464-519`). Se nenhum texto mudou, procura texto inserido em
bloco que estava vazio (`find_empty_block_text`, `codigo/view.rs:744-772`).

Enquanto `composing` está ligado, retorna `None` — IME nunca gera transações
intermediárias (`codigo/view.rs:465-467`; teste
`crates/taino-edit-dom/tests/composition.rs:73-113`).

O que **não** é lido de volta: nós adicionados/removidos/movidos pelo browser,
quebra de linha nativa, colagem nativa, arrasto nativo. Tudo isso é
interceptado (ou simplesmente não acontece) — é a razão de os adapters
sequestrarem o teclado.

### 7.4 Patch incremental

`update(new_doc)` → `patch_children` compara posição a posição; se
`try_patch` falha, substitui o nó; filhos extras são removidos
(`codigo/view.rs:611-623`, `894-960`). `try_patch` mantém o DOM quando o nó é
igual, atualiza `text.set_data` quando só o texto mudou e recursa quando o tipo
e os attrs batem (`codigo/view.rs:927-985`).

Detalhe de engenharia bom de copiar: o `<br data-taino-trailing-break>` sintético
em blocos de texto vazios — sem ele, `<p></p>` tem altura zero e o caret não
entra. Ele é reconciliado em `reconcile_trailing_break`
(`codigo/view.rs:685-724`) e o **mesmo** marcador é emitido pelo serializador
SSR (`crates/taino-edit-dom/src/view_html.rs:24-45,80-124`).

### 7.5 Seleção DOM ↔ documento

`position_map.rs` faz o mapeamento bidirecional: posição de doc → `(node, offset)`
e volta, cobrindo fronteiras entre blocos, offsets dentro de textblock e offsets
dentro de text run (`crates/taino-edit-dom/src/position_map.rs:1-20`). `Cell` é
renderizada como range contíguo porque um `Range` do browser não pinta retângulo;
o destaque vem de decorations (`codigo/view.rs:390-430`).

### 7.6 Decorações

Duas formas (`crates/taino-edit-dom/src/decoration.rs:1-45`):

- `Node` → adiciona classe CSS ao elemento do nó;
- `Inline` → **overlay** de caixas absolutas, irmão do root, recalculado a cada
  scroll/resize.

A justificativa é excelente e vale copiar: envolver o range em `<span>` dividiria
os nós de texto e quebraria a leitura por `text.data()`. O overlay mantém o DOM
editável intocado (`decoration.rs:1-20`, `codigo/view.rs:248-325`).

## 8. Serialização

| Formato | Direção | Garantia | Fonte |
| --- | --- | --- | --- |
| JSON | ambos | validado contra o schema, formato ProseMirror | `crates/taino-edit-core/src/json.rs:1-10,96-128` |
| HTML | ambos | saída escapada; parser próprio estrito, `MAX_DEPTH=100`, tags desconhecidas são desembrulhadas | `crates/taino-edit-core/src/html.rs:1-31,606-615` |
| Markdown | ambos | hard-coded para os nomes canônicos das extensões; parse via `pulldown-cmark`, validado | `crates/taino-edit-core/src/markdown.rs:1-20` |
| HTML "de view" | saída | markup **exatamente** igual ao montado pelo editor (SSR) | `crates/taino-edit-dom/src/view_html.rs:1-45` |

O parser HTML próprio (sem engine de terceiros) é uma escolha defensável para
`core` sem dependências, mas é explicitamente "not a full HTML5 conformance
parser" (`html.rs:12-20`) — para colar conteúdo da web real ele vai perder
estrutura.

`to_markdown` documenta que estrutura sem equivalente Markdown é emitida como
HTML inline (`markdown.rs:1-10`).

## 9. Colaboração

**Não implementada.** Existe um documento de projeto detalhado
(`COLLAB_DESIGN.md`):

- escolha do `loro` (Peritext para marks, árvore móvel para blocos)
  (`COLLAB_DESIGN.md:36-51`);
- arquitetura **(B) modelo duplo com ponte no nível de operação**: o
  `EditorState` continua sendo a verdade local, um `CollabSession` mantém um
  `LoroDoc` sombra, steps locais viram ops loro e diffs remotos viram `Step`s
  aplicados por transação `no_history` (`COLLAB_DESIGN.md:52-78`);
- tabela de tradução `Step` → op loro, incluindo decomposição de `Replace` entre
  blocos em edições de texto + ops de árvore (`COLLAB_DESIGN.md:79-111`);
- posições em scalars Unicode dos dois lados, sem UTF-16 (`COLLAB_DESIGN.md:96-100`);
- crate próprio `taino-edit-collab`, justificado por disciplina de dependência
  (`COLLAB_DESIGN.md:129-172`);
- estratégia de teste: simulação determinística de dois peers + propriedades com
  RNG semeado, tudo host-side (`COLLAB_DESIGN.md:173-191`);
- **5 questões abertas** (Q1–Q5) não resolvidas: granularidade de átomos,
  seed do documento inicial, rota de undo, MSRV do `loro` e nome da feature
  (`COLLAB_DESIGN.md:192-205`).

Para nós: o desenho é um bom mapa de riscos, mas não há nada executável para
reaproveitar — e a conclusão de que `map_against` não foi necessário é um dado
útil (não superinvista nela).

## 10. Acessibilidade

Estado real:

- `contenteditable="true"` e `tabindex="0"` no root; `set_tabindex(-1|0)`
  disponível (`codigo/view.rs:102-106,343-347`);
- `focus()`/`has_focus()` (`codigo/view.rs:326-342`);
- nada de `role`, `aria-*`, `aria-multiline`, `aria-label`, `spellcheck`;
- toolbar dos exemplos usa `role="toolbar"`/`role="search"`, mas **sem**
  `aria-pressed` nos botões de toggle (`examples/basic-leptos/src/main.rs:300-340`);
- o ROADMAP declara "Full WCAG audit" fora de escopo
  (`ROADMAP.md:305-315`).

Ou seja: baseline de foco por teclado, nada de leitor de tela. Para o
`rich-editor-auy` isso é insuficiente e é uma das áreas onde devemos divergir.

## 11. Performance

Medições próprias (release, `CARGO_TARGET_DIR=/tmp`, melhor de 3):

| Blocos | µs/tecla | Observação |
| --- | --- | --- |
| 100 | 27 | |
| 1 000 | 29–65 | variação entre execuções |
| 5 000 | 94–119 | |
| 10 000 | 189 | |
| 20 000 | 825 | degradação superlinear |

Causas prováveis (`[INFERENCE]`, não perfilei):

1. `EditorState::apply` clona `History` inteiro e inverte todos os steps
   (`codigo/state.rs:190-198`);
2. `Node::node_size()` conta chars do texto a cada chamada
   (`codigo/node.rs:189-197`), e é chamado ao montar `Fragment`s durante o
   replace;
3. `Fragment::from_nodes` recalcula `size` somando `node_size` de todos os filhos
   (`fragment.rs:28-32`).

O ROADMAP já registra "Performance em documentos >10k nós" como risco
(`ROADMAP.md:323`) e adia armazenamento em rope.

## 12. Limitações estruturais (resumo)

1. Um único renderer (`contenteditable`); nenhum trait de renderer.
2. Leitura do DOM só de texto divergente; nada estrutural.
3. Sem stored marks; sem copy/cut/DnD nos adapters.
4. Restrição de marks por nó declarada e não aplicada.
5. `style` inline no documento (`text-align`, `width` de célula).
6. Wiring de eventos duplicado entre Leptos e Dioxus.
7. Histórico clonado por transação; agrupamento não usado pelos adapters.
8. Sem colaboração; sem a11y real.
