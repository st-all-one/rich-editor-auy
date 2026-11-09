# Importação em lote e recebimento de tokens

## 1. Escopo

Word, CSV, PDF e APIs externas **não entram no core**. O conversor de cada formato é do
consumidor/adaptador. O AUY recebe **tokens canônicos** já convertidos e oferece:

- validação contra o schema completo;
- aplicação atômica em uma única transação;
- um único grupo de undo;
- `ChangeSet`, seleção final e relatório de importação;
- política explícita para IDs, limites e recursos (imagens/anexos).

O batch não é um parser de formato. É a **fronteira de ingestão de dados**.

### 1.1 O AUY recebe estrutura; o parser é do consumidor

Word, PDF, Excel e CSV têm semânticas próprias e, muitas vezes, tratamento especial. O AUY não
deve tentar interpretar XML do Word, layout de PDF, fórmulas de Excel ou dialeto de CSV. Ele deve
receber um **grafo estrutural canônico**:

- blocos, runs, marks, átomos e tabelas;
- atributos tipados e metadata;
- referências de asset para imagens/anexos;
- IDs e referências opcionais;
- uma origem declarada (`word`, `pdf`, `excel`, `csv`, `api`), usada só para relatório/undo.

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

O mesmo contrato deve aceitar três granularidades:

### 2.1 Documento completo

```ts
interface ImportDocument {
  mode: "document";
  schema: string;              // fingerprint do schema completo
  doc: DocumentJSON;           // Doc canônico
  options?: ImportOptions;
}
```

Uso: “importar do Word” criando/substituindo a publicação inteira.

### 2.2 Fragmento

```ts
interface ImportFragment {
  mode: "replace" | "insert";
  schema: string;
  at?: number;                 // insert
  from?: number;               // replace
  to?: number;                 // replace
  fragment: FragmentJSON;      // blocos sem wrapper de doc
  options?: ImportOptions;
}
```

Uso: inserir uma tabela CSV no cursor, colar uma seção do Word, substituir um bloco.

### 2.3 Operações

```ts
type BatchOp =
  | { op: "replaceDocument"; doc: DocumentJSON }
  | { op: "replaceRange"; from: number; to: number; fragment: FragmentJSON }
  | { op: "insertFragment"; at: number; fragment: FragmentJSON }
  | { op: "insertTable"; at: number; table: TableJSON }
  | { op: "setDocAttr"; name: string; value: AttrValue }
  | { op: "setNodeAttr"; at: number; name: string; value: AttrValue }
  | { op: "setInlineAttr"; range: [number, number]; name: string; value: AttrValue }
  | { op: "setSelection"; selection: Selection }
  | { op: "setMetadata"; metadata: AttrMap };

interface ImportOps {
  mode: "ops";
  schema: string;
  baseVersion: number;
  ops: BatchOp[];
  options?: ImportOptions;
}
```

Uso: API estruturada, migração, importação incremental, composição de várias fontes.

---

## 3. Tokens canônicos

O mínimo para “recebimento completo”:

```ts
interface FragmentJSON {
  blocks: BlockToken[];
  openStart?: number;
  openEnd?: number;
}

interface BlockToken {
  kind: string;                // paragraph, heading, table, ...
  id?: string;                 // opcional; política de ID decide
  attrs?: AttrMap;
  runs?: RunToken[];           // textblock
  atoms?: AtomToken[];         // inline atoms
  children?: BlockToken[];     // container
}

interface RunToken {
  text: string;
  marks?: MarkToken[];
}

interface MarkToken {
  type: string;                // strong, em, link, ref...
  attrs?: AttrMap;             // href, target...
}

interface AtomToken {
  kind: string;                // image, hard_break...
  attrs?: AttrMap;
}

```

Isso cobre parágrafos, títulos, listas, tabelas, imagens, links, referências, marks e metadata.
O `FragmentJSON` precisa de parser/validator próprio; hoje o wire só conhece `Doc`.

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
- marks e runs preservam a ordem do array de entrada;
- IDs preservados ou gerados de forma determinística; `regenerate` exige `idSeed` e deriva do
  caminho estável (`seed + path + occurrence`), nunca de RNG/tempo;
