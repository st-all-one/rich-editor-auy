# Importação em lote e recebimento de tokens

> **Estado:** implementado no core, no WASM e no TypeScript. `Editor.import`/`Editor.dryRun`
> recebem documento, fragmento ou lote de operações; documento e fragmento viram um único
> `ReplaceDoc` com `Origin::Import`. As lacunas reais estão em §7.

## 1. Escopo

Word, CSV, PDF e APIs externas **não entram no core**. O conversor de cada formato é do
consumidor/adaptador. O AUY recebe **tokens canônicos** já convertidos e oferece:

- validação contra o schema completo;
- aplicação atômica em relação ao estado visível (working copy descartado em erro);
- um grupo de undo por ingestão (`GroupId` derivado da versão, `Origin::Import`);
- `ImportReport` com problemas, `idMap`, versão e estado final;
- política explícita para IDs, limites e recursos (imagens/anexos).

O batch não é um parser de formato. É a **fronteira de ingestão de dados**.

`source`/`destination` são rótulos livres ecoados no relatório; o histórico registra apenas
`Origin::Import`, sem carregar o nome da origem.

### 1.1 O AUY recebe estrutura; o parser é do consumidor

Word, PDF, Excel e CSV têm semânticas próprias e, muitas vezes, tratamento especial. O AUY não
deve tentar interpretar XML do Word, layout de PDF, fórmulas de Excel ou dialeto de CSV. Ele deve
receber um **grafo estrutural canônico**:

- blocos, runs, marks, átomos e tabelas;
- atributos tipados e metadata;
- referências de asset para imagens/anexos;
- IDs e referências opcionais;
- rótulos de origem/destino (`source`/`destination`), usados só no relatório.

O conversor pode fazer qualquer tratamento especial antes de chamar o AUY: mapear estilos,
resolver campos, achatar tracked changes, converter planilhas em tabelas, escolher ordem de leitura
de PDF, fazer OCR, etc. O AUY não vê nada disso.

**Limite honesto:** “receber todos os dados estruturalmente” significa todos os dados que o
schema completo consegue representar. O conversor deve mapear para um tipo conhecido antes de
chamar o AUY. Se não existe tipo semântico, há duas opções fora do AUY:

1. adicionar um tipo/atributo ao schema completo, versionado; ou
2. representar como `raw_code`/`image` quando a semântica permitir.

Token desconhecido **não entra**. A ingestão falha com `ProblemDetails` e `suggestion`. O que não
pode existir é XML/HTML bruto ou JSON arbitrário sem validação. Isso quebraria determinismo,
limites e segurança.

---

## 2. Formas de entrada

Um único envelope (campos desconhecidos no envelope são ignorados; o que é estrito são os
conteúdos: token, atributo, mark e nome de `op` desconhecidos falham):

```ts
interface ImportEnvelope {
  mode: "document" | "fragment" | "replace" | "insert" | "ops";
  schema?: string;              // fingerprint; se presente, precisa bater com o editor
  doc?: DocumentJSON;           // mode: "document"
  fragment?: FragmentJSON;      // mode: "fragment" | "replace" | "insert"
  at?: number;                  // mode: "insert"; "fragment" com at também insere
  from?: number;                // modes: "replace" (e "fragment" sem at)
  to?: number;
  baseVersion?: number;         // mode: "ops"
  ops?: BatchOp[];              // mode: "ops"
  options?: ImportOptions;
  requestId?: string;           // idempotência externa; o core só ecoa
  source?: string;              // rótulo livre de auditoria
  destination?: string;         // rótulo livre de auditoria
}
```

### 2.1 Documento completo

```ts
{ mode: "document", schema?: string, doc: DocumentJSON, options?, source?, destination? }
```

Uso: “importar do Word” criando/substituindo a publicação inteira. O documento decodificado passa
por `Doc::validate` e substitui o estado atual em uma transação `ReplaceDoc`.

### 2.2 Fragmento

