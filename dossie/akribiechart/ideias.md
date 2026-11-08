# akribie — ideias para o `rich-editor-auy`

> Priorizado por **valor de transferência conceitual** × **custo** × **risco de contaminar o core**.
> Cada ideia tem: hipótese, evidência no akribie (`caminho:linha`), onde encaixa no nosso desenho
> (`plan/04_arquitetura.md` e `plan/06_api-e-integracao.md`), custo/risco, e veredito
> **agora / pós-MVP / rejeitar**.
>
> Filtro aplicado: o akribie **não é um editor**. Nada de geometria, formas, fontes, dagre ou
> `.excalidraw` entra nesta lista. O que atravessa é o contrato: bytes canônicos, erro como dado,
> a11y derivada do modelo, diff semântico, e disciplina de guarda mecânica.

## 0. Resumo

| # | Ideia | Alvo | Veredito |
|---|---|---|---|
| 1 | `Diagnostic` com código estável + tabela única + paridade doc↔código | `error`, `wire`, CLI, MCP | **agora** |
| 2 | `validate(&Doc)` barato, separado da construção de view data | `state`/`audit` | **agora** |
| 3 | Forma canônica do wire com prova de idempotência | `wire` | **agora** |
| 4 | Envelope único de saída; erro de fronteira é *resultado* | CLI, MCP, `wire` | **agora** |
| 5 | Matriz de features em CI + guarda única de float→texto | `math`, CI | **agora** |
| 6 | `DocOutline`: descrição textual derivada do modelo | `wire`/`describe` + adaptador | **pós-MVP** (a `A11yTree` estrutural é F1/D58/D61) |
| 7 | Diff semântico por identidade de nó | crate de revisão/`audit` | **pós-MVP** |
| 8 | `ImportReport` obrigatório com degradação contada e round-trip testado | `-html`, `-markdown` | **pós-MVP** |
| 9 | Paleta fechada e tokens de cor dentro do core | — | **rejeitar** |
| 10 | Gramática textual linha-a-linha como formato de wire | — | **rejeitar** |
| 11 | Busca linear por id em estrutura quente | — | **rejeitar** |

---

## 1. `Diagnostic` com código estável, `hint` e tabela única — **agora**

### Hipótese

Um `enum EditError` sozinho não resolve o problema de diagnóstico. O consumidor (UI, agente, CLI,
MCP) precisa de quatro coisas que o enum não dá: **(a)** um identificador **estável** para
correlacionar com docs/suporte/tests, **(b)** uma **posição** no documento, **(c)** o que se
**esperava** e **(d)** como **corrigir**. Sem isso, a UI improvisa mensagens e o agente adivinha.

O segundo half da hipótese: a tabela de diagnósticos tem de ser **uma** `const` que alimenta
simultaneamente o validador, o `--json`, o comando `schema` e o `prime` — e um teste que falha
quando a documentação diverge.

### Evidência no akribie

- `crates/akribie-core/src/diag.rs:28-53` — `Code { code: &'static str, name, severity }`, com
  `err()`/`warn()` como `const fn`. A severidade é **intrínseca ao código**, não escolhida no call
  site (não existe `Diagnostic::warning(...)` que possa mentir).
- `diag.rs:55-91` — 16 variáveis `pub const` (`E001`…`W007`) e
  `pub const ALL: [Code; 16]`. A doc do módulo (`diag.rs:1-5`) declara que `ALL` alimenta
  `akr schema` e `akr prime`, "então código novo entra na tabela, no teste de paridade e na
  documentação".
- Os consumidores são literais: `crates/akribie-cli/src/commands/schema.rs:31` (`diag::ALL` → JSON),
  `crates/akribie-cli/src/commands/prime.rs:119` (`diag::ALL` → texto), e
  `crates/akribie-core/src/error.rs:29-35` faz o mesmo para `ErrorKind::ALL`.
- `diag.rs:134-159` — `render(file) -> String` (texto `arquivo:linha:col: CODE nome: msg` +
  `\n  hint: …`) e `to_json()` com exatamente `{code, severity, line, col, message, hint?}`.
  Um formato para humano, um para máquina, mesma fonte.
- `diag.rs:167-172` — `sort()` por `(line, col, code)`. Determinismo da **ordem dos erros**, não só
  dos bytes.
