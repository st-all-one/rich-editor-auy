# Plano de dados e lógica para um editor integrável

## 0. Decisão central

O WASM deixa de ser uma coleção de funções `JSON -> JSON` e passa a ser um **editor stateful**:

```text
UI (fora do projeto)
  -> comandos semânticos
  -> Editor / Transaction
  -> EditorState (Rust)
  -> Applied { ChangeSet, Selection, HistoryInfo }
  -> View/DocView para a UI renderizar
```

A UI continua responsável por DOM, eventos, layout e pintura. O projeto entrega **dados, posições, comandos, validação, histórico e change sets**. Nada de `contenteditable` no core.

Também há uma decisão de produto importante:

> **Só existe um schema público: o schema completo.**
> `Schema::basic`, `basicSchema`, `documentSchema` e schemas customizados deixam de ser API pública. O builder continua existindo apenas como ferramenta interna/testkit.

---

## 1. O que o WASM deve expor como dados

Hoje o TS vê basicamente `Document` e `EditorState` com operações isoladas (`insertText`, `deleteRange`, `splitBlock`, `joinBackward`, `toggleMark`). Falta o contrato de dados que uma UI precisa para renderizar e reconciliar.

### 1.1 `DocView`

Uma projeção de renderização, separada da `A11yTree`:

```ts
interface DocView {
  version: number;
  attrs: AttrMap;              // nomes, nunca AttrId
  blocks: BlockView[];
}

interface BlockView {
  index: number;
  kind: string;                // "paragraph", "table_cell", ...
  id?: string;
  attrs: AttrMap;
  range: [number, number];
  text?: string;
  runs?: RunView[];
  atoms?: AtomView[];
  children?: BlockView[];
}

interface RunView {
  text: string;
  range: [number, number];
  marks: MarkView[];           // strong, em, link(href), ref(target)...
}

interface AtomView {
  kind: string;                // "image", "hard_break", ...
  attrs: AttrMap;              // alt, src, width, height, ...
  range: [number, number];
}
```

`BlockView.attrs` e `AtomView.attrs` são a fonte para:

- `width`/`height`/`intrinsic_width`/`intrinsic_height` de imagem;
- `start`, `reversed`, `tight` de listas;
- `language`, `meta` de code block;
- `colspan`, `rowspan`, `scope`, `align` de células;
- `caption`, `credit`, `title`, `alt`, `srcset`;
- `id`, `lang`, `dir`, metadados etc.

A `A11yTree` continua existindo, mas **não substitui** o `DocView`. Ela é semântica, não é árvore de apresentação.

### 1.2 `Selection`

Serializar e aceitar seleção explicitamente:

```ts
type Selection =
  | { type: "none" }
  | { type: "caret"; pos: number; assoc: -1 | 1 }
  | { type: "text"; anchor: number; head: number; assoc: -1 | 1 }
  | { type: "node"; pos: number }
  | { type: "table"; table: number; anchor: CellRef; head: CellRef };
```

O caso `table` é necessário para operações de linha/coluna/merge sem fingir que é seleção de texto.

### 1.3 `ChangeSet` utilizável

O `ChangeSet` atual existe, mas `blocks` é praticamente `0..block_count` e `full` fica verdadeiro com mais de um range. Para preview incremental e reconciliação, ele precisa informar blocos realmente afetados.

```ts
interface ChangeSet {
  version: number;
  blocks: { from: number; to: number };
  ranges: Array<{ old: [number, number]; new: [number, number] }>;
  selection: boolean;
  full: boolean;
  insertedBlocks?: number[];
  removedBlocks?: number[];
}
```

A UI usa isso para atualizar apenas os blocos alterados. O preview, quando existir, usa o mesmo dado.

### 1.4 `Applied`

`apply`, `undo` e `redo` devolvem um resultado, não um documento inteiro:

```ts
interface Applied {
  version: number;
  changes: ChangeSet;
  selection: Selection;
  history: HistoryInfo;
}
```

O documento permanece no `Editor`; a UI consulta `editor.view(range?)` ou `editor.block(index)`.

---

## 2. Schema completo único

### 2.1 Estrutura mínima

Substituir `Schema::basic()` por:

```rust
impl Schema {
    pub fn full() -> Result<Self, SchemaError>;
}
```

O schema completo deve conter, no mínimo:

**Blocos**
- `paragraph`
- `heading` — `level: 1..6`
- `bullet_list` — `tight`, `checked?`
- `ordered_list` — `start`, `reversed`, `tight`, `type` (`1`, `a`, `A`, `i`, `I`)
- `list_item`
- `blockquote`
- `code_block` — `language`, `meta`, `line_numbers`
- `raw_code` — `language`, `format`, `filename`, `meta`, `line_numbers`; payload textual, nunca HTML/XML executável
- `table` — `summary`, `col_widths`
- `table_caption`
- `table_row`
- `table_header` / `table_cell` — `colspan`, `rowspan`, `scope`, `align`, `abbr`, `headers`
- `figure`
- `caption`
- `separator`
- `image` — inline atom
- `video`, `audio`, `attachment` — átomos/blocos conforme o renderer final
- `hard_break` — inline atom

**Marcas**
- `strong`, `em`, `code`
- `underline`, `strike`, `highlight`, `sup`, `sub`
- `link` — `href`, `title`, `rel`, `target`
- `ref` — `target: RefTarget`

**Inline**
- `text`
- `image`
- `hard_break`

### 2.2 Atributos que faltam hoje

| Dado | Onde | Atributos |
|---|---|---|
| Imagem | `image` | `src`, `alt` obrigatório, `title`, `width`, `height`, `intrinsic_width`, `intrinsic_height`, `srcset`, `sizes`, `caption`, `credit`, `loading`, `decoding` |
| Lista ordenada | `ordered_list` | `start`, `reversed`, `tight`, `type` |
| Código | `code_block` | `language`, `meta`, `line_numbers` |
| Código bruto | `raw_code` | `language`, `format` (`xml`, `html`, `json`, `yaml`, `sql`, `text`...), `filename`, `meta`, `line_numbers` |
| Tabela | `table` | `summary`, `col_widths` |
| Célula | `table_header`/`table_cell` | `colspan`, `rowspan`, `scope`, `align`, `abbr`, `headers` |
| Parágrafo/heading | bloco | `align` |
| Documento | root | `lang`, `dir`, `title`, `description`, `canonical_url`, `authors`, `tags`, `metadata` |
| IDs | blocos | `id` já reservado; manter estável |
| Link | mark | `href`, `title`, `rel`, `target` |

`width` e `height` entram como inteiros positivos, não floats, para manter o wire canônico. `intrinsic_*` serve para o preview/renderer calcular proporção sem depender de carregar a imagem.

### 2.3 Sistema de atributos

O `AttrType` atual cobre `Str`, `Int`, `Bool`, `Ref`, `Any`. Para o schema completo, adicionar:

```rust
pub enum AttrType {
    Str,
    Int { min: i64, max: i64 },
    Bool,
    Ref,
    Enum(Box<[Box<str>]>),       // align, scope, loading, decoding...
    List(Box<AttrType>),          // authors, tags, headers
    Map(Box<AttrType>),           // metadata
    Any,
}
```

E no valor:

```rust
pub enum AttrValue {
    Null,
    Bool(bool),
    Int(i64),
    Str(Box<str>),
    Array(Box<[AttrValue]>),
    Map(Box<[(Box<str>, AttrValue)]>), // ordenado por chave
    Ref(RefTarget),
}
```

Isso permite `metadata` como objeto canônico, com chaves ordenadas, limites de profundidade/entradas e sem `HashMap`. O envelope continua carregando metadados de publicação (`doc_id`, versão); `Doc.attrs.metadata` é o metadado editorial.

### 2.4 Passo para atributos inline

Hoje `SetNodeAttr` atinge o bloco que contém a posição. Uma imagem é um átomo inline; não existe step para alterar `width`, `height`, `alt` ou `src`.

Adicionar:

```rust
Step::SetInlineAttr {
    from: Pos,
    to: Pos,          // range do átomo
    attr: AttrId,
    value: AttrValue,
}
```

E na transação:

```rust
tx.set_inline_attr(range, attr, value)?;
tx.set_mark_attr(range, mark_id, attr, value)?;
```

Sem isso, “redimensionar imagem” e “editar link” não são representáveis no histórico.

### 2.5 Migração e schema único

