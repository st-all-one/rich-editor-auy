# Brainstorm 01 — Modelo, memória e álgebra

> **Status:** exploração crua. Este documento registra alternativas consideradas, o que cada uma
> custa e por que a recomendação venceu. Não é contrato; o contrato está em
> [`../02_decisoes.md`](../02_decisoes.md). Números marcados como "estimativa" não foram medidos
> neste projeto (que ainda não tem código); são ordens de grandeza para comparar alternativas.

---

## 1. Seis eixos de decisão

Qualquer modelo de editor é uma escolha em seis eixos que se puxam:

1. **Forma**: árvore, lista plana, rope, eventos lineares.
2. **Persistência**: mutável, imutável com clone, imutável com compartilhamento, arena versionada.
3. **Unidade de texto**: byte, escalar, UTF-16, grapheme, chunk.
4. **Marcas**: nós, metadado por run, bitset, conjunto internado.
5. **Mudança**: mutação direta, step invertível, delta, log de comandos.
6. **Extensão**: tipos Rust, schema em dados, macro, plugin dinâmico.

O erro clássico é otimizar um eixo isolado. Por exemplo: escolher uma árvore imutável com `Arc` por
nó (bom para snapshots) e depois descobrir que editar uma tabela de 100×100 clona 100 irmãos por
tecla. Ou escolher uma lista plana (boa para edição) e descobrir que listas aninhadas e tabelas não
têm representação elegante. O desenho tem que escolher **onde** o custo vai morrer.

---

## 2. Forma do documento

### 2.1 Hipótese A — árvore genérica de nós (ProseMirror-like)

```text
doc
├── paragraph
│   ├── text "Olá, "
│   └── strong
│       └── text "mundo"
└── bullet_list
    ├── list_item
    │   └── paragraph ...
```

**Prós:** expressividade máxima; qualquer aninhamento; schema uniforme; o modelo do ProseMirror é
testado há anos.

**Contras:** uma marca vira um nó, então aplicar/remover marca é cirurgia de árvore; posições dentro
de um parágrafo viram caminho; a UI precisa reconstruir runs a partir da árvore de marcas; marcas
aninhadas geram ambiguidade (a ordem das marcas vira parte da árvore).

**Custo de edição:** editar um texto no fundo de uma árvore clona o caminho até a raiz. Se cada nó
tem `Vec<filho>`, editar um item de uma lista de 1.000 itens clona o vetor de 1.000 irmãos. Em uma
tabela 100×100, editar uma célula pode clonar 100 linhas. É o custo que o ProseMirror paga e que o
taino-edit herda.

**Veredito:** rejeitada como *forma principal*. A parte boa — árvore de blocos — é absorvida; a
parte ruim — marcas como nós — é eliminada.

### 2.2 Hipótese B — árvore de blocos + inline plano

```text
doc
├── paragraph  [run("Olá, ", {}), run("mundo", {strong})]
└── bullet_list
    └── list_item
        └── paragraph [run("item", {})]
```

**Prós:** o conteúdo inline é uma sequência; posição dentro do parágrafo é offset; aplicar marca é
dividir runs; a UI recebe runs prontos; a árvore só existe para estrutura de bloco (listas,
citações, tabelas), que é editada com menos frequência.

**Contras:** conteúdo inline aninhado não é representável. Um link que contém um bloco, uma
anotação que contém um parágrafo, um “inline container” genérico — nada disso cabe. Na prática,
links são marcas; anotações ricas viram átomos/blocos; é uma restrição de escopo, não um bug.

**Custo de edição:** editar um textblock é local ao bloco; editar um container clona o caminho e os
irmãos do nível, como no PM, mas o nível de inline não paga árvore.

**Veredito:** **escolhida**. É o modelo que o guia do ProseMirror descreve (“inline content is
modeled as a flat sequence, with the markup attached as metadata”) e o que o taino-edit já usa.

### 2.3 Hipótese C — lista flat de blocos + Delta

```text
blocks: [paragraph("Olá, mundo", marks...), paragraph("...")]
changes: retain/insert/delete
```

**Prós:** edição e mudança muito simples; Delta é uma álgebra compacta; ótimo para texto linear.

**Contras:** listas aninhadas, tabelas, citações e qualquer estrutura hierárquica exigem extensões
ou uma árvore paralela. O Quill faz isso com Parchment/blots e paga o acoplamento ao DOM. Um editor
rico sem hierarquia é um editor de texto com negrito.