- `crates/akribie-cli/tests/docs.rs:1-3, 12-21, 32-55, 99-120` — o teste de paridade de verdade:
  `VERBS` mapeia cada um dos 8 verbos para o guia em `docs/usage/`; `flags()` extrai os `--flag` de
  um texto e `declared_verbs()` lê o `--help` do **binário real**; os testes comparam os dois
  conjuntos. Flag documentada que não existe (ou o contrário) quebra o CI. É a prova de que
  "documentação é entregável" aqui é gate, não slogan.

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| `error` (`plan/04_arquitetura.md:70`) | manter `EditError` como **causa tipada** para `?` e `match`, e acrescentar `Diagnostic` como **relatório**: `struct Diagnostic { code: DiagCode, span: Span, severity, message, hint }`. `EditError: Into<Diagnostic>` num único `impl`, para não haver conversão ad-hoc. |
| `DiagCode` | `pub struct DiagCode { pub code: &'static str, pub name: &'static str, pub severity: Severity }` + `pub const ALL: &[DiagCode]`. Sem `String`: o código é estático, senão não é estável. |
| `Span` | hoje a nossa `Pos` é offset de byte. Para diagnóstico precisamos de `Span { start: Pos, end: Pos }` **e** de uma forma de renderizar `linha:coluna` — que é uma função do adaptador (o core não conhece "linha visual", mas pode contar `\n` no texto do bloco). Decidir explicitamente: `Span` no core em `Pos`, renderização de linha/coluna num helper do core (`text::line_col`), não no adaptador, para o MCP e a CLI não divergirem. |
| `wire` | `Diagnostic` precisa de `to_json`/`from_json` canônicos (`plan/04_arquitetura.md:67`). Campo `code` é `&'static str` na origem e `String`/`Cow` na deserialização. |
| CLI / MCP | os dois consomem `ALL` para o comando `schema`/`tools/list` e serializam `Diagnostic` sem tradução própria. **Uma** tabela de exit codes, como `error.rs:1-4`, publicada pela mesma tabela. |
| CI | o teste de paridade do akribie é o modelo: um teste que lê `--help` (ou o `schema` do CLI) e compara com `docs/` — mais um que garante `ALL.len()` == `#[non_exhaustive]`? (não dá; em vez disso, teste que cada `DiagCode` aparece na doc). |

### Custo e risco

**Custo**: baixo. `DiagCode` + `Diagnostic` + `Into` são ~150 linhas; a tabela cresce com o projeto.
O `hint` exige disciplina de escrita, não código.
**Risco**: duplicação entre `EditError` e `Diagnostic` se ninguém definir a fronteira. Regra:
`EditError` é para **quem programa** (`?`, `match`); `Diagnostic` é para **quem consome** (UI, agente).
Um não substitui o outro, e todo `EditError` que chega à fronteira tem `Into<Diagnostic>`.
**Risco 2**: `&'static str` no código quebra `no_std`? Não — `&'static str` é `core`. Mas obriga
`ALL` a ser `&'static [DiagCode]`, o que é bom.

---

## 2. `validate(&Doc)` barato, separado da construção de view data — **agora**

### Hipótese

Se validar documento exige construir view data, ninguém valida a cada tecla. O custo de validação
precisa ser estruturalmente menor que o de renderizar — e isso é uma decisão de **separação de
estágios**, não de otimização.

### Evidência no akribie

- `crates/akribie-core/src/pipeline.rs:27-40` — `check()` é **seis chamadas puras**: `parse`,
  `resolve`, `isolated`, `missing_glyphs`, `check_contrast`, `sort`. Nenhuma delas chama `layout`,
  `paint` ou `emit`.
- Medido (§[`exemplos/README.md`](exemplos/README.md) §5.2, binário release, mediana de 7):
  1000 nós / 999 arestas → `check` **14,4 ms** vs `render` **80,8 ms**. Razão **5,6×**. O `check` não
  paga dagre nem pintura.