- relatório usa paths estáveis (`blocks[3].children[1]`);
- a validação é estrita: token desconhecido falha, sem fallback silencioso;
- nenhum relógio, RNG ou `HashMap` no core.

---

## 4. API de batch

```ts
class Editor {
  import(batch: ImportDocument | ImportFragment | ImportOps): ImportResult;
  dryRun(batch: ImportDocument | ImportFragment | ImportOps): ImportReport;
}

interface ImportOptions {
  source?: "word" | "pdf" | "excel" | "csv" | "api" | string;
  idPolicy?: "preserve" | "regenerate" | "reject";
  idSeed?: string;             // obrigatório com regenerate
  addToHistory?: boolean;      // default true
  requestId?: string;          // idempotência externa; o core só ecoa
  limits?: BatchLimits;
}

interface ImportResult {
  applied: Applied;
  report: ImportReport;
}

interface ImportReport {
  ok: boolean;
  schema: string;
  acceptedBlocks: number;
  idMap: Record<string, string>;
  requestId?: string;
}
```

### Atomicidade e erros

- A ingestão é **estrita**: qualquer token desconhecido, atributo inválido, mark inválido, ID
  duplicado ou estrutura sem representação rejeita o batch inteiro; nada muda.
- O erro é `ProblemDetails` (RFC 9457) com `type`, `title`, `status`, `detail`, `instance`,
  `code` e `suggestion`.
- `dryRun` executa a mesma validação e retorna `ImportReport` ou `ProblemDetails`; não cria
  transação.
- `baseVersion` obrigatório para `ops`; se o editor mudou, `StaleBatch`.
- O batch inteiro é uma transação; um único `GroupId`; um único undo.
- `Origin::Import(source)` no histórico.
- `ChangeSet` final inclui `insertedBlocks`/`removedBlocks` e seleção.
- Não há fallback silencioso, `dropped` ou `best_effort` dentro do AUY; essas decisões são do
  conversor.

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

Códigos iniciais: `unknown_token`, `unknown_mark`, `unknown_attr`, `invalid_attr`,
`unrepresentable_feature`, `missing_asset`, `duplicate_id`, `stale_batch`, `limit_exceeded`.

### Conversão recomendada

| Origem | Destino AUY | Observação |
|---|---|---|
| gráfico | `image` | `asset_id`, `alt`, `caption`; renderizar antes |
| fórmula | `image` | snapshot renderizado; `alt` com a fórmula |
| quebra de página | **erro** | `unrepresentable_feature`; remover ou usar `separator` se aceitável |
| XML/HTML/código bruto | `raw_code` | `language`/`format`; texto escapado, nunca executado |
| text box | `blockquote`/`paragraph` | achatar na ordem de leitura |
| comentário | **erro** | remover ou tratar fora do documento |
| tracked change | **erro** | aceitar/rejeitar antes |
| nota de rodapé | **erro** até schema próprio | mover para corpo ou `raw_code` |
| desconhecido | **erro** | `unknown_token` + `suggestion` |

### Limites

```ts
interface BatchLimits {
  maxOps: number;
  maxBlocks: number;
  maxBytes: number;
  maxDepth: number;
  maxTableCells: number;
}
```

Erros de limite são `ImportLimitError` com código/path; não são truncamento.

---

## 5. IDs, referências e recursos

### IDs

- `preserve`: mantém `NodeId`/`id` se único; colisão é erro.
- `regenerate`: gera novos IDs para conteúdo importado e devolve `idMap`; referências internas
  ao fragmento são remapeadas.
- `reject`: qualquer ID duplicado falha.

`DocId` não entra no `Doc`; é envelope/publicação. Referências externas permanecem intactas.
Referências internas ao fragmento precisam ser remapeadas quando `idPolicy = regenerate`.

### Imagens e anexos

Word/PDF trazem bytes; o AUY não deve carregar binário no documento.

Adicionar aos nós de mídia:

- `asset_id` (referência opaca ao storage do consumidor);
- `src` (URL final, quando já resolvida);
- `mime`, `size_bytes`, `width`, `height`, `intrinsic_width`, `intrinsic_height`, `alt`.

