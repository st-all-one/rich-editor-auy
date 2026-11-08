# Ideias cruzadas: `akribie` e `calc-auy` para o `rich-editor-auy`

> **Status:** investigação, não decisão ratificada. As ideias abaixo foram extraídas da leitura
> direta dos dois projetos; os dossiês [`akribiechart/`](akribiechart/) e [`calc-auy/`](calc-auy/)
> trazem a evidência completa, os testes e a revisão independente. Nenhuma linha de código do
> `rich-editor-auy` foi alterada.
>
> **Veredito curto:** nenhum dos dois é editor de texto rico; ambos são bons **fornecedores de
> disciplina de artefato**. O maior valor não está no DOM, no layout ou no CSS — está em
> **identidade, hash, diff, validação em camadas, schema introspectável e erros estáveis**.

## 1. O que cada projeto é (e não é)

| Projeto | O que é | O que não é | Valor para nós |
|---|---|---|---|
| **akribie / akribiechart** | compilador determinístico de diagramas: `.akrc` → `Document` → `Chart` → `Placed` → `Scene` → SVG/JSON/Excalidraw/raster | editor interativo, editor de texto, DOM, contenteditable | pipeline puro, diff semântico, schema derivado, a11y derivada do modelo, diagnóstico por linha |
| **calc-auy** | calculadora exata/auditável: AST em arena, custódia verificada, hash canônico, assinatura BLAKE3, typestate, render LaTeX/HTML/SVG/PNG | editor, modelo de documento, layout de texto | separação conteúdo/proveniência, artefato verificado, erros estruturados, schema versionado, disciplina de testes |

Os dois usam Rust, lints fortes, `#![forbid(unsafe_code)]` no núcleo, testes de contrato e documentação
de decisão. Isso **confirma** o plano atual; o que segue são os pontos que ainda não estão
explicitados nele.

## 2. Prioridade

| # | Ideia | Origem principal | Encaixe | Custo | Recomendação |
|---|---|---|---|---|---|
| **I1** | `content_hash` e `trace_hash` separados | calc-auy | `Doc`/wire/cache | baixo | **agora** (API derivada, sem hot path) |
| **I1b** | assinatura criptográfica sobre o envelope canônico | calc/akribie | `integrity` + `wire` | médio | **agora como crate separado**; BLAKE3 é o único hash, FNV/SHA-256 não entram |
| **I2** | `Doc::diff(old, new)` semântico | akribie | crate `diff`/CLI/revisão | médio | **agora** como crate separado, pós-MVP no core |
| **I3** | `Schema::describe()` + JSON Schema gerado | akribie + calc-auy | schema/CLI/PHP/TS | médio | **agora** no `xtask`, sem `build.rs` |
| **I4** | validação em camadas + `ValidationReport` | akribie | `validate`, `refs`, import | baixo | **agora** |
| **I5** | `A11yTree`/descrição semântica para a11y | akribie | core semântico + adaptador | baixo/médio | **agora**, invariante de core (D58/D61/D62); não é pós-MVP |
| **I6** | erro estável com `code`/`path`/`hint`/`warnings` | akribie + calc-auy | `AuyError`, CLI, PHP/TS | baixo | **agora** (forma), sem HTTP/UUID |
| **I7** | import com recuperação e múltiplos diagnósticos | akribie | `html`/`markdown`/JSON | médio | **pós-MVP**, junto do import |
| **I8** | `ValidatedDoc`/artefato verificado | calc-auy | `Doc`/import | baixo, mas risco de overengineering | **rejeitar como tipo público**; manter invariantes |
| **I9** | nó de matemática/fórmula exata | calc-auy | extensão de produto | alto | **pós-MVP/experimental**, fora do core |
| **I10** | `schema`/`prime`/MCP para agentes | akribie | CLI/LLM tooling | baixo | **pós-MVP**, derivado do schema |

## 3. As ideias em detalhe

### I1. `content_hash` ≠ `trace_hash` (calc-auy)

O calc separa dois hashes do mesmo artefato:

- `value_hash` canonicaliza a AST **sem metadados** e serve de chave de memoização/dedup
  (`ref/calc-auy/crates/calc-auy-core/src/integrity/hash.rs:1-25`;
  `engine/eval.rs:311-335`; `engine/cache.rs:1-50`);