- Cada regra é uma função independente que devolve `Vec<Diagnostic>` (`pipeline.rs:43-64` `W002`,
  `:67-100` `W003`, `:33` `W007` via `a11y::check_contrast`). Testáveis isoladamente
  (`pipeline.rs:118-160`, 5 testes).

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| `state`/novo `audit` | `pub fn validate(doc: &Doc, schema: &Schema) -> Diagnostics` (coleção ordenada, `SmallVec`/`Vec`). **Não** retorna `DocView`, `ChangeSet` nem nada de layout. |
| `schema` | as regras que já existem (content match, marks permitidas, attrs) são **reutilizadas** aqui, não reimplementadas: `validate` é o *driver* que as chama e acumula. É o padrão do akribie: `check` não valida nada por conta própria, ele orquestra. |
| `view` | `view_data(&Doc, &Schema, &Viewport)` continua caro e separado. `Applied` carrega `changes` (que a UI usa), mas `validate` é chamável a qualquer momento sobre um `Doc` imutável — inclusive sobre um snapshot `Arc` em outra thread (`plan/04_arquitetura.md:11`). |
| contrato | `Diagnostics::is_empty()` é o que a UI usa para esconder o painel; `has_errors()` para bloquear exportação. Duas perguntas distintas, dois métodos (como `diag::has_errors`, `diag.rs:163-165`). |
| teste | um teste de **razão**: `validate` não pode ser mais caro que `view_data` num corpus fixo. O akribie não tem esse teste — e é uma lacuna dele que podemos fechar melhor. |

### Custo e risco

**Custo**: baixo-médio — o custo real é a disciplina de não chamar `view` de dentro de `audit`.
**Risco**: alguém "otimiza" adicionando cache de validação dentro de `Doc`, o que quebra imutabilidade
e `Send + Sync`. Regra explícita: **validação é função pura; cache é do consumidor.**
**Bônus**: `validate` é o que o `ImportReport` e o prompt de agente vão usar depois; e é a base da
ideia 6 (descrição acessível) sem duplicar travessia.

---

## 3. Forma canônica do wire com prova de idempotência — **agora**

### Hipótese

"Uma única representação válida" só é verificável se houver uma função `canonical()` e uma
propriedade testável `canonical(canonical(x)) == canonical(x)`. Sem isso, "canônico" é uma intenção:
dois caminhos de código produzem dois JSONs diferentes para o mesmo documento e o contrato de bytes
morre.

### Evidência no akribie

- `crates/akribie-core/src/fmt.rs:3-13` — **oito regras enumeradas e finitas** no doc-comment:
  ordem de classes, exatamente uma linha em branco entre classes, comentários colados ao próximo
  statement, um espaço separador, strings sempre entre aspas, opções em ordem alfabética, números
  canônicos, arquivo terminado com exatamente um `\n` (documento vazio ⇒ saída vazia).
- `fmt.rs:19-47` — `format(document) -> String` constrói blocos por classe e junta com `\n`.
  `fmt.rs:76-90` ordena as chaves com `sort_unstable` **a cada emissão** — a ordem não é confiada ao
  `IndexMap`.
- `crates/akribie-core/src/num.rs:8-25` — o único caminho de número para texto.
- Testes: `crates/akribie-cli/tests/fmt.rs:7-47` (goldens `fmt_canonical.akrc` → `fmt_expected.akrc`,
  idempotência, `--check` divergente = `exit 1`). Verificado por mim **[E]**: duas passadas de `fmt`
  dão md5 idêntico (`791747402290505e0e02155d0510775e`).

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| `wire` (`plan/04_arquitetura.md:67`) | `pub fn canonical(doc: &Doc) -> Bytes` (e `canonical_value`) como **única** função de emissão. Um `Envelope { version, doc, schema_fingerprint }` fixo, com ordem de campos determinística. |
| `schema` | `fingerprint` já está no plano (`plan/04_arquitetura.md:59`). A forma canônica **deve** incluí-lo, para que um documento com schema incompatível não passe por "igual". |
| teste (testkit) | propriedade: `canonical(parse(canonical(d))?) == canonical(d)` para todo `Doc` gerado por strategy. Mais um teste de *estabilidade*: um corpus fixo mapeia para `golden` de bytes, e o golden é versionado. |
| guarda | `canonical` é a única função que pode produzir `Bytes` de documento; `Debug` não é wire (`plan/06_api-e-integracao.md:451` — o nosso plano já diz isso; o akribie reforça: `fmt` é a única escrita). |
| CLI | um `--check` equivalente (`rich-editor-auy vet`/`fmt --check`) que sai não-zero quando o arquivo difere, para prender a forma canônica em CI de consumidores. |

### Custo e risco