**Veredito:** rejeitada como modelo; **absorvida** como inspiração para o `ChangeSet`.

### 2.4 Hipótese D — rope de eventos

Representar o documento como uma sequência de tokens:

```text
[Open(paragraph), Text("Olá, "), Open(strong), Text("mundo"), Close(strong), Close(paragraph)]
```

Uma rope (árvore balanceada) de tokens com tamanhos de subárvore dá posições O(log n), edição
O(log n), snapshots persistentes e uma única estrutura. É elegante no papel.

**Contras:** toda consulta de UI precisa decodificar a sequência em árvore/blocos; validação de
schema após um splice é complexa; depuração é difícil; operações como “split block” inserem
fechamento/abertura e precisam normalizar; o ganho sobre uma árvore com chunks é incerto.

**Veredito:** rejeitada para o MVP. É uma possível otimização de backend se as medições mostrarem
que a árvore com `Vec<Arc<Block>>` não escala — mas seria um backend atrás da mesma API, não uma
mudança de modelo.

### 2.5 Hipótese E — DOM como verdade

**Prós:** zero modelo; o navegador edita; `execCommand` faz o trabalho.

**Contras:** sem invariantes, sem undo correto, sem bytes canônicos, sem colaboração, sem testes
diferenciais. É o Summernote.

**Veredito:** rejeitada. É o defeito que o projeto existe para não repetir.

---

## 3. Persistência e snapshots

### 3.1 Clone total por transação

```rust
let next = doc.clone();
apply_steps(&mut next, steps)?;
```

**Custo:** O(documento) por tecla. Um doc de 1 MB com 10k blocos copia pelo menos 1 MB + metadados.
A 10 GB/s, ~100 µs; acima do orçamento. **Rejeitado.**

### 3.2 `Arc` por nó + estrutura persistente

```rust
struct Node(Arc<NodeInner>);
struct NodeInner { children: Vec<Node>, ... }
```

**Custo:** editar um filho clona o caminho e o vetor de irmãos em cada nível. Para um parágrafo no
topo, O(1) irmãos; para uma célula de tabela, O(linhas). Snapshots são baratos (Arc bump). O taino
usa isso.

**Veredito:** bom para snapshots, ruim para edições em estruturas largas. **Não escolhido como
forma principal**, mas a ideia de `Arc` no topo é absorvida.

### 3.3 COW com `Arc<DocInner>` + `Vec<Arc<Block>>`

```rust
struct Doc(Arc<DocInner>);
struct DocInner { blocks: Vec<Arc<Block>>, size: Size }
```