- `Schema::full()` vira a única construção pública.
- `SchemaBuilder` passa a `pub(crate)`/testkit.
- `Schema::basic()` pode sobreviver apenas como fixture privada de migração, nunca exportada.
- `Envelope` ganha `schema_version = 2` e fingerprint do schema completo.
- `migrate_v1_to_full` aceita documentos básicos, adiciona defaults e reemite envelope; documentos com tipos desconhecidos falham com `UnsupportedNode`.
- Bench `all_types` deixa de montar `rich_schema` e passa a usar `Schema::full()`.
- Remover `basicSchema()`/`documentSchema()` do pacote TS; expor `SCHEMA_FINGERPRINT` e tipos.

### 2.6 Política de tokens: apenas tipos conhecidos

Todo token recebido deve ser um nó, mark, átomo ou atributo declarado no schema completo.
**Token desconhecido não é armazenado, não é convertido em `Any` e não é descartado em silêncio.**
A ingestão falha atomicamente com `ProblemDetails` (RFC 9457):

```json
{
  "type": "https://rich-editor-auy.dev/problems/unknown-token",
  "title": "Unknown AUY token",
  "status": 422,
  "detail": "The token 'chart' has no representation in the full schema.",
  "instance": "blocks[3].kind",
  "code": "unknown_token",
  "suggestion": "Render the chart as an image with alt text, or omit it before import."
}
```

Campos mínimos: `type`, `title`, `status`, `detail`, `instance`, `code`, `suggestion`.
O mesmo formato cobre `unknown_mark`, `unknown_attr`, `invalid_attr`, `unrepresentable_feature`,
`missing_asset`, `duplicate_id`, `stale_batch` e `limit_exceeded`.

Não existe `embed` como escape hatch. O conversor deve mapear antes de chamar o AUY. A documentação
e o erro devem recomendar a conversão:

| Origem | Destino AUY | Observação |
|---|---|---|
| gráfico | `image` | renderizar como imagem com `alt`, `caption` e `asset_id` |
| fórmula | `image` | renderizar como imagem; `alt` com a fórmula textual |
| quebra de página | **não representável** | erro `unrepresentable_feature`; remover ou usar `separator` se o produto aceitar semântica diferente |
| XML/HTML/código bruto | `raw_code` | payload textual, `language`/`format`; o renderer escapa, nunca executa |
| text box | `blockquote`/`paragraph` | achatar conteúdo na ordem de leitura |
| comentário | **não representável** | erro; conversor remove ou exporta separadamente |
| tracked change | **não representável** | converter aceita/rejeita antes da ingestão |
| nota de rodapé | **não representável** até schema próprio | erro com sugestão de corpo/`raw_code` |
| desconhecido | **erro** | `unknown_token` com sugestão |

`raw_code` é um bloco `text*` sem marks, com role a11y de código e `code_language` exposto no
`A11yNode`/`DocView`. Ele representa XML, HTML, JSON, SQL ou qualquer fonte bruta como texto
seguro; não é HTML renderizado.

---

## 3. API WASM stateful

### 3.1 Classes, não funções soltas

```ts
class Editor {
  static fromJSON(json: string, options?: EditorOptions): Editor;
  static fromEnvelope(bytes: string | Uint8Array): Editor;

  toJSON(): string;
  view(range?: BlockRange): DocView;
  block(index: number): BlockView | null;
  a11yTree(): A11yNode;
  selection(): Selection;
  setSelection(selection: Selection): void;

  transaction(): Transaction;
  apply(tx: Transaction): Applied;

  // comandos de alto nível na seleção atual
  insertText(text: string): Applied;
  deleteBackward(): Applied;
  deleteForward(): Applied;
  deleteSelection(): Applied;
  splitBlock(): Applied;
  insertHardBreak(): Applied;
  setBlockType(kind: string, attrs?: AttrMap): Applied;
  toggleMark(name: string, attrs?: AttrMap): Applied;
  setNodeAttr(name: string, value: AttrValue): Applied;
  setInlineAttr(name: string, value: AttrValue): Applied;
  setDocAttr(name: string, value: AttrValue): Applied;
  setNodeId(pos: number, id: string | null): Applied;

  // histórico
  canUndo(): boolean;
  canRedo(): boolean;
  undo(): Applied | null;
  redo(): Applied | null;
  history(): HistoryInfo;
  clearHistory(): void;

  // composição
  beginComposition(range: [number, number]): void;
  updateComposition(text: string): Applied;
  commitComposition(): Applied;
  cancelComposition(): Applied;

  // clipboard/estrutura
  slice(from: number, to: number): SliceJSON;
  insertSlice(at: number, slice: SliceJSON, origin?: Origin): Applied;

  validate(): ValidationReport;
  validateA11y(): A11yReport;
}
```