**Custo**: baixo para o writer; médio para o corpus de goldens.
**Risco**: canônico com float. O nosso plano já proíbe float no wire (`AGENTS.md` §3.7), o que
elimina a maior fonte de instabilidade — mas **torna obrigatório** decidir a forma canônica de
`AttrValue` (ordem de chaves, escapes, normalização Unicode). O akribie tem a mesma pressão e resolve
com "opções em ordem alfabética" (`fmt.rs:76-90`); nós precisamos da regra para `BTreeMap` de attrs.
**Risco 2**: `canonical` versionado. O akribie amarra a gramática a `chart v1` e usa `E009` para
versão desconhecida (`diag.rs:71-72`). Precisamos do mesmo: `wire_version` no envelope, e recusa
**com diagnóstico**, não best-effort.

---

## 4. Envelope único; erro de fronteira é *resultado*, não exceção — **agora**

### Hipótese

Duas superfícies de borda (CLI e MCP) que reportam erro de formas diferentes obrigam o consumidor a
duas lógicas. E uma ferramenta que **derruba** o servidor por causa de uma entrada ruim é pior que
uma que devolve erro.

### Evidência no akribie

- `ref/akribiechart/README.md:91` + verificado **[E]**: `--json` emite exatamente
  `{"success","command","data"?,"error"?,"warnings"}` — uma forma, em qualquer verbo.
- `crates/akribie-mcp/src/tools.rs:1-5` — a doc diz, sem rodeio: "Toda falha de execução volta como
  **resultado** com `isError: true` (e a primeira linha do diagnóstico em `content[0].text`); só
  `tools/call` com nome desconhecido é erro de protocolo. O servidor nunca cai por causa de um
  `.akrc` ruim."
- `tools/run.rs:11-39` — `check()` devolve `error_result(&diagnostic_line(...))`; nunca `Err`.
  `tools.rs:77-90` — `call()` só retorna `Err` para nome de ferramenta desconhecido.
- `crates/akribie-mcp/tests/mcp.rs` — 5 testes, incluindo
  `malformed_json_is_a_protocol_error_without_killing_the_server` e
  `invalid_source_is_is_error_and_the_server_stays_alive`. Verificado **[E]**: 4 chamadas reais numa
  sessão, incluindo `chart_diff` com `~ node a label: a -> A2`.
- `crates/akribie-core/src/error.rs:37-46` — 5 naturezas → 5 exit codes, numa tabela única.

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| `wire` | `Envelope<T> { success: bool, command: &'static str, data: Option<T>, error: Option<WireError>, warnings: Vec<Diagnostic> }`, com `WireError { code, message, diagnostics }`. Serialização determinística. |
| CLI | exit code derivado de `ErrorKind`/`DiagCode` por **uma** função; `--json` sempre emite o envelope em `stdout`, log sempre em `stderr`. |
| MCP | `tools/call` que falha retorna `{ isError: true, content: [{type:"text", text}] }`. Erro de protocolo só para JSON-RPC inválido e ferramenta desconhecida. |
| regra de desenho | **o domínio não sabe que o envelope existe**. Isto o akribie acerta: `pipeline::check` devolve `CheckReport`, e é `emit_outcome`/`tools::run` que monta o envelope. Se o envelope aparecer no core, o core deixou de ser headless. |
| teste | um teste por superfície que prova "entrada inválida não mata o processo/servidor". É o teste mais barato e mais valioso destes três. |

### Custo e risco

**Custo**: baixo — é contrato, quase não é código.
**Risco**: tentação de pôr `success`/`warnings` no core "para simplificar". Proibido: envelope é
adaptador. O `warnings: Vec<Diagnostic>` pode ser reutilizado do core; o *envelope* não.
**Risco 2, com evidência**: campo de envelope que ninguém preenche. O akribie declara
`pub(crate) warnings: Vec<String>` e os **três** construtores o fixam em `Vec::new()`
(`crates/akribie-cli/src/envelope.rs:35, 46, 60, 78`); os avisos reais ficam em
`data.diagnostics`. Resultado: superfície pública com campo morto, que o consumidor vai tentar ler.
Regra para nós: **todo campo do envelope tem teste que o vê não-vazio em algum cenário**, ou o campo
não existe.

---

## 5. Matriz de features em CI + guarda única de float→texto — **agora**

### Hipótese

Uma feature declarada e não testada é uma feature quebrada com documentação mentindo. O akribie tem
exatamente esse defeito, e o custo de preveni-lo é um alvo de `make`.

### Evidência no akribie (o defeito)