```ts
{ mode: "replace", schema?, from, to, fragment: FragmentJSON, options? }
{ mode: "insert", schema?, at, fragment: FragmentJSON, options? }
{ mode: "fragment", schema?, at?, from?, to?, fragment: FragmentJSON, options? }
```

`fragment` é o modo genérico: com `at` insere, sem `at` exige `from`/`to` e substitui. Uso:
inserir uma tabela CSV no cursor, colar uma seção do Word, substituir um bloco.

### 2.3 Operações

```ts
interface BatchPayload {
  ops: BatchOp[];
  baseVersion?: number;         // detecta edição concorrente; use sempre que houver
  origin?: string;              // rótulo de origem do histórico
  group?: number;               // grupo explícito de undo
  addToHistory?: boolean;       // default true
}

type BatchOp =
  | { op: "insert_text"; at: number; text: string }
  | { op: "delete_range"; from: number; to: number }
  | { op: "split_block"; at: number }
  | { op: "join_backward"; at: number }
  | { op: "toggle_mark"; from: number; to: number; mark: string; active?: boolean }
  | { op: "set_block_attr"; at: number; name: string; value: AttrValue }
  | { op: "set_doc_attr"; name: string; value: AttrValue }
  | { op: "set_inline_attr"; from: number; to: number; name: string; value: AttrValue }
  | { op: "set_mark_attr"; from: number; to: number; mark: string; name: string; value: AttrValue }
  | { op: "set_block_type"; at: number; kind: string; attrs?: AttrMap }
  | { op: "set_node_id"; at: number; id: string | null }
  | { op: "replace_document"; doc: DocumentJSON }
  | { op: "replace_fragment"; at?: number; from?: number; to?: number; fragment: FragmentJSON }
  | { op: "replace_slice"; from: number; to: number; slice: SliceJSON }
  | { op: "set_selection"; selection: Selection };
```

Os nomes camelCase (`insertText`, `setBlockAttr`, `replaceFragment`, …) são aceitos como alias.
Tabelas entram por `replace_fragment`/`replace_slice`; não existe op dedicada de tabela.

Uso: API estruturada, migração, importação incremental, composição de várias fontes.

---

## 3. Tokens canônicos

O wire aceita exatamente o JSON canônico de documento; um fragmento é o mesmo bloco sem o wrapper
de doc:

```ts
interface FragmentJSON {
  blocks: BlockToken[];
}

interface DocJSON {
  type: "doc";
  attrs?: AttrMap;
  content: BlockToken[];
}

interface BlockToken {
  type: string;                          // "paragraph", "heading", "table", ...
  attrs?: AttrMap;                       // inclui "id" (âncora, AttrId 0)
  content?: Array<BlockToken | TextToken | AtomToken>;
}

interface TextToken {
  type: "text";
  text: string;
  marks?: MarkToken[];
}

interface AtomToken {
  type: string;                          // "image", "hard_break", "video", ...
  attrs?: AttrMap;
}

interface MarkToken {
  type: string;                          // "strong", "link", "ref", ...
  // atributos do mark são planos: {"type":"link","href":"..."}
}
```

Regras de forma:

- bloco de texto: `content` é o array inline (`TextToken`/`AtomToken`), na ordem do texto;
- bloco contêiner: `content` é um array de blocos;
- `image_block` é átomo de bloco: só `attrs`, sem `content`; `separator` é folha sem texto
  (`content` ausente ou `[]`); `hard_break` é átomo inline dentro do array de texto;
- âncora de nó vive em `attrs.id` (attr reservado); `DocId` nunca entra no `Doc`;
- marks são emitidos com atributos **planos** dentro do objeto do mark; o decoder também aceita
  `"attrs"` aninhado para compatibilidade, mas a saída canônica é plana;
- `Slice` tem JSON próprio: estrutural `{"blocks":[...],"openStart"?,"openEnd"?}`, inline
  `{"text":"...","runs":[...]}` (`runs` ausente quando não há marcas/átomos).

Isso cobre parágrafos, títulos, listas, tabelas, imagens, links, referências, marks e metadata.
`wire::from_fragment_json` e `wire::from_slice_json` decodificam esses dois formatos; não há um
terceiro parser de tokens.