O batch pode inserir `asset_id` + placeholder; upload/resolução é do consumidor. O core valida
`alt` obrigatório e dimensões; nunca aceita `data:` nem HTML bruto.

---

## 6. Cenários

### Word

O conversor externo mapeia estilos para blocos/marks e emite tokens. O AUY suporta o que estiver
no schema completo:

- parágrafos, headings, listas com `start`, blockquote, code block;
- tabelas com header, `colspan`/`rowspan`, caption;
- imagens com `alt`, dimensões, asset;
- links, refs e marks;
- metadata do documento.

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
AUY devolve `ProblemDetails` com `unrepresentable_feature`. Se o produto exige
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
- batch `insertTable` ou `replaceRange` de uma tabela.

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

## 7. O que ainda não está hábil

| # | Lacuna | Impacto | Correção |
|---|---|---|---|
| 1 | WASM usa `Schema::basic()` fixo | rejeita tabela, blockquote, code block, listas ordenadas e atributos | schema completo único no WASM |
| 2 | Não existe `FragmentJSON`/parser de fragmento | só dá para importar documento inteiro, não inserir no cursor | wire de fragmento + validação |
| 3 | Não existe `Doc::slice` público | copy/paste e import de seleção não têm fonte canônica | expor `Slice`/`Fragment` |
| 4 | `Transaction` não tem `replace_document`/batch | importação não é atômica nem uma entrada de undo | operação de batch no core/WASM |
| 5 | `Transaction` só tem `SetNodeAttr` de bloco | imagem/átomo não tem `width`, `height`, `alt` editáveis | `SetInlineAttr`/`SetMarkAttr` |
| 6 | Schema básico não tem tabela completa nem `ordered_list.start` | CSV/Word perdem estrutura | schema completo |
| 7 | Não há metadata de documento nem `AttrValue::Map`/`List` | Word/PDF/API perdem título, autor, tags, custom props | tipos de attr + `metadata` |
| 8 | Imagem não tem `asset_id`/dimensões | Word/PDF com imagens embutidas não representável | atributos de mídia + resource map |
| 9 | Não há `ProblemDetails`/`ImportReport` | token inválido falha de forma opaca ou é perdido | erro RFC 9457 + relatório |
| 10 | Não há `Origin::Import` nem batch group | undo mistura importação com edição | origin + GroupId + um undo |
| 11 | `ChangeSet.blocks` é grosseiro | UI não sabe o que atualizar após import | change set real + inserted/removed |
| 12 | Não há política de IDs | IDs do Word colidem ou refs quebram | preserve/regenerate/reject + idMap |
| 13 | Não há limites de batch | DOCX/PDF grande pode estourar memória | BatchLimits + erros |
| 14 | Estruturas sem tipo semântico (Word/PDF/Excel) | conversor não tem slot; token desconhecido não é permitido | tipo de schema novo, `raw_code`/`image` ou erro antes do envio |
| 15 | Não há `dryRun` | import inválido só falha depois de aplicar | validação prévia |
| 16 | Não há versão/base no batch | import sobrescreve edição concorrente | `baseVersion`/`StaleBatch` |
| 17 | Não há `raw_code` no schema completo | XML/HTML/código bruto não têm representação segura | `raw_code` textual com `language`/`format` e a11y |
| 18 | IDs regenerados sem seed | import não é determinístico entre execuções | `idSeed` + path estável |

---

## 8. Ordem

1. Schema completo + `FragmentJSON` + `Slice` público.
2. `SetInlineAttr`/`SetMarkAttr` + metadata + mídia.
3. `Editor.import`/`dryRun` + `ImportReport` + `Origin::Import`.
4. IDs, limites, `baseVersion`.
5. Tabela/CSV/Excel; depois Word/PDF no conversor externo.
6. Testes de importação, fuzz de fragmento, benchmarks de batch.

O batch deve ser implementado **depois** do schema completo e do `Editor` stateful, porque depende
de ambos. A conversão Word/PDF/Excel/CSV permanece fora do projeto; o AUY garante que os tokens
gerados sejam representáveis, validados, determinísticos e aplicados atomicamente.