- `crates/akribie-core/Cargo.toml:12-16` declara `default = ["raster"]` e
  `raster = ["dep:resvg", …]`, com o comentário de que desligar deixa "só SVG/scene/excalidraw".
- **Verificado [E]**: `cargo build -p akribie-core --no-default-features` →
  `error[E0433]: cannot find module or crate 'image' in this scope`. `grep -rn 'feature = "raster"'
  crates/` → **vazio**. O módulo é declarado incondicionalmente (`crates/akribie-core/src/emit/mod.rs:4`).
- Consequência: `Makefile:41-43` roda `cargo tree -p akribie-core --no-default-features`, que
  **não compila nada** — logo o gate passa sem provar a configuração. Teatro.

### Evidência no akribie (o acerto correlato)

- `crates/akribie-core/src/num.rs:1-25` — uma função, um contrato: `f32::to_string` é proibido na
  saída "porque muda entre versões da std"; o substituto fixa 2 casas, remove zeros e normaliza `-0`.
- `Cargo.toml:77-120` — `disallowed_methods`, `disallowed_types`, `disallowed_macros` como `deny`
  com `allow_attributes_without_reason`. Ou seja: o mecanismo para *proibir* `to_string` por lint já
  existe no workspace; falta usá-lo para isso.

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| `math` (`plan/04_arquitetura.md:68`) | uma única `fn format(v) -> &'static str`/`itoa`-like para qualquer número que chegue ao wire. Mesmo com "sem float no wire", há inteiros (tamanhos, offsets, contadores) e a **mesma** regra vale: uma porta. |
| CI | matriz de features explícita: `--no-default-features`, `--features serde`, `--features macros`, `--all-features`, `--no-default-features --features alloc`. Cada combinação **compila** e roda um subconjunto de testes. |
| lint | `clippy::disallowed_methods` com `f32::to_string`/`f64::to_string` (e `format!` com `{v}` de float) negados; `allow` só com `reason`. |
| guarda extra | teste que percorre `src/**/*.rs` procurando `to_string` em tipo float — o equivalente do `scripts/check_file_length.sh` do akribie (`Makefile:26-27`), que é o padrão certo: gate em script no `make check`. |

### Custo e risco

**Custo**: ~30 linhas de `Makefile`/CI + a matriz rodando 5× (tempo de CI).
**Risco**: `no_std` + `alloc` + feature matrix multiplica tempo de CI. Mitigação: matriz completa só
no CI, e um subconjunto (`default` + `no-default`) no `make check` local. É o que o akribie tenta
fazer e falha por não compilar a variante.

---

## 6. `DocOutline`: descrição acessível derivada do modelo — **pós-MVP**

### Hipótese

Se cada adaptador (DOM, SVG, terminal, Markdown export) inventa sua própria descrição textual do
documento, eles divergem. Uma descrição **estruturada** derivada do `Doc` no core — lista de blocos,
texto, headings, links, marcação de ênfase — dá a todos a mesma fonte, sem que o core prometa ARIA.

### Evidência no akribie

- `crates/akribie-core/src/a11y.rs:16-42` — `describe(&Chart) -> String` monta
  `"N nós, M arestas. Nós: a; b.\nArestas: x → y (label)."` **só** a partir de `chart.nodes` e
  `chart.edges`. Zero geometria.
- `docs/adr/ADR-0005-acessibilidade-svg.md:82-85` — rejeita explicitamente "`<desc>` gerado da
  geometria": "descreveria posições em vez de relações — ruído para o leitor de tela e para o agente".
- `crates/akribie-core/src/emit/svg.rs:34-37` — `<desc>` é o segundo filho, sempre presente;
  `a11y.rs:1-4` diz que a descrição "é o que um leitor de tela lê e o que um LLM recebe no lugar da
  imagem. Ela é determinística e não depende de layout".