- `trace_hash` inclui metadados e serve de chave de integridade/auditoria
  (`integrity/hash.rs:20-24`; `canonical/mod.rs:51-67` demonstra `Trace` vs `Value`).

No nosso caso, isso vira:

```rust
impl Doc {
    /// Estrutura + texto + marks + attrs, sem NodeId/DocId/proveniência.
    /// Cache de layout, dedup, SSR, comparação de conteúdo.
    pub fn content_hash(&self) -> ContentHash;

    /// Inclui NodeId e RefTarget; integridade/persistência/version tag.
    pub fn trace_hash(&self) -> TraceHash;
}
```

Regras:

- **Hash não é identidade.** `NodeId` continua sendo o ID opaco de D53; `content_hash` não pode
  substituir nem reescrever IDs.
- `schema_fingerprint` é uma terceira coisa (schema, não documento) e não entra no `Doc`.
- `content_hash` pode ser **por bloco** e por documento: o hash do bloco invalida cache de layout/
  re-render; o hash do documento serve de ETag/dedup. Não confunda os dois com `trace_hash` nem com
  `schema_fingerprint`.
- `content_hash` pode ignorar `NodeId`, `DocId`, `created_at` e outros metadados de publicação;
  `trace_hash` inclui o que precisa sobreviver a uma auditoria.
- Calcular sob demanda (`save`, `check`, checkpoint de collab), **nunca por tecla**. O calc só
  calcula hash de subárvore no caminho de avaliação memoizada; nós não precisamos disso no keystroke.
- **Assinatura não é hash de cache.** O requisito de “conteúdo não alterado” exige um crate
  separado (`rich-editor-auy-integrity`) com **BLAKE3-256 + Ed25519** sobre os bytes canônicos do
  `Envelope`; o fingerprint de schema (D13) também é BLAKE3-256 e **não** prova integridade. A chave é do
  host; o core só canonicaliza e expõe bytes. Assinar o `A11yTree` ou o `ChangeSet` não faz sentido:
  eles são derivados.
- Não copie BLAKE3 por reflexo para o cache: use o hash existente. BLAKE3 é o único hash do
  projeto (fingerprint, envelope e integridade), com teste de vetor conhecido.

**O que evita:** cache invalidado por metadado, dedup errado, “hash de conteúdo” usado como âncora,
e a confusão entre fingerprint de schema, hash de documento e ID.

### I2. `Doc::diff(old, new)` semântico (akribie)

O `akr diff` compara o **modelo resolvido**, não geometria nem bytes
(`ref/akribiechart/crates/akribie-core/src/diff/mod.rs:1-16`). A identidade é explícita: nó/grupo por
`id`, aresta por `(from, to, ordinal)`, nota por âncora + posição. O resultado é `+`/`-`/`~` com
campo, antes e depois (`Change::line`, `diff/mod.rs:34-58`). Fonte inválida não produz diff; produz
diagnósticos (`cli/commands/diff.rs:1-30`).

No `rich-editor-auy`:

- `ChangeSet` continua sendo **derivado de steps** (D25), para invalidar UI no caminho quente.
- `diff(old, new)` é outra operação: compara **dois documentos persistidos**, sem steps, para
  revisão, histórico, import e “o que mudou entre v7 e v8”.
- Identidade: `NodeId` quando existir; senão caminho estrutural estável (tipo + índice entre irmãos
  do mesmo tipo). Não use posição absoluta: ela muda com qualquer edição anterior.
- Mudanças de bloco: `type`, `attrs`, `marks`, texto e `RefTarget`; mudanças de inline podem ser
  resumidas por run/range, não por caractere.
- O diff **não** é hot path e **não** deve entrar no core publicado; um crate
  `rich-editor-auy-diff` (ou módulo opcional) mantém o core pequeno.
- Saída estruturada (`kind`, `path`, `node_id`, `field`, `before`, `after`) antes de texto; o CLI
  formata. Isso serve para UI, auditoria e agentes.