### 3.2 `Transaction`

A transação é o caminho para operações atômicas e para a UI agrupar comandos:

```ts
class Transaction {
  insertText(at: number, text: string): void;
  delete(from: number, to: number): void;
  replace(from: number, to: number, slice: SliceJSON): void;
  splitBlock(at: number): void;
  joinBackward(at: number): void;
  setBlockType(at: number, kind: string, attrs?: AttrMap): void;
  addMark(range: [number, number], mark: MarkJSON): void;
  removeMark(range: [number, number], mark: MarkJSON): void;
  setNodeAttr(at: number, name: string, value: AttrValue): void;
  setInlineAttr(range: [number, number], name: string, value: AttrValue): void;
  setMarkAttr(range: [number, number], mark: string, name: string, value: AttrValue): void;
  setDocAttr(name: string, value: AttrValue): void;
  setNodeId(at: number, id: string | null): void;
  setSelection(selection: Selection): void;
  setOrigin(origin: Origin): void;
  setGroup(group: number): void;
  setAddToHistory(add: boolean): void;
  applyComposition(event: CompositionEvent): void;

  // tabela
  insertTable(at: number, rows: number, cols: number, headerRows: number): void;
  insertRow(cell: CellRef, where: "before" | "after"): void;
  deleteRow(cell: CellRef): void;
  insertColumn(cell: CellRef, where: "before" | "after"): void;
  deleteColumn(cell: CellRef): void;
  mergeCells(range: CellRange): void;
  splitCell(cell: CellRef): void;
  setCellHeader(cell: CellRef, header: boolean): void;
  setCellSpan(cell: CellRef, colspan: number, rowspan: number): void;
}
```

### 3.3 Regras de consistência

- `Transaction` carrega a versão/base do editor. Se o editor mudou antes do `apply`, devolver `StaleTransaction`.
- `apply` consome a transação; não permitir reutilização.
- `Applied` não carrega o documento inteiro; carrega `version`, `changes`, `selection`, `history`.
- `view(range?)` só materializa o que a UI pedir.
- Posições expostas em `u32`; conversões explícitas para UTF-16 no WASM.
- Erros estruturados: `{ code, message, path?, offset? }`, nunca string opaca.

### 3.4 Serialização da fronteira

Usar `serde`/`serde-wasm-bindgen` **apenas no crate WASM**, para produzir objetos JS tipados. O core continua `no_std`, sem `serde`. Os DTOs (`DocView`, `ChangeSet`, `Selection`, `HistoryInfo`, `SliceJSON`, `TableView`) são versionados e têm testes de contrato.

---

## 4. Loop de edição controlado sem colocar DOM no core

O core não deve escutar `beforeinput`, `compositionupdate` ou `paste`. Ele deve oferecer operações semânticas; a UI traduz eventos para elas.

### 4.1 Contrato de tradução

| Evento da UI | Chamada no core |
|---|---|
| `beforeinput: insertText` | `editor.insertText(data)` |
| `deleteContentBackward` | `editor.deleteBackward()` |
| `deleteContentForward` | `editor.deleteForward()` |
| seleção + delete | `editor.deleteSelection()` |
| `insertParagraph` | `editor.splitBlock()` |
| `insertLineBreak` | `editor.insertHardBreak()` |
| `formatBold` | `editor.toggleMark("strong")` |
| `formatItalic` | `editor.toggleMark("em")` |
| `compositionstart` | `editor.beginComposition(range)` |
| `compositionupdate` | `editor.updateComposition(text)` |
| `compositionend` | `editor.commitComposition()` |
| `paste` | `editor.insertSlice(selection.start, slice, "paste")` |
| `cut` | `editor.slice(...)` + delete |
| `undo`/`redo` | `editor.undo()` / `editor.redo()` |
| tabela | comandos de `Transaction` |

A UI continua convertendo seleção DOM para posição; o core fornece as funções de conversão e a álgebra.

### 4.2 Posições e Unicode

Expor no WASM:

- `pos_to_utf16(doc, pos)`
- `utf16_to_pos(doc, block, utf16)`
- `block_at(pos)` / `block_range(index)`
- `snap_to_scalar(doc, pos)`
- `snap_to_grapheme(doc, pos)` — opcional via feature/crate `unicode`, para emoji/combining marks
- `run_at(pos)` / `inline_at(pos)`