- **O contraexemplo está no mesmo arquivo**: `a11y.rs:30-41` tem "nós"/"arestas" hardcoded, e com
  `chart … lang=en` o SVG declara `lang="en"` e descreve em português (**[E]**, ver
  [`exemplos/README.md`](exemplos/README.md) §7).

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| core, módulo novo `outline` | `pub fn outline(doc: &Doc, schema: &Schema) -> DocOutline` onde `DocOutline` é **dado estruturado**, não prosa: `Vec<OutlineBlock>` com `kind: BlockKind`, `text: &str` (ou `TextRun` view), `children: Vec<OutlineBlock>`, `link: Option<RefTarget>`, `level: u8`. Determinístico, sem alocação de `String` para o texto (fatias do `Doc`). |
| adaptador | **ele** formata: `<desc>` do SVG, `aria-label`, texto para leitor de tela, sumário de Markdown, prévia de busca. Cada superfície escolhe granularidade e idioma. |
| i18n | a string de contagem/plural **não** vai para o core: `DocOutline` carrega contagens e o adaptador escolhe as palavras. É como se evita o defeito do akribie. |
| reuso | `markdown`/`text` export (pós-MVP) consomem o mesmo `DocOutline` — uma travessia, três saídas. |
| contrato | `outline` é puro e barato (uma travessia O(nós)), logo também serve ao ideia 2 (`validate` pode usar para checar limites de `alt`). |

### Custo e risco

**Custo**: médio — `DocOutline` é uma segunda "view" do documento e precisa de teste de equivalência
contra `view_data` (mesmos blocos, mesma ordem, mesmo texto).
**Risco 1**: virar uma segunda árvore de renderização que ninguém mantém. Mitigação: `DocOutline`
tem **menos** campos que `Doc` — só o que uma descrição pode dizer. Não é view data.
**Risco 2**: a11y real exige mais que descrição (foco, ordem de tabulação, live regions). Essas
**não** são do core (`plan/06_api-e-integracao.md:22-25` já diz isso) e continuam sendo do adaptador.
**Não fazer agora** porque o MVP ainda não tem export nem a11y renderizada; construir `outline` antes
de haver dois consumidores é especulação.

---

## 7. Diff semântico por identidade de nó — **pós-MVP**

### Hipótese

`ChangeSet` responde "o que mudou nesta transação". Ele não responde "qual é a diferença entre estes
dois documentos" — a pergunta de revisão, de auditoria, de "o que o agente mudou", e de teste (o
oráculo: *replay* dos steps deve produzir exatamente o diff semântico calculado por fora).

### Evidência no akribie

- `crates/akribie-core/src/diff/mod.rs:1-7` — o doc-comment declara o desenho inteiro: compara o
  modelo resolvido, "nunca geometria: o layout é derivado e muda sem mudança de intenção";
  identidade `node`/`group` por `id`, `edge` por `(from, to, ordinal)`, `note` por âncora + posição
  entre as notas da mesma âncora (porque "a gramática não dá id a nota" — regra declarada, não
  improvisada).
- `diff/mod.rs:32-44` — `Change { kind: Added|Removed|Changed, element: String, field:
  Option<&'static str>, before: Option<String>, after: Option<String> }`. `Option` porque `Added` e
  `Removed` não têm par antes/depois — o tipo não permite estado inválido.
- `diff/mod.rs:84-132` — ordem de saída fixa: todos `added`, depois `removed`, depois `changed`,
  cada grupo ordenado por `element` (e por `field` nos empates). **Determinístico sem `HashMap`**
  (`BTreeMap` em `:9`).
- `diff/describe.rs:77-127` — a tabela de campos por tipo de elemento (`node_fields`, `edge_fields`,
  …). É isto que faz o diff ser "campo mudou" e não "string mudou".
- Executado **[E]**: `akr diff retry.akrc novo.akrc` → `- edge retry->api "backoff"` +
  `~ node retry shape: cloud -> rounded`, exit 1.

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| core, `diff`/`audit` | `pub fn diff(before: &Doc, after: &Doc) -> Vec<DocChange>` com identidade **`NodeId`** (já temos ids estáveis — `plan/00_panorama.md` §1 e `plan/04_arquitetura.md` §3). Para texto, o "elemento" é o bloco e o "campo" é o texto/mark/attr — comparar por `(NodeId, field)`. |
| `DocChange` | `enum DocChange { NodeAdded(NodeId), NodeRemoved(NodeId), Field { id: NodeId, field: &'static str, before: Option<Cow<str>>, after: Option<Cow<str>> }, TextChanged { id: NodeId, range: Range<Pos> } }`. Mesma disciplina de `Option` para não permitir estado inválido. |
| `testkit` (uso imediato) | **oráculo**: depois de aplicar uma `Transaction`, `diff(before.into(), after)` deve ser consistente com os `steps` — todo `replace` produz `TextChanged` no range mapeado, todo `set_attr` produz `Field`. Isso pega bug de `Mapping`/`invert` que teste de estado não pega. **Este é o motivo de valer a pena mesmo antes de existir UI de revisão.** |
| crate de revisão (pós-MVP) | a UI de review, o `RefTarget`/backlinks e o prompt de agente ("o que mudou desde X") consomem o mesmo `diff`. |
| CUIDADO | a identidade do akribie é "id do nó"; a nossa precisa de uma regra para nó **sem** id e para texto dentro de bloco. O akribie declara a regra no topo do módulo (`:4-7`); nós precisamos declarar a nossa **antes** de implementar, senão o diff vira "tudo mudou". |