Não existe `embed`. Todo token precisa ser um tipo, mark, átomo ou atributo conhecido do schema
completo. Se o conversor não consegue mapear, ele não chama o AUY: corrige, achata ou descarta
antes. A ingestão desconhecida falha atomicamente com `ProblemDetails`.

`raw_code` é a representação canônica para payload bruto (XML, HTML, JSON, SQL, código): bloco
`text*`, sem marks, com `language`/`format`/`filename` e role a11y de código. Nunca é interpretado
como HTML.

### 3.1 Determinismo

O mesmo `payload` + `baseVersion` + `options` deve produzir o mesmo documento canônico e os mesmos
bytes:

- ordem de atributos é a ordem do schema, não a ordem do JSON de entrada;
- chaves de `Map` são ordenadas lexicalmente;
- marks saem na ordem do schema e duplicatas de mark falham (`duplicate_mark`);
- runs são **canonicalizados**, não preservados como vieram: runs adjacentes com o mesmo `MarkSet`
  são fundidos e um run que excede `MAX_RUN_BYTES` (4 KiB) é dividido de forma determinística;
- IDs preservados ou gerados de forma determinística; `regenerate` exige `idSeed` e deriva de
  `NodeId::derived(seed, path)`, onde `path` é o caminho de índices de bloco — nunca RNG/tempo;
- relatório usa paths estáveis e compactos: `ParseError.path` é numérico pontuado (`"3.1"`,
  `"doc.attrs"`); a reescrita de IDs usa `blocks[3][1]`;
- a validação é estrita: token desconhecido falha, sem fallback silencioso;
- nenhum relógio, RNG ou `HashMap` no core.

---

## 4. API de batch

```ts
class Editor {
  import(payload: ImportEnvelope | string): ImportReport;
  dryRun(payload: BatchPayload | string): ImportReport;
}
```

- `import` não lança para payload semanticamente inválido: devolve `ImportReport` com
  `accepted: false` e `problems`. JSON malformado também vira relatório (`malformed_batch`).
- `dryRun` aceita documento, fragmento e lote `{ "ops": [...] }`, valida em uma working copy e
  devolve o mesmo relatório sem tocar no editor; `state.version` continua a versão atual.
  Envelope malformado ou falha de precheck (schema/`baseVersion`/`maxBytes`) lança `ProblemDetails`.

```ts
interface ImportOptions {
  idPolicy?: "preserve" | "regenerate" | "reject";  // default "preserve"
  idSeed?: string;                                   // obrigatório com regenerate
  addToHistory?: boolean;                            // default true
  limits?: BatchLimits;                              // só aperta, nunca afrouxa
}

interface ImportReport {
  accepted: boolean;
  blocks: number;             // blocos de topo após a tentativa
  acceptedBlocks: number;     // blocos aceitos do payload
  version: number;
  schema: string;             // fingerprint do schema do editor
  requestId: string | null;
  source: string | null;
  destination: string | null;
  idMap: { from: string; to: string }[];
  problems: ProblemDetails[];
  state: EditorStateJSON;     // documento + seleção resultantes
}
```

O relatório carrega o estado final; `ChangeSet` por bloco continua disponível nos caminhos de
edição (`apply`), não no import.

### Atomicidade e erros

- A ingestão é **estrita**: qualquer token desconhecido, atributo inválido, mark inválido, ID
  duplicado ou estrutura sem representação rejeita o batch inteiro; nada muda.
- Documento/fragmento são decodificados e montados em uma **working copy**; em erro ela é
  descartada e o estado visível permanece intacto.
- Documento/fragmento viram **uma** transação `ReplaceDoc` com `Origin::Import` e
  `GroupId(version + 1)`; um único undo restaura o snapshot anterior.
- `ops` aplica cada operação como transação própria sobre a mesma working copy, compartilhando um
  `GroupId(version + 1)`; em erro, a working copy é descartada. `options.addToHistory: false`
  mantém a ingestão fora do histórico nos três modos; `group` explícito continua exclusivo de
  `Editor.apply(BatchPayload)`.