**O que evita:** “diff” textual ruidoso, comparação por posição que quebra após edições, e a
necessidade de reconstruir steps para comparar versões.

### I3. `Schema::describe()` + JSON Schema gerado (akribie + calc-auy)

O akribie deriva o `akr schema` das **mesmas tabelas** que alimentam o parser e o `prime`
(`ref/akribiechart/crates/akribie-cli/src/commands/schema.rs:1-90`): statements, chaves, tipos,
defaults, valores fechados, temas, contraste, códigos e exit codes. Nada é escrito à mão; uma chave
nova aparece no schema sem edição adicional.

O calc mantém JSON Schema, CDDL e Protobuf versionados (`ref/calc-auy/schema/`), com
`trace_version`; a fonte é o schema, e os codecs derivam dele. O ponto positivo é o **artefato
legível por consumidores**; o negativo é a duplicação/build.rs (ver §4).

No `rich-editor-auy`:

```rust
impl Schema {
    pub fn describe(&self) -> SchemaDescription; // nodes, marks, attrs, content, groups, limits
    pub fn to_json_schema(&self) -> String;      // derivado, não fonte
}
```

- `SchemaDescription` é a fonte de tooling: UI builder, TS/PHP, `llms.txt`, `schema --json`.
- Um JSON Schema **gerado por `xtask` e commitado** ajuda PHP/TS a validar a *forma*; a semântica
  continua no Rust. Teste de paridade obrigatório (`schema()` == arquivo commitado).
- Não use `build.rs` (regra do projeto) e não mantenha uma segunda definição manual.
- Inclua `schema_fingerprint`, versão do formato, limites e códigos de erro no mesmo documento.
- O schema nunca é autoridade de validação no cliente; é contrato de forma para mensagens e
  ferramentas.

**O que evita:** PHP/TS adivinhando nomes de campos, UI builder hardcoded, LLM inventando schema, e
documentação que diverge do código.

### I4. Validação em camadas + `ValidationReport` (akribie)

O akribie tem `check` estrutural e `check --docs` que acrescenta a verificação de `doc=`, só quando
pedida (`ref/akribiechart/crates/akribie-cli/src/commands/check.rs:1-70`;
`doc.rs:1-45`). A separação é explícita: o núcleo não toca no disco para validar referências em todo
save; o I/O é opcional e o diagnóstico é o mesmo tipo.

No nosso caso:

```rust
pub enum ValidationLevel {
    Structural,          // schema, normalização, IDs únicos, limites
    WithReferences,      // + RefTarget formal, sem resolver corpus
    // corpus/backlinks ficam no crate `refs`, não no core
}

pub struct ValidationReport {
    pub errors: Vec<Problem>,
    pub warnings: Vec<Problem>,
}
```

- `Doc::validate(Structural)` é puro e barato; é o que `apply`/`from_json` já devem garantir.
- **Limites durante a leitura, não depois.** O calc impõe `Limits` no `DeserializeSeed`
  (`ref/calc-auy/crates/calc-auy-core/src/ast/wire.rs:172-183`): um JSON hostil é rejeitado antes
  de virar árvore. O nosso `from_json` deve fazer o mesmo (profundidade, nós, bytes, runs) com um
  visitor/parser que conta enquanto lê; `serde` derive sozinho aloca primeiro e valida depois.
- `refs::validate(corpus)` é a camada que resolve `MissingDoc`/`MissingNode`/`VersionMismatch`;
  não entra no core.
- `ImportReport` (D38) e `ValidationReport` compartilham `Problem`, mas não se confundem: import
  fala de perda de mapeamento; validação fala de invariantes.
- `warnings` existem para diagnóstico não fatal (ex.: nó sem ID, referência não resolvida,
  atributo ignorado). `--strict` promove warnings a erro, como no akribie.
- O CLI devolve `ValidationReport` em JSON; o host decide o que bloquear.

**O que evita:** validação de referências cara em todo save, I/O no core, e a tentação de tratar
“warning” como erro ou de descartar diagnóstico não fatal.

### I5. `A11yTree`/descrição semântica para acessibilidade (akribie)