### Custo e risco

**Custo**: médio. A tabela de campos por tipo é chata e precisa acompanhar o schema (mesmo problema
do `syntax::specs` do akribie: uma fonte, três consumidores).
**Risco 1**: diff de texto é O(texto) e pode ser caro em documentos grandes. Mitigação: só por
`NodeId` + `Range`, sem LCS fino no MVP do módulo.
**Risco 2**: virar uma segunda serialização que mente. Mitigação: `diff` é derivado e testado contra
o replay de steps (o oráculo acima) — nunca fonte de verdade.
**Por que pós-MVP**: antes de haver colaboração/revisão, o `ChangeSet` cobre a UI. O uso imediato é
**teste**, e isso pode entrar no `testkit` cedo, sem API pública.

---

## 8. `ImportReport` obrigatório, com degradação contada e round-trip testado — **pós-MVP**

### Hipótese

Import que "funciona" mas perde semântica em silêncio é pior que import que falha: o consumidor
acredita que tem o documento original. O relatório precisa ser **retorno obrigatório** (não
`Option`), **contado** por categoria, e o round-trip precisa de teste.

### Evidência no akribie (o defeito)

Verificado **[E]** em [`exemplos/README.md`](exemplos/README.md) §9.3:

| original | após `akr import` | perda |
|---|---|---|
| `layout=flow` | `layout=free` | topologia vira coordenada |
| `shape=cloud`, `shape=cylinder` | todos `rounded` | forma |
| `curve=arc` | ausente | estilo de rota |
| ids `api` | `akr-api` | identidade renomeada |

O akribie **tem** o código `W006 element-dropped` (`crates/akribie-core/src/diag.rs:84`,
"Elemento não representável descartado na importação") e **tem** `W005 color-unmapped`
(`diag.rs:82`). Mas o `akr import retry.excalidraw` que executei **não emitiu nenhum `W006`** para as
três perdas acima: o formato não as considera "descartadas" — ele as considera "convertidas com
sucesso". É a diferença entre *loss* e *lossy*, e o relatório não distingue.

### Encaixe no `rich-editor-auy`

| camada | mudança |
|---|---|
| `-html` / `-markdown` (`plan/04_arquitetura.md:40-41`) | `pub fn import(bytes: &[u8], schema: &Schema) -> Result<(Doc, ImportReport), ParseError>` — **o relatório está no `Ok`**, não é opcional. |
| `ImportReport` | campos **contados** por categoria: `dropped: Vec<DroppedItem>`, `downgraded: Vec<Downgraded>`, `unknown_attrs: Vec<…>`, `truncated: bool`, `bytes_in/out`. `plan/06_api-e-integracao.md:287` já prevê "lista o que foi descartado" — a ideia é incluir **downgrade**, não só descarte. |
| `wire` | `ImportReport` serializável (é parte do `Envelope`, `plan/04_arquitetura.md:67`). |
| diagnóstico | cada item degradado vira um `DiagCode` **`W`** (ideia 1), e `import --strict` promove a erro. Assim o mesmo mecanismo de diagnóstico serve ao import. |
| teste | round-trip: `export(import(x))` comparado com `x` no **nível semântico** (usando a ideia 7) + assert de que o relatório conta exatamente as degradações. É o teste que o akribie não tem. |

### Custo e risco

**Custo**: médio-alto (dois formatos, dois parsers). É por isso que é pós-MVP.
**Risco**: ninguém usa o relatório e ele apodrece. Mitigação: `#[must_use]` no `ImportReport`, e o
`Envelope`/CLI **sempre** o imprime quando não vazio.
**Por que pós-MVP**: depende de existir import. Mas o **contrato** (`Result<(Doc, ImportReport), _>`)
deve ser decidido agora, porque mudá-lo depois é breaking.

---

## 9. **Rejeitar**: paleta fechada e tokens de cor dentro do core