- Dono único: `Arc::try_unwrap`/`make_mut` muta o vetor e o bloco no lugar.
- Snapshot retido: clona o `DocInner` (cópia do vetor de ponteiros) e o bloco tocado.
- O caminho comum é O(edição); o caminho com snapshot é O(#blocos) em ponteiros, não em conteúdo.

**Custo estimado:** 10k blocos = 80 KiB de ponteiros; cópia + alocação ~3–10 µs. 100k blocos =
800 KiB; ~30–50 µs (gatilho de chunking).

**Veredito:** **escolhido**. É o melhor custo/benefício sem `unsafe` e sem vetor persistente.

### 3.4 Arena versionada

```rust
struct Arena { nodes: Vec<NodeData> }
struct Doc { arena: Arena, root: NodeId, version: u32 }
```

**Prós:** nós contíguos, sem `Arc` por nó, índices `u32`, snapshots por versão. **Contras:** GC/
retain de versões, ponteiros inválidos, complexidade de ciclo de vida, difícil de tornar `Send`
com snapshots sem locks ou cópia. **Rejeitado** para o MVP; possível backend futuro.

### 3.5 Vetor persistente (RRB/ChunkedVec)

**Prós:** splice O(log n), snapshots O(1), sem cópia de 100k ponteiros. **Contras:** implementação
significativa (~1–2k LOC), iteração mais lenta, risco de bug sutil, necessidade de benchmark para
justificar.

**Veredito:** **adiado** (D06). O MVP usa `Vec<Arc<Block>>`; se S4 estourar o orçamento, o
`ChunkedVec` entra atrás da mesma API interna.

---

## 4. Armazenamento de texto

### 4.1 Um `String` por textblock

**Custo:** inserir no meio de um parágrafo de 1 MiB move ~1 MiB. A 30 GB/s, ~33 µs. Para 100 KiB,
~3 µs. Para 10 KiB, ~0,3 µs. Depende do tamanho do parágrafo.

**Veredito:** simples, mas sem limite superior. Um único parágrafo gigante (colado de um PDF) trava.

### 4.2 Runs com teto de 4 KiB

Dividir o texto em runs de no máximo 4 KiB (em fronteira de escalar UTF-8). O teto é uma invariante
canônica, não semântica.

**Custo:** inserir move ≤ 4 KiB (~0,13 µs); encontrar o run custa O(#runs no bloco) ≤ 256 para 1 MiB.
Se o scan de 256 for caro, um índice local de prefixos (256 × u32 = 1 KiB) resolve com binária.

**Prós:** simples, usa `String`, sem estrutura exótica, pior caso limitado, serialização
determinística (a divisão é canônica).

**Contras:** mais runs do que o necessário; a UI precisa mesclar visualmente; um run “lógico” pode
ser vários runs físicos.

**Veredito:** **escolhido** (D03).

### 4.3 Gap buffer

**Prós:** inserção/deleção O(1) amortizado perto do gap; ideal para edição sequencial. **Contras:**
duas fatias, conversão para contígua na serialização, complexidade de marks/ranges, gap pode
“viajar” e custar O(n) em saltos.

**Veredito:** rejeitado no MVP; possível otimização por textblock se S3 estourar.

### 4.4 Rope

**Prós:** O(log n) para qualquer edição, independente do tamanho. **Contras:** overhead por nó,
iteração mais lenta, complexidade de implementação e de teste, exagero para parágrafos típicos.

**Veredito:** rejeitado no MVP. Runs limitados cobrem o pior caso realista.

### 4.5 Piece table

**Prós:** edição barata, undo natural, texto original imutável. **Contras:** acesso aleatório
O(#pieces), marks/ranges complexos, memória de peças, difícil de tornar canônico.

**Veredito:** rejeitado.

---

## 5. Unidade de posição

| Unidade | Indexar | Fatiar | UI web | UI nativa | Grapheme | Veredito |
|---|---|---|---|---|---|---|
| byte UTF-8 | O(1) | O(1) | precisa converter | precisa converter | não | **escolhida** |
| `char` (escalar) | O(n) | O(n) | precisa converter | natural em Rust | não | rejeitada (taino usa) |
| UTF-16 | O(1) | O(1) | natural | precisa converter | não | rejeitada (core não é JS) |
| grapheme | O(n) | O(n) | precisa converter | precisa converter | sim | rejeitada (é `motion`, não modelo) |
| (bloco, offset) | O(1) | O(1) | precisa compor | precisa compor | depende | rejeitada (não serve seleção entre blocos) |

**Decisão:** byte UTF-8 + tokens estruturais. A UI converte para a unidade dela; `motion` converte
para grapheme. O core garante fronteira de escalar, não de grapheme.

**Custo de conversão:** `pos_to_utf16` percorre o texto do bloco até a posição; para 1 KiB, ~0,1 µs;
para 1 MiB, ~100 µs. A UI deve cachear por bloco/linha, não converter em cada frame.

---

## 6. Marcas

### 6.1 `Vec<Mark>` por run

**Prós:** simples, suporta atributos. **Contras:** alocação por run mesmo sem marca; comparação
O(n); ordem precisa ser canonicalizada.

### 6.2 Máscara `u64` + attrs

**Prós:** 8 bytes no caso comum; comparação O(1); ordem implícita; zero alocação para marcas sem
atributo. **Contras:** limite de 64 tipos; marcas com atributos precisam de side slice.

**Veredito:** **escolhida** (D11). 64 marcas é muito para um schema de rich text; se um consumidor
precisar de mais, é sinal de schema mal desenhado.

### 6.3 Conjunto internado (`MarkSetId`)

**Prós:** run de 8 bytes + texto; comparação O(1); ótimo para muitos runs. **Contras:** tabela
interna por documento, GC, serialização, complexidade de igualdade entre documentos.

**Veredito:** adiado; a máscara resolve o caso comum.

### 6.4 Marcas como nós (modelo A)

Já rejeitado: transforma edição de marca em cirurgia de árvore e cria ambiguidade de aninhamento.

---

## 7. Representação de mudança

### 7.1 Mutação direta

```rust
doc.block_mut(3).insert_text(5, "x");
```

**Prós:** simples. **Contras:** sem undo, sem colaboração, sem mapeamento, sem auditoria; a UI muta
o que não deveria. **Rejeitado.**

### 7.2 Trait object por step

```rust
trait Step { fn apply(&self, ...); fn invert(&self, ...); }
type BoxedStep = Box<dyn Step>;
```

**Prós:** extensível; cada step encapsula lógica. **Contras:** alocação por step, dispatch dinâmico,
`match` impossível, serialização por downcast, `Send + Sync` obrigatório, variantes novas quebram
suposições. O taino usa isso.

**Veredito:** rejeitado (D16).

### 7.3 Enum fechado de steps

**Prós:** zero alocação, `match` exaustivo, serialização direta, `Copy` onde couber, fácil de
testar. **Contras:** step customizado exige composição com os existentes.

**Veredito:** **escolhido**. Se um dia um step customizado for necessário, ele é expresso como
`Replace`/`SetNodeAttr`; um trait object pode ser extensão, não base.

### 7.4 Log de comandos

**Prós:** replay de alto nível, undo semântico. **Contras:** comandos dependem do estado; replay
não é determinístico se o estado mudou; não serve para colaboração fina. **Rejeitado.**

### 7.5 Splice universal

Um único step `Replace(from, to, slice)` cobre texto, split, join e wrap (com slices abertos). **Prós:**
minimalismo. **Contras:** wraps/lifts eficientes precisam de `ReplaceAround` para não copiar o
conteúdo; marks precisam de steps próprios para não reescrever texto.

**Veredito:** `Replace` + `ReplaceAround` + `AddMark`/`RemoveMark` + `SetAttr` (6 variantes).

---

## 8. Schema

### 8.1 Tipos Rust estáticos por nó

```rust
struct Paragraph { runs: Vec<Run> }
struct Heading { level: u8, runs: Vec<Run> }
enum Node { Paragraph(Paragraph), Heading(Heading), ... }
```

**Prós:** máxima segurança de tipo, pattern matching, zero lookup. **Contras:** o consumidor não
consegue adicionar um nó sem recompilar o core; extensões viram um enum gigante; schema dinâmico
(servidor) é impossível; o core precisa conhecer todos os nós.

**Veredito:** rejeitado como única forma. O schema é dado, não um enum global.

### 8.2 Schema em dados (runtime)

```rust
let schema = SchemaBuilder::new()
    .node("paragraph", NodeSpec { content: "text*", .. })
    .build()?;
```

**Prós:** extensível, carregável, testável, o core não conhece nós. **Contras:** validação em
runtime, lookup por id, erros só na construção.

**Veredito:** **escolhido** como fonte de verdade.

### 8.3 Macro de schema (compile-time)

```rust
let schema = schema! {
    doc { content: "block+" }
    paragraph { group: "block", content: "text*", marks: "_" }
    ...
};
```

**Prós:** valida nomes/expressões/attrs em compilação; gera acessores tipados; documentação e
autocomplete. **Contras:** mais um crate, mais complexidade, não serve para schema dinâmico.

**Veredito:** **front-end opcional** do builder (D09). O builder é a verdade; a macro é açúcar
verificado.

### 8.4 Macro `macro_rules!` em vez de proc-macro

**Prós:** sem crate extra, sem dependência, compila mais rápido. **Contras:** não valida strings de
content expression em compilação; a validação volta para runtime.

**Veredito:** rejeitado para o objetivo de “erro de schema em compilação”; um `macro_rules!` pode
existir como atalho, mas a macro principal é proc-macro.

---

## 9. Serialização

| Formato | Tamanho | Velocidade | Determinismo | Interop | `no_std` | Veredito |
|---|---|---|---|---|---|---|
| JSON compacto | médio | bom | fácil | máximo | sim | **escolhido** |
| JSON pretty | maior | pior | não (whitespace) | máximo | sim | saída humana, não canônica |
| CBOR | menor | bom | fácil | médio | sim | pós-MVP |
| MessagePack | menor | bom | fácil | médio | sim | pós-MVP |
| binário custom | menor | ótimo | fácil | baixo | sim | pós-MVP, se necessário |
| HTML | maior | depende | depende | máximo | depende | adaptador, não canônico |

**Writer:** nosso, em `fmt::Write`, sem árvore intermediária. **Parser:** nosso, atrás de feature,
fuzzado; fallback `serde_json` na feature se necessário. **Envelope:** versão + fingerprint + doc.

---

## 10. Plugins e extensão

### 10.1 Registry genérico

```rust
trait Plugin { type State; fn init(...); fn apply(...); }
struct Plugins { states: HashMap<TypeId, Box<dyn Any>> }
```

**Prós:** qualquer plugin, estado próprio. **Contras:** `Any`/`TypeId`, ordem de inicialização,
compatibilidade, ciclo de vida, `Send + Sync`, difícil de documentar. O taino tentou e manteve o
`History` fora do trait.

**Veredito:** adiado (D30). O MVP tem pontos de extensão declarativos: schema, comandos, input
rules, keymap, decorações calculadas.

### 10.2 Hooks declarativos

Funções com assinatura fixa que o consumidor fornece: `fn(&EditorState) -> Option<Transaction>`,
`fn(&Doc) -> Vec<Decoration>`, `fn(&str) -> Option<InputRuleMatch>`. **Prós:** simples, testável,
sem estado escondido. **Contras:** menos poderoso que um registry.

**Veredito:** **escolhido** para o MVP.

---

## 11. Colaboração

| Abordagem | Prós | Contras | Veredito |
|---|---|---|---|
| Nenhuma | escopo | consumidor precisa resolver | MVP |
| OT | maduro, steps pequenos | transformação de pares, servidor, complexidade | pós-MVP |
| CRDT (Yjs/Loro/Automerge) | offline, convergência | IDs, tombstones, metadata, tamanho | pós-MVP, crate separado |
| Log de steps + rebase | alinhado ao modelo | sem convergência automática | base para pós-MVP |

**Decisão:** o core garante steps serializáveis e mapeáveis, sem IDs de CRDT. A colaboração é um
crate separado que traduz steps para o transporte escolhido. O doc canônico continua sem IDs de
CRDT e sem IDs efêmeros de UI; âncoras opcionais (`NodeId`) são a exceção explícita para referência
entre publicações.

---

## 12. UI

O core não escolhe DOM/canvas/nativo. A pergunta “como a UI desenha?” tem respostas diferentes:

- **DOM**: contenteditable é hostil; a UI pode usar DOM só para pintar, com `beforeinput`/IME.
- **Canvas**: a UI mede e pinta; o core só dá runs/ranges.
- **Nativa**: a UI usa a API de texto do toolkit; o core dá o modelo.
- **Terminal**: a UI pinta com células; o core dá o texto e as marcas.

O que o core precisa oferecer para todos: iteração de blocos/runs, ranges absolutos, seleção
lógica, change set, conversão de posição, motion lógico. O que ele não pode oferecer: geometria.

---

## 13. O que não exploramos (e por quê)

- **Editor de código** (syntax tree, indentação, folding): outro domínio; o modelo de rich text
  não é o de código.
- **Layout/bidi/shaping**: pertence à UI; o core não mede texto.
- **Colaboração com servidor**: pós-MVP; o modelo de steps é a fronteira.
- **Formato binário**: pós-MVP; JSON é suficiente para provar o modelo.
- **Plugins dinâmicos**: adiado; o custo não se paga antes do MVP.
- **Multi-seleção**: adiada; a API de seleção pode evoluir para `SelectionSet`.
- **Tabelas complexas**: o modelo suporta blocos aninhados; a UX de tabela é pós-MVP.

## 14. Inferências e perguntas abertas

- **[INFERENCE]** O custo de COW em 10k blocos (~3–10 µs) é uma estimativa de memcpy + alocador; a
  medição real decide se o alvo de 10 µs é viável.
- **[INFERENCE]** O custo de runs de 4 KiB em um parágrafo de 1 MiB (~256 runs) é dominado pelo
  memmove de 4 KiB, não pelo scan; a medição decide se um índice local é necessário.
- **Pergunta:** o `MarkSet` com máscara `u64` cobre todos os schemas reais? Se não, o limite de 64
  é aceitável ou precisamos de `u128`/heap?
- **Pergunta:** o parser JSON próprio justifica o custo versus `serde_json` na feature? A resposta
  depende do tamanho WASM e do `no_std`.
- **Pergunta:** a macro `schema!` deve ser proc-macro desde o início ou `macro_rules!` + validação
  em runtime no MVP? A recomendação é proc-macro, mas o custo de manutenção é real.
- **Pergunta:** o `ChangeSet` por ranges é suficiente para uma UI de canvas que precisa repintar
  linhas visuais? Pode ser necessário um `dirty_rows` calculado pela UI.