- `baseVersion` é verificado no modo `ops`; divergência → `stale_batch`.
- Fingerprint de `schema` presente e diferente do editor → `schema_mismatch`.
- Não há fallback silencioso, `dropped` ou `best_effort` dentro do AUY; essas decisões são do
  conversor.

```json
{
  "type": "https://rich-editor-auy.dev/problems/unknown-token",
  "title": "Unknown AUY token",
  "status": 422,
  "detail": "The token 'chart' has no representation in the full schema.",
  "instance": "3",
  "code": "unknown_token",
  "suggestion": "Render the chart as an image with alt text, or omit it before import."
}
```

Códigos emitidos hoje no caminho de importação: `unknown_token`, `unknown_mark`, `unknown_attr`,
`invalid_attr`, `unsafe_url`, `duplicate_id`, `stale_batch`, `schema_mismatch`, `limit_exceeded`,
`id_rejected`, `invalid_id_policy`, `malformed_batch`, `invalid_mode`, além dos códigos de
parse/step herdados. `unrepresentable_feature` existe no core, mas o decoder atual reporta token
sem representação como `unknown_token`; o código fica reservado para quando houver um caminho que
distinga os dois casos.

### Conversão recomendada

| Origem | Destino AUY | Observação |
|---|---|---|
| gráfico | `image`/`image_block` | `asset_id`, `alt`, `caption`; renderizar antes |
| fórmula | `image` | snapshot renderizado; `alt` com a fórmula |
| quebra de página | **erro** | remover no conversor; `separator` só se a semântica aceitar |
| XML/HTML/código bruto | `raw_code` | `language`/`format`; texto escapado, nunca executado |
| text box | `blockquote`/`paragraph` | achatar na ordem de leitura |
| comentário | **erro** | remover ou tratar fora do documento |
| tracked change | **erro** | aceitar/rejeitar antes |
| nota de rodapé | **erro** até schema próprio | mover para corpo ou `raw_code` |
| desconhecido | **erro** | `unknown_token` + `suggestion` |

### Limites

```ts
interface BatchLimits {
  maxBytes?: number;          // bytes do payload bruto, antes do decode
  maxBlocks?: number;         // blocos de topo após a importação
  maxDepth?: number;          // aperta o limite do decoder
  maxNodes?: number;          // aperta o limite do decoder
  maxRunsPerBlock?: number;   // aperta o limite do decoder
  maxOps?: number;            // operações em um único modo "ops"
  maxTableCells?: number;     // células de tabela no documento resultante
}
```

Os três últimos só **reduzem** o limite padrão (`min` com o default); nunca o aumentam. Erros de
limite são `ProblemDetails` com `code: "limit_exceeded"` e `instance` nomeando o limite
(`maxBytes`, `maxBlocks`, ...); não são truncamento.

---

## 5. IDs, referências e recursos

### IDs

- `preserve` (default): mantém a âncora `attrs.id`; duplicata é rejeitada pela validação do
  documento (`I16`) e vira `duplicate_id`.
- `regenerate`: substitui a âncora de cada bloco por `NodeId::derived(seed, path)`, devolve
  `idMap` como pares `{from, to}` e reescreve todo `RefTarget.node` (em atributos de bloco, átomo
  inline e mark) que apontava para o ID antigo, em duas passadas — referências para frente incluídas.
- `reject`: qualquer âncora no payload falha com `id_rejected`.

`DocId` não entra no `Doc`; é envelope/publicação. O `RefTarget.doc` é preservado: o import só
remapeia o `node`, porque a publicação de destino é do consumidor.

### Imagens e anexos

Word/PDF trazem bytes; o AUY não deve carregar binário no documento.

Nós de mídia do schema completo (`image`, `image_block`, `video`, `audio`, `attachment`) declaram:

- `asset_id` (referência opaca ao storage do consumidor);
- `src` (URL final, quando já resolvida), `poster`/`srcset`/`sizes` quando aplicáveis;
- `mime`, `size_bytes`, `width`, `height`, `intrinsic_width`, `intrinsic_height`, `alt`;
- `caption`/`credit`/`transcript`/`filename` conforme o tipo.

O batch pode inserir `asset_id` + placeholder; upload/resolução é do consumidor. O core valida
`alt` obrigatório e dimensões; `data:`/`javascript:` em `src`/`href`/`poster`/`canonical_url`
falham com `unsafe_url`, e nada é interpretado como HTML.

---

## 6. Cenários

### Word

O conversor externo mapeia estilos para blocos/marks e emite tokens. O schema completo cobre:

- parágrafos, `heading`, listas (`start`, `reversed`, `type`, `tight`, `checked`), `blockquote`,
  `code_block`, `raw_code`;
- tabelas com `table_caption`, `table_row`, `table_header`/`table_cell`, `colspan`/`rowspan`,
  `col_widths`, `summary`;
- `figure`/`caption`, `image`/`image_block`/`video`/`audio`/`attachment` com `alt`, dimensões,
  `asset_id`;
- links, refs, marks (`strong`, `em`, `code`, `underline`, `strike`, `highlight`, `sup`, `sub`,
  `link`, `ref`);
- metadata do documento (`title`, `description`, `authors`, `tags`, `metadata`, `lang`, `dir`).

Features que **não** estão no schema atual e precisam de uma decisão do conversor:

- headers/footers;
- notas de rodapé/fim;
- comentários;
- tracked changes;
- text boxes/formas flutuantes;
- quebras de página/seção;
- TOC/campos;
- equações, gráficos e objetos embutidos;
- cores/fontes arbitrárias (o AUY é semântico, não CSS).

O conversor pode: mapear para um tipo existente, achatar, usar `raw_code`/`image` quando couber ou
descartar antes de chamar o AUY. Um token desses sem representação **não é enviado**; se for, o
AUY devolve `ProblemDetails` com `unknown_token` e `suggestion`. Se o produto exige
comentários/tracked changes como dados editáveis, isso vira tipo de schema novo, não lógica de
parser no AUY.

### CSV e Excel

CSV e Excel viram uma tabela canônica:

- primeira linha como `table_header` quando configurado;
- células com texto, quebras e aspas já decodificadas pelo conversor;
- sem marks ricos;
- `colspan`/`rowspan` só quando o conversor os extrair (Excel);
- fórmulas/formatos de número como atributos ou metadata, não como engine de cálculo;
- múltiplas planilhas como múltiplas tabelas/seções, decisão do conversor;
- limites de células/bytes;
- batch `replace_fragment`/`insert` de uma tabela.

O ponto crítico é o schema completo de tabela e a validação de grade, não o CSV/Excel em si.

### PDF

O conversor emite parágrafos, headings, listas, tabelas e imagens na ordem de leitura. O AUY não
modela páginas, colunas ou cabeçalhos/rodapés de página; se o produto precisar, isso é schema novo
(`page_break`, `section`, `header/footer`) ou metadado de origem. Sem isso, o PDF é achatado para
fluxo de blocos.

### API JSON

A API pode enviar:

- `DocumentJSON` completo; ou
- `FragmentJSON` para inserir/substituir; ou
- `ops` para alterações pontuais.

Tudo valida contra o schema completo e fingerprint. JSON desconhecido não é aceito por `Any`;
atributo, mark ou token desconhecido falha com `ProblemDetails`, sem fallback.

---

## 7. Estado da implementação

Resolvido desde a primeira versão deste plano:

| Área | Implementação | Evidência |
|---|---|---|
| Schema completo | `Schema::full` é o único preset usado por WASM/TS e pela ingestão | `tests/schema_full*.rs`, `tests/tables.rs` |
| Fragmento/slice | `wire::from_fragment_json`, `wire::from_slice_json`, `Slice::to_json_*` | `tests/fragment_json.rs`, `tests/clipboard.rs`, `tests/slice_open.rs` |
| Cópia | `Doc::slice`/`insert_slice`/`replace_slice` com bordas abertas | `tests/slice_open.rs` |
| Batch atômico | `Transaction::replace_document` + `BatchPayload`/`Op` no WASM | `wasm/src/tests.rs` |
| Attr inline/mark | `SetInlineAttr`/`SetMarkAttr` (steps, `Transaction`, comandos) | `tests/step_mark_attrs.rs`, `tests/attributes.rs` |
| Tabela/lista | tabela com caption/spans, `ordered_list.start` | `tests/tables.rs`, `tests/schema_full*.rs` |
| Metadata | `title`/`description`/`authors`/`tags`/`metadata` + `AttrType::{List,Map}` | `tests/schema_full_attrs.rs` |
| Mídia | `asset_id`, `mime`, `size_bytes`, dimensões, `alt` obrigatório | `tests/schema_full_attrs.rs` |
| Erros/relatório | `ProblemDetails` RFC 9457 + `ImportReport` com `state`/`problems`/`idMap` | `wasm/src/problem.rs`, `wasm/src/convert.rs` |
| Histórico | `Origin::Import` + `GroupId` único por ingestão | `state/history.rs`, `wasm/src/import.rs` |
| ChangeSet | `insertedBlocks`/`removedBlocks` por bloco | `tests/changes.rs` |
| IDs | `preserve`/`regenerate`/`reject`, `idSeed`, `idMap` | `wasm/src/ids.rs` |
| Limites | `maxBytes`/`maxBlocks`/`maxDepth`/`maxNodes`/`maxRunsPerBlock` | `wasm/src/dto/import.rs` |
| `baseVersion` | `stale_batch` no modo `ops` | `wasm/src/editor/import.rs` |
| `raw_code` | bloco textual com `language`/`format`/`filename` | `schema/full/nodes.rs` |

Fechadas neste ciclo:

| # | Antes | Agora | Evidência |
|---|---|---|---|
| 1 | `dryRun` validava só `ops` | valida documento, fragmento e ops em working copy, sem mutar, com o mesmo `ImportReport` | `wasm/src/editor/import.rs`; `mod_test.ts` |
| 2 | `regenerate` reescrevia só âncoras de bloco | duas passadas: âncoras + todo `RefTarget.node` (bloco, átomo, mark), inclusive refs para frente | `wasm/src/ids.rs`; `regenerate_rewrites_references_to_imported_anchors` |
| 3 | `ImportOptions` sem `addToHistory` | `addToHistory: false` vale para documento, fragmento e ops | `wasm/src/import.rs`; `mod_test.ts` |
| 4 | `BatchLimits` sem `maxOps`/`maxTableCells` | `maxOps` antes de aplicar; `maxTableCells` conta células aninhadas do documento resultante | `wasm/src/import.rs`; testes nativos e `mod_test.ts` |

Estruturas sem tipo semântico (Word/PDF/Excel) continuam sendo decisão do conversor: tipo novo,
`raw_code`/`image` ou erro **antes** do envio. Isso não é lacuna do AUY.

---

## 8. Ordem

1. ~~Schema completo + `FragmentJSON` + `Slice` público.~~
2. ~~`SetInlineAttr`/`SetMarkAttr` + metadata + mídia.~~
3. ~~`Editor.import`/`dryRun` + `ImportReport` + `Origin::Import`.~~
4. ~~IDs, limites, `baseVersion`.~~
5. Conversor Word/PDF/Excel/CSV: permanece fora do repositório; a tabela canônica está coberta.
6. ~~Fuzz de fragmento/slice~~ (`fuzz/fuzz_targets/fragment.rs`); desempenho do lote segue coberto
   pelos benches de `apply`/`replace_document` em `bench/hot.rs`.

O batch foi implementado **depois** do schema completo e do `Editor` stateful, porque depende de
ambos. A conversão Word/PDF/Excel/CSV permanece fora do projeto; o AUY garante que os tokens
gerados sejam representáveis, validados, determinísticos e aplicados atomicamente.