`crates/akribie-core/src/theme/mod.rs:37-72` tem `Token` com 9 variantes e dois temas fechados, e
`a11y.rs:58-136` valida contraste **no core**.

É correto para o akribie (o SVG é o produto; a cor é semântica do diagrama) e **errado para nós**:

- `AGENTS.md` §3.13 e `plan/06_api-e-integracao.md:22-25` dizem que ARIA, contraste e tema são do
  adaptador. Um `Token::Blue` no core é exatamente a "segunda fonte de verdade" que o
  `AGENTS.md` §3.8 proíbe.
- O nosso wire não pode carregar cor: um documento compartilhado entre um host claro e um escuro não
  pode serializar "azul". O akribie pode, porque o artefato é a imagem.

**O que aproveitar**: o *mecanismo* "conjunto declarado → teste exaustivo de todas as combinações"
(`theme/tests.rs`) transfere para os nossos conjuntos declarados: `NodeSpec` × `MarkSpec` ×
`AttrSpec` × `ContentMatch`. Um teste que percorre **todas** as combinações do schema é o equivalente
exato em custo/benefício. Fazer isso no `testkit`, não um módulo de cor.

## 10. **Rejeitar**: gramática textual linha-a-linha como formato de wire

`crates/akribie-core/src/lex.rs:41-57` e `docs/specs/GRAMMAR.md:9-21` implementam um formato de uma
afirmação por linha, com `chave=valor` indivisível (`lex.rs:59-95`) e tab proibido (`lex.rs:47-52`).

É uma ótima escolha **para um arquivo editado por humanos/LLM em diff de Git**. Não serve ao nosso
wire:

- O nosso wire é JSON canônico consumido por PHP/TS, com `AttrValue` tipado e `schema_fingerprint`
  (`plan/04_arquitetura.md:67`). Reimplementar um parser de linha próprio adiciona superfície de erro
  sem ganho: o JSON já tem parser no consumidor.
- A fragilidade é mensurável: `edge api->retry` é `E007` porque a seta precisa ser um token isolado
  (`lex.rs:97-123`, verificado **[E]**). Um gerador de texto humano escreve a forma compacta. Nós não
  temos esse problema porque não geramos texto à mão.
- O que **é** transferível: a **recuperação por unidade** (um erro não invalida o resto,
  `GRAMMAR.md:3-5`) e `Span` em **caracteres** e 1-based, apontando o que o autor vê
  (`crates/akribie-core/src/span.rs:5-13`). Isso entra no ideia 1.

## 11. **Rejeitar**: busca linear por id

`crates/akribie-core/src/model/mod.rs:151-174` — `Chart::node(id)` é `self.nodes.iter().find(...)`,
usado por `label_of` em `a11y::describe` (por aresta) e no diff.

Em 1000 nós isso é invisível (14 ms de `check`), mas o padrão é O(V·E). Nós temos `NodeId` estável e
documentos com dezenas de milhares de nós; herdar isso seria herdar um teto. O akribie já resolve o
caso quente com um `IndexMap` (`crates/akribie-core/src/layout/finish.rs:40-45`) — a inconsistência
é o cheiro. **Regra para nós**: toda travessia por id usa índice (`BTreeMap<NodeId, _>` ou
`NodeSlot`), e o índice é construído uma vez por operação, não por consulta.

---

## 12. Ordem de execução sugerida

1. **Antes de fechar `error`/`wire`** (bloqueia o resto): ideias **1** e **4**. São contrato; mudá-las
   depois é breaking.
2. **Junto com `state`**: ideia **2**. `validate` nasce separado de `view_data` ou nunca nasce.
3. **Junto com `wire`**: ideia **3** e a metade `math` da ideia **5**.
4. **No CI, quando F0 existir**: matriz de features da ideia **5**.
5. **Quando o primeiro import existir**: ideia **8** (o contrato `(Doc, ImportReport)` pode ser
   decidido antes; a implementação espera).
6. **Quando houver segunda superfície de a11y/export**: ideia **6**.
7. **Quando houver revisão/colaboração — ou antes, como oráculo no `testkit`**: ideia **7**.

Duas coisas que este dossiê **não** recomenda copiar em nenhuma fase: a paleta fechada (ideia 9) e a
gramática textual (ideia 10). Ambas são boas **para o akribie** e ruins para um engine headless cujo
wire é JSON e cuja apresentação é do host.