O `a11y::describe` do akribie produz uma descrição textual determinística **do modelo**, não da
geometria: contagem, nós e arestas (`ref/akribiechart/crates/akribie-core/src/a11y.rs:1-55`). O
contraste é uma checagem separada (`a11y.rs:57-130`), porque depende de tema; não deve ir para o
nosso core.

No `rich-editor-auy`, o análogo **não é um `outline` pós-MVP**: é a `A11yTree` do core, com roles,
nomes, `alt`, `lang`/`dir`, ordem de leitura, ranges e validação (D58/D61/D62). Ela é derivada do
`Doc`+`Schema`, determinística e não é wire nem DOM:

```rust
pub struct A11yNode {
    pub id: Option<NodeId>,
    pub role: A11yRole,        // heading(level), paragraph, list, quote, code, link, image...
    pub name: Option<Box<str>>,
    pub alt: Option<Box<str>>,
    pub lang: Option<Box<str>>,
    pub dir: Option<Dir>,
    pub range: Range<Pos>,
    pub children: Vec<A11yNode>,
}

impl Doc {
    pub fn a11y_tree(&self, schema: &Schema) -> A11yTree;
    pub fn validate_a11y(&self, schema: &Schema) -> A11yReport;
}
```

- O core conhece schema e estrutura; deriva headings/listas/links/`alt`/`lang`/`dir` e valida o que
  falta (`alt` obrigatório, link sem nome, heading sem texto).
- O adaptador DOM mapeia a árvore para elementos nativos/ARIA, foco, teclado e anúncios; não inventa
  roles.
- O host continua dono de contraste, foco visível, leitor de tela e auditoria WCAG.
- `describe()` textual continua útil para LLM, busca e snapshot, mas é **view** da árvore; não
  substitui role/alt/nome nem validação.
- Não copie `W007`/contraste: o core não tem tema nem CSS.

**O que evita:** ARIA inventada no adaptador sem semântica do documento, documento que nasce sem
`alt`/nome, e acessibilidade tratada como pós-MVP. **A11y é invariante de core**, não feature
opcional; se uma entrega não tem árvore/validação, ela não está pronta.

### I6. Erro estável com `code`/`path`/`hint`/`warnings` (akribie + calc-auy)

O akribie tem `Diagnostic { code, span, message, hint }`, códigos estáveis (`E001`…`W007`) e uma
tabela `ALL` que alimenta `schema` e `prime` (`ref/akribiechart/crates/akribie-core/src/diag.rs:1-170`).
O envelope `--json` é `{success, command, data?, error?, warnings}` e preserva diagnósticos
(`crates/akribie-cli/src/envelope.rs:1-70`).

O calc tem `ProblemDetails` RFC 7807 com `type`, `title`, `status`, `detail`, `instance` e `context`
(`ref/calc-auy/crates/calc-auy-core/src/error.rs:50-120`), mas `instance` vem de um `Clock`/UUID
(`ADR-014-clock.md`), o que **não** é determinístico. Isso é uma boa forma, com um detalhe a
remover.

Para nós:

```rust
pub struct Problem {
    pub code: ErrorCode,          // &'static str estável
    pub severity: Severity,       // error | warning
    pub path: DocPath,            // bloco/run/campo
    pub offset: u32,              // byte/UTF-16? escolher e documentar
    pub detail: String,
    pub hint: Option<String>,
    pub context: BTreeMap<String, AttrValue>,
}
```

- `code` é estável e documentado; `schema()` publica a tabela.
- `path`/`offset` substituem “índice de array” e “posição de DOM”.
- `hint` é texto de correção; `warnings` são problemas não fatais.
- No CLI/bindings, um envelope `{success, command, data, error, warnings}` facilita PHP/TS.
- **Não** copie `status` HTTP para o core, nem `instance` UUID/relógio. Se precisar de
  `instance`, derive de `content_hash`/path+code ou deixe o host gerar.
- `Problem` é `Send + Sync`, `Clone`, e não aloca mais que o necessário; `BTreeMap` para ordem
  determinística (não `HashMap`).

**O que evita:** erros opacos, `any` no TS, mensagens que o PHP não consegue distinguir, e
não-determinismo em snapshots/testes.

### I7. Import com recuperação e múltiplos diagnósticos (akribie)