O core garante fronteira de escalar; a UI pode pedir grafema. RTL/bidi permanece responsabilidade visual da UI; as posições são lógicas.

### 4.3 Reconciliação

O contrato é:

1. UI aplica a operação.
2. Recebe `Applied.changes` com blocos/ranges.
3. Consulta `view(range)` ou `block(index)`.
4. Reconstrói apenas o DOM afetado.
5. Aplica a `Selection` retornada.

O DOM nunca é fonte de verdade. Se a UI deixar o browser mutar `contenteditable` sem aplicar no core, o documento diverge — o contrato deve proibir isso.

---

## 5. Histórico e composição

O core já tem `History`, `Origin`, `GroupId` e `CompositionEvent`, mas o WASM perde tudo a cada chamada e o agrupamento não está usando `GroupId`.

### 5.1 Histórico

- `Editor` mantém o `History` por sessão.
- `HistoryInfo`:
  ```ts
  interface HistoryInfo {
    canUndo: boolean;
    canRedo: boolean;
    undoDepth: number;
    redoDepth: number;
    bytes: number;
    lastOrigin: Origin | null;
  }
  ```
- `Transaction.setGroup(id)` + `History::record` deve **mesclar** grupos consecutivos com o mesmo `GroupId`.
- `setAddToHistory(false)` para operações auxiliares.
- `Origin` (`User`, `Paste`, `Composition`, `Api`, `Remote`) exposto como string.
- `undo`/`redo` retornam `Applied`, restauram `selection_before`/`selection_after` e atualizam `ChangeSet`.
- `clearHistory` e limites (`maxGroups`, `maxBytes`) configuráveis no `Editor`.
- Histórico não é serializado no documento; é sessão.

### 5.2 Composição

O modelo atual não restaura o texto no `Cancel` e não agrupa os updates como uma entrada.

Fluxo correto:

1. `beginComposition(range)` guarda snapshot do range e texto original.
2. `updateComposition(text)` aplica substituições com `add_to_history = false`.
3. `commitComposition()` grava **uma** entrada de undo com o range original → texto final.
4. `cancelComposition()` restaura o texto original e não grava histórico.
5. Selection fica no fim do texto composto; stored marks não são alterados.

---

## 6. Tabelas

Tabela é o ponto mais caro. O schema já existe no benchmark, mas não há comandos nem validação de grade.

### 6.1 Modelo

```text
table
  table_caption?
  table_row+
    table_header | table_cell
      block+
```

Atributos de célula: `colspan`, `rowspan`, `scope`, `align`, `abbr`, `headers`.
Atributos de tabela: `summary`, `col_widths` (array de inteiros).
Cabeçalho é o tipo do nó (`table_header`), não um booleano solto.

### 6.2 Invariantes

- `colspan >= 1`, `rowspan >= 1`.
- A grade efetiva de cada linha deve ter a mesma largura.
- `rowspan` não pode ultrapassar a última linha.
- Células não podem se sobrepor.
- Cabeçalhos devem ter `scope` válido quando aplicável.
- `headers` referencia IDs existentes.
- Tabela sem linhas é inválida.

### 6.3 Comandos

- criar tabela;
- inserir/remover linha;
- inserir/remover coluna;
- merge/split de células;
- alternar header;
- definir `colspan`/`rowspan`;
- definir `scope`/`align`;
- definir caption/summary;
- copiar/colar intervalo de células.

Para a primeira versão, os comandos estruturais podem substituir o bloco `table` inteiro via `Step::Replace` com `Slice::from_fragment`, preservando undo. O importante é a API e a validação; otimização por célula vem depois.

### 6.4 Seleção de tabela

Adicionar `Selection::Table` com `anchor`/`head` em `CellRef`. Sem isso, “apagar coluna” e “mesclar seleção” não têm representação correta.

---

## 7. Clipboard, block type e marcas

### 7.1 Clipboard

Expor `Slice` como dado:

```ts
interface SliceJSON {
  text: string;
  runs?: RunJSON[];
  blocks?: BlockJSON[];
  openStart: number;
  openEnd: number;
}
```