O parser do akribie **nunca falha no total**: cada linha inválida vira diagnóstico e as outras
continuam (`ref/akribiechart/crates/akribie-core/src/parse/mod.rs:1-90`). Isso é deliberado para um
LLM/agente receber todas as correções de uma vez. O `check` devolve `Document` + `diagnostics` no
mesmo `CheckReport` (`pipeline.rs:15-30`).

No nosso import HTML/Markdown:

- Um erro em um bloco não deve descartar o documento inteiro; produza `ImportReport` com
  `mapped`, `approximated`, `dropped` e `problems` (D38).
- Se o parser não conseguir montar um nó, mantenha o texto como parágrafo/`text` e registre
  `approximated`; não perca conteúdo em silêncio.
- O `ImportReport` deve ser serializável e determinístico; o CLI pode imprimi-lo em JSON.
- No core, o parser de wire JSON pode continuar estrito (não é import de terceiros); a recuperação é
  dos crates `html`/`markdown`.
- Limite o número de diagnósticos por documento para evitar flood (ex.: 100) e diga que truncou.

**O que evita:** “primeiro erro aborta”, perda silenciosa de conteúdo, e um import que não explica
o que aproximou.

### I8. `ValidatedDoc`/artefato verificado: copiar a disciplina, não o tipo (calc-auy)

O calc tem `VerifiedAst`: campo privado, só construído por `verify_custody`, e `commit` só aceita
`VerifiedAst` (`ref/calc-auy/crates/calc-auy-core/src/integrity/custody.rs:1-45`;
`engine/commit.rs:55-70`). É “parse, don't validate” aplicado a um artefato que atravessa contexto.

No nosso caso, **não** proponho um `ValidatedDoc` público em todo `apply`:

- `Doc` já tem campos privados e construtores validados; `apply` devolve `Result<State>`, e
  `State` só existe com `Doc` válido. O tipo extra não paga o custo de API.
- O que **vale** copiar do calc é o typestate de **estágio**, não de schema: `EditorState` →
  `Applied` → `ChangeSet` já impede pedir `change_set()` de uma transação não aplicada, e
  `Doc::from_json` já devolve `Result<Doc>`. Não crie `Doc<Draft>`/`Doc<Validated>` para cada nó;
  isso transforma schema dinâmico em explosão de tipos.
- Onde o padrão `VerifiedAst` ajuda de verdade: **import de terceiros**
  (`HtmlImporter` → `ValidatedDoc` local → `Doc`), **wire desconhecido** e **refs** que recebem
  documento de fora. Aí, um wrapper local impede usar o resultado antes de validar.
- Se houver um construtor “unchecked” público (não deve), o wrapper é obrigatório. Se não houver,
  o wrapper é cerimônia.

**O que evita:** copiar um typestate que não resolve um problema real do nosso modelo, e ao mesmo
tempo manter a porta para “doc de fora sem validar”. O calc recomenda `Draft → Committed → Verified`
“agora”; a versão mínima defensável para nós é `EditorState → Applied` + um wrapper **local** de
import, não um novo estado público de `Doc`.

### I9. Nó de matemática/fórmula exata (calc-auy) — experimental

O calc tem racional híbrido, AST verificada, LaTeX/Unicode/HTML/SVG/PNG e avaliação exata. Isso não
pertence ao core de texto rico: o core não deve saber avaliar expressões, nem depender de
`num-bigint`/KaTeX.

O que pode fazer sentido como **extensão de produto**:

- um nó atômico `Math`/`Formula` com `source` (TeX), `display` (inline/block) e um `value` opcional
  já calculado;
- um crate separado `rich-editor-auy-math` (ou integração do host) que valida/avalia via
  `calc-auy` e produz LaTeX/MathML;
- renderização no crate HTML/Markdown (KaTeX/MathML), nunca no core;
- o core só preserva o átomo, o `source` e o `value` opaco; não avalia.

**Recomendação:** post-MVP, somente se matemática for requisito de produto. Não acoplar o core a
`calc-auy`; a licença MPL-2.0 também desaconselha copiar código para um crate MIT/Apache sem
isolar o arquivo.

### I10. `schema`/`prime`/MCP para agentes (akribie) — opcional

O akribie tem `akr prime` (gramática + tabelas para colar em um agente) e um servidor MCP com
`chart_check`/`chart_render`/`chart_diff` (`ref/akribiechart/crates/akribie-cli/src/commands/prime.rs:1-80`;
`crates/akribie-mcp/src/tools.rs:1-80`). É uma boa ideia de **tooling**, não de core.

No nosso caso:

- `schema --json` e um `llms.txt`/`SKILL.md` gerados do schema já cobrem 80% do valor;
- um MCP `doc_check`/`doc_diff`/`doc_render` pode ser um binário opcional pós-MVP;
- não coloque MCP, JSON-RPC ou filesystem no core.

**Recomendação:** pós-MVP; primeiro `schema`, depois `llms.txt`, depois MCP se houver demanda.

## 4. O que **não** copiar

| Armadilha | Onde aparece | Por que não |
|---|---|---|
| `ProblemDetails.instance` com UUID/`Clock` | calc-auy `error.rs` + ADR-014 | quebra determinismo de erro/snapshot; o core não tem relógio |
| `build.rs` + schema duplicado | calc-auy `processors/build.rs` + `schema/*` | o projeto proíbe `build.rs` e segunda fonte; use `xtask` + paridade |
| `VerifiedAst` em todo `apply` | calc-auy | overengineering se `Doc` já é invariante; use só na borda de import |
| Racional/BLAKE3/KaTeX no core | calc-auy | acopla o core a matemática/cripto/render; mantenha em crate opcional |
| Contraste/tema/fonte no core | akribie `theme`/`a11y`/`font` | o core não tem CSS, tema nem layout; isso é adaptador/host |
| `f32`/formatação de geometria | akribie `num`/`layout` | o nosso wire não tem float; não introduza |
| `fmt` de fonte textual `.akrc` | akribie | o nosso formato é JSON; `normalize` + JSON canônico já é o equivalente |
| MCP/`prime` no core | akribie | tooling separado; o core continua sem I/O/rede |

## 5. Propostas de trabalho (não ratificadas)

Se o plano quiser absorver as ideias, a sequência de menor risco é:

1. **P1 — hashes de documento:** `content_hash`/`trace_hash`, sem identidade e sem hot path.
2. **P2 — diff semântico:** crate separado; `ChangeSet` continua por steps.
3. **P3 — schema introspectável:** `Schema::describe()` + JSON Schema gerado por `xtask`, com
   paridade e sem `build.rs`.
4. **P4 — validação em camadas:** `Structural`/`WithReferences`, `ValidationReport` com warnings.
5. **P5 — erros estáveis:** `Problem` com `code`/`path`/`offset`/`hint`/`context`, envelope de CLI
   sem UUID/HTTP no core.
6. **P6 — integridade/assinatura:** bytes canônicos + hash criptográfico/assinatura em crate
   separado (`integrity`); BLAKE3-256 é o fingerprint de schema e o digest do envelope. Chaves e
   política são do host.

`I2/I3/I4/I5` são independentes; `I1` deve vir antes de qualquer cache de layout. `I6` pode entrar
junto de `I4`. `I8` não vira decisão; é uma diretriz de implementação. `I9/I10` ficam explicitamente
fora do MVP. **A11y não é mais proposta:** `A11yTree`/`validate_a11y` são D58/D61/D62 e entram
antes de F1/F3; não há “D66 de a11y”.

## 6. Critério para aceitar uma ideia

Uma ideia só entra se responder “sim” a todas:

- resolve um problema real do nosso modelo/API, não do produto dos outros;
- cabe no core sem I/O, relógio, RNG, CSS ou dependência pesada; ou fica em crate separado;
- tem teste de contrato (golden/propriedade/diferencial) e um caminho de erro;
- não cria segunda fonte de verdade;
- não piora o caminho quente (keystroke);
- pode ser explicada em uma frase e recusada sem drama.

As duas que mais pagam agora são **I1** e **I2**; a que mais reduz atrito de integração é **I3**;
a que mais melhora diagnóstico é **I4 + I6**. As demais são opcionais e devem esperar evidência de
uso.