- `Doc::slice(range) -> Slice` público.
- `sliceToJSON` / `sliceFromJSON`.
- `insertSlice` valida contra o schema.
- `origin: "paste"` no histórico.
- UI lê `text/html`, TSV ou texto puro; o core recebe apenas o slice canônico ou texto.
- Para tabela, um `TableSlice` pode ser um `Slice` com blocos `table`/`table_row`/células.

### 7.2 Block type e atributos

Adicionar `Transaction::set_block_type`, que troca o tipo do bloco mantendo conteúdo compatível, e `set_node_attr` por nome. Listas usam `start`, `tight`, `type` via atributos; imagens usam `set_inline_attr`.

### 7.3 Mark attrs

Adicionar `SetMarkAttr` para alterar `href`/`title` de link existente sem remover/readicionar a marca.

---

## 8. Ordem de implementação

### M0 — Contrato e migração

- Congelar `DocView`, `Selection`, `ChangeSet`, `Applied`, `HistoryInfo`, `SliceJSON`, `TableView`.
- Definir `Schema::full()` e a lista de tipos/atributos.
- Definir versão do envelope e estratégia v1 → v2.
- Atualizar `DIVERGENCES.md` e o plano.

**Aceite:** tipos documentados; nenhum DTO depende de `AttrId`.

### M1 — Dados no core

- `Schema::full()` com todos os nós/marcas/atributos.
- `AttrType::{Enum,List,Map}`, `AttrValue::Map`.
- `SetInlineAttr` e `SetMarkAttr`.
- `DocView`/`View` com nomes de atributos.
- Remover schema básico público; migração privada.
- Atualizar bench `all_types` para `Schema::full()`.

**Aceite:** corpus all-types completo; image width/height, list start, metadata e table attrs aparecem no `DocView`; round-trip canônico.

### M2 — WASM stateful + histórico

- `Editor`/`Transaction`/`Applied` no crate WASM.
- `EditorState` persistente; sem reparse por chamada.
- `undo`/`redo`, `HistoryInfo`, `GroupId`, `Origin`, `clearHistory`.
- `ChangeSet` real e `version`.
- `Selection`/posições/UTF-16.
- Composição com commit/cancel corretos.
- Erros estruturados e `StaleTransaction`.

**Aceite:** JS executa múltiplas edições, undo/redo, composição e recebe change sets sem enviar o documento inteiro.

### M3 — Estrutura e clipboard

- `set_block_type`, `set_node_attr`, `set_inline_attr`, `set_mark_attr`.
- `slice`/`insertSlice`.
- Listas ordenadas com `start`.
- Imagens com dimensões.
- Block quote, code block, separator, figure.

**Aceite:** todos os tipos têm comandos de edição e atributos representáveis.

### M4 — Tabelas

- `TableInfo`/`CellRef`/`Selection::Table`.
- Comandos de linha/coluna/merge/split/header/span.
- Validação de grade.
- Clipboard de tabela.

**Aceite:** tabela editável apenas via core; undo restaura estrutura; A11y de headers.

### M5 — Migração, testes e performance

- Migração de documentos básicos.
- Atualizar TS/JSR, exemplos, CLI e PHP (sem schema custom).
- Testes de contrato, fuzz, wasm, benchmarks.
- Remover `basicSchema`/`documentSchema` e código morto.

---

## 9. Critérios de aceite finais

O editor é “verdadeiramente integrável” quando:

1. Um cliente JS consegue criar `Editor`, editar, selecionar, aplicar IME, copiar/colar, undo/redo e tabela sem reimplementar semântica.
2. Nenhum caminho público usa schema customizado ou `Schema::basic()`.
3. `DocView` contém tudo que um renderer final precisa: imagem com dimensões e `src`, listas com `start`, metadata, atributos de tabela, IDs, marks com atributos.
4. `ChangeSet` permite atualizar blocos específicos; `Applied` não devolve o documento inteiro.
5. Histórico persiste entre chamadas WASM; composição é uma única entrada; cancel restaura o original.
6. A UI não precisa conhecer `AttrId`, steps ou JSON interno; só os DTOs e comandos.
7. O core continua `no_std`, sem DOM, sem CSS e sem dependência de framework.

O risco maior é tabela (merge/split, spans, seleção), seguido de migração do schema/fingerprint e das sutilezas de histórico/composição. O iframe não entra neste plano — ele é apenas um consumidor do `DocView`/`ChangeSet` quando existir.
