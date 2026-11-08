# Arquitetura — `calc-auy`

> Caminhos relativos a `ref/calc-auy/` salvo indicação contrária. Commit
> `08f5a3f0a907cf503f8b2afbdae0a19f6618f79f` (2026-10-04), versão de workspace
> `0.1.0` (`Cargo.toml:21`), licença `MPL-2.0` (`Cargo.toml:24`), edition 2024
> (`Cargo.toml:22`), MSRV declarada 1.98 (`Cargo.toml:23`,
> `rust-toolchain.toml:2`).
>
> Este projeto **não é um editor**. É uma calculadora determinística com
> cadeia de custódia. O dossiê o lê como fonte de padrões de *pipeline*,
> *canonicalização*, *tipos de estado* e *wire format* — não de edição de texto.

## 1. O que é, em termos estruturais

Um workspace Cargo com 13 pacotes: 12 crates e `xtask` (`Cargo.toml:2-16`),
com `default-members = ["crates/calc-auy"]` (`Cargo.toml:18`). O grafo de
dependências aponta sempre para o núcleo (`docs/book/src/arquitetura.md:9-24`).

| Crate | Papel declarado | Evidência |
|---|---|---|
| `calc-auy-core` | racional híbrido, AST em arena, integridade, engine, lotes | `crates/calc-auy-core/src/lib.rs:8-32` |
| `calc-auy-parser` | lexer zero-copy + parser de precedência | `crates/calc-auy-parser/src/lib.rs`; `parser.rs:127` |
| `calc-auy-builder` | API fluente com *typestate* `Draft → Committed → Verified` | `crates/calc-auy-builder/src/calculation.rs:27-45` |
| `calc-auy-output` | decimal, monetário, LaTeX, Unicode, Mermaid, JSON multi-formato | `crates/calc-auy-output/src/output.rs:62-95` |
| `calc-auy-nlg` | SRL + trait `VerbalRealizer` | `docs/book/src/i18n.md:3-8` |
| `calc-auy-i18n` | 9 realizers (`Language` com 9 variantes) | `crates/calc-auy-i18n/src/lib.rs:73-93` |
| `calc-auy-processors` | Protobuf, CBOR, MessagePack sobre um `WireNode` único | `crates/calc-auy-processors/src/wire.rs:1-9` |
| `calc-auy-render` | HTML/KaTeX, SVG e PNG via `latex-rust` | `crates/calc-auy-render/src/html.rs:1-6`; `svg.rs:1-3` |
| `calc-auy-api` | fachada pública única | `crates/calc-auy-api/src/lib.rs:1-30` |
| `calc-auy-wasm` | `cdylib` com `WasmCalc` | `crates/calc-auy-wasm/src/lib.rs:1-9` |
| `calc-auy-macros` | `#[derive(Canonical)]` | `docs/book/src/arquitetura.md:9` |
| `calc-auy` | binário CLI (casca fina sobre a API) | `crates/calc-auy/src/main.rs:1-2` |
| `xtask` | tarefas de build/release/bench | `xtask/src/main.rs:39-56` |

## 2. O pipeline

```
texto ──lexer──▶ tokens (&str zero-copy)
      ──parser Pratt──▶ AstBuilder (arena, limites por invariante)
      ──finish──▶ FlatAst
      ──verify_custody──▶ VerifiedAst          (parse, don't validate)
      ──eval──▶ Rational exato
      ──rounding──▶ Rational arredondado
      ──SignaturePayload::new + sign(BLAKE3 keyed)──▶ Committed
      ──Output──▶ decimal | monetário | LaTeX | Unicode | Mermaid | verbal | JSON
      ──render (feature)──▶ HTML/KaTeX | SVG | PNG
```

Cada seta é um tipo, não uma convenção:

1. **Lexer** devolve `Token<'a>` com fatia `&str` da entrada original e `Span`
   em *bytes* (`crates/calc-auy-parser/src/lexer.rs:48-56`). Números aceitam
   `_`, `.` e expoente; a sequência de dígitos é consumida por byte
   (`lexer.rs:90-133`).
2. **Parser** é *precedence climbing* e emite **direto na arena**
   (`parser.rs:1`, `:127-145`). O *binding power* vem de
   `OperationType::precedence` (`parser.rs:37-45`, `ast/types.rs:86-94`), com
   `^` associativo à direita (`ast/types.rs:96-100`). Cadeias associativas
   (`add`, `mul`) são **achatadas** num único nó com N operandos
   (`parser.rs:147-192`), o que reduz profundidade e custo de avaliação.
3. **Builder** impõe os limites na construção, não no *call site*
   (`crates/calc-auy-core/src/ast/builder.rs:126-165`; ADR-0006).
4. **Custódia** produz o único tipo que o motor aceita (`custody.rs:13-20`,
   `:61-75`).
5. **Avaliação** é iterativa pós-ordem com pilha explícita — não recursiva —
   e memoização de DAG quando a arena tem subárvore compartilhada
   (`eval.rs:74-124`).
6. **Commit** avalia → arredonda → revalida bits → monta o payload → assina
   (`engine/commit.rs:63-78`).
7. **Saída** deriva sempre do `Committed` (nenhuma função aceita rascunho):
   `crates/calc-auy-output/src/output.rs:62-70`.
8. **Render** parte do **LaTeX canônico** de `Output::to_latex`
   (`crates/calc-auy-output/src/render.rs:1-6`, `:35-71`), garantindo que
   imagem e texto digam a mesma coisa.

### 2.1 Por que os estados intermediários existem

`Committed` tem campos privados e só é produzido por `commit`, que exige
`VerifiedAst` (`engine/commit.rs:13-22`, `:63`). `VerifiedAst.inner` é privado
e só é construído em `FlatAst::verify_custody` (`custody.rs:13-33`, `:61-75`).
O padrão é "parse, don't validate" levado ao tipo: um `Control` (elo de
custódia entre contextos) não pode ser avaliado sem que a assinatura anterior
tenha sido verificada por um `CustodyVerifier` fornecido pelo host
(`custody.rs:36-52`; ADR-0005).

## 3. Modelo de dados

### 3.1 Racional híbrido

`Rational { repr: Repr }` com duas representações e **a mesma semântica**
(`crates/calc-auy-core/src/numeric/rational.rs:20-36`):

- `Small { n: u64, d: u64 }` — caminho rápido, sem alocação;
- `Large { n: BigInt, d: BigInt }` — precisão arbitrária.

Invariantes em ambas: `d > 0` e `gcd(n, d) == 1` (`rational.rs:1-6`, `:20-24`).
`Add`/`Sub`/`Mul` promovem para `Large` em overflow em vez de falhar
(`rational.rs:98-114`); `Div`/`Rem` retornam `Result` (ADR-0001). Não existe
`f64` no domínio: o trait de ingestão `IntoRational` é **selado** e `f64`/`f32`
não o implementam (`crates/calc-auy-builder/src/into_rational.rs:7-14`,
`:62-88`), com teste de compilação que falha de propósito
(`crates/calc-auy-builder/tests/ui/f64_rejected.rs`).

### 3.2 Arena

`FlatAst` guarda `nodes: Vec<NodeData>`, `metadata: Vec<Arc<Metadata>>`,
`depths: Vec<usize>` e `root: NodeId` (`ast/flat.rs:74-90`); `Arc<FlatAst>`
clona em O(1) (`ast/flat.rs:78-81`). `NodeId` é `NonZeroU32` com `índice+1`,
de modo que `Option<NodeId>` usa o *niche* do zero e continua com 4 bytes
(`ast/types.rs:12-45`). Operações usam `SmallVec<[NodeId; 4]>`
(`ast/flat.rs:4-25`): até 4 operandos *inline*.

Quatro variantes de nó (`ast/flat.rs:12-45`):

```rust
Literal  { value: Rational, original_input: Box<str> }
Operation{ op: OperationType, operands: SmallVec<[NodeId; 4]> }
Group    { child: NodeId }
Control  { child, previous_context_label, previous_signature, previous_round_strategy }
```

`original_input` preserva o texto do usuário ("14.75%") para auditoria,
enquanto `value` guarda o racional exato — é a separação entre **o que foi
digitado** e **o que foi calculado**, usada depois para reconhecer raízes
(`output.rs:13-46`).

### 3.3 Metadados sem `f64`

`MetaValue` é enum fechado `Null|Bool|Int|Uint|Str|Array|Object` e **não tem
variante de ponto flutuante** (`crates/calc-auy-core/src/meta_value/mod.rs:12-31`;
ADR-0003). Consequências verificáveis: é `Eq + Hash` (derive na mesma linha) e
a canonicalização de objeto usa `BTreeMap` (`meta_value/mod.rs:28`), portanto
determinística.

## 4. Canonicalização e assinatura

### 4.1 Forma canônica

O trait `Canonical` escreve bytes em `Vec<u8>` (`canonical/mod.rs:16-19`); a
forma v1 é JSON compacto com **chaves em ordem de bytes UTF-8**, escrita à mão
por nó (`canonical/mod.rs:231-284`), não por `serde_json` com mapa ordenado.
`Rational` é serializado como `{"d":"...","n":"..."}` com numerador e
denominador em **string decimal** (`canonical/mod.rs:183-190`), eliminando
ambiguidade de representação numérica (ADR-0002).

Dois modos (`canonical/mod.rs:54-66`):

| Modo | Inclui metadados | Uso |
|---|---|---|
| `Trace` | sim | `trace_hash` / assinatura |
| `Value` | não | `value_hash` / chave de memoização |

O teste `value_hash_ignores_metadata` verifica que o metadado entra no trace e
não no valor (`canonical/mod.rs:369-397`). `write_json_string` faz escape
mínimo e hex de controles < 0x20 (`canonical/mod.rs:22-50`) — mesma política
que o `rich-editor-auy` quer para o wire do documento.

### 4.2 Assinatura

```
key  = blake3::derive_key("calc-auy/v1 signature", salt)
sig  = blake3::keyed_hash(key, canonical_bytes)
```

(`crates/calc-auy-core/src/integrity/sign.rs:9-21`). O comentário do código
explica a razão: `derive_key` elimina a ambiguidade de fronteira
`canonical ‖ salt` (`sign.rs:12-15`). `Salt` é `SecretString`, não implementa
`Debug` com conteúdo nem `Serialize`, e o buffer é zerado no drop
(`integrity/salt.rs:1-35`).

O payload assinado é `SignaturePayload { ast, final_result, round_strategy,
trace_version }` (`integrity/payload.rs:49-70`), com
`trace_version: NonZeroU16` (`payload.rs:19-40`). Existe `SignaturePayloadV2`
com `previous: Option<Digest>` para encadear elos (`payload.rs:94-110`), e
`AuditTrace` como envelope `{ payload, signature }` (`payload.rs:137-160`).

### 4.3 Determinismo: o que realmente é garantido

- Mesma entrada, mesmo sal, mesmo processo → mesma assinatura. Testado em
  `sign.rs:40-68` e na CLI por `params_op_sign_is_deterministic`
  (`crates/calc-auy/tests/cli.rs:134-140`), que compara 64 hex de duas
  execuções do binário.
- Canonicalização idempotente por construção (escrita manual, ordem fixa);
  teste `ast_canonical_is_deterministic` (`canonical/mod.rs:346-357`).
- O payload **não contém tempo nem aleatoriedade** (ADR-014); `ProblemDetails`
  usa `Clock` injetado e, com `deterministic: true`, a CLI usa `MockClock`
  (`crates/calc-auy/src/main.rs:65-72`), o que torna o `instance` do erro
  reprodutível (`tests/cli.rs:171-179`).
- **Não garantido**: assinatura entre *salts* diferentes (por desenho) nem
  entre versões de payload (v1 × v2 assinam bytes diferentes).

## 5. Custódia, typestate e *parse, don't validate*

`Calculation<S: State>` carrega o estado no parâmetro de tipo
(`crates/calc-auy-builder/src/calculation.rs:27-49`) e `PhantomData` não custa
memória (`:44-46`). Os métodos existem por estado:

| Estado | Métodos disponíveis | Evidência |
|---|---|---|
| `Draft` | `from`, `from_flat_ast`, `hydrate`, `add`/`sub`/`mul`/`div`/`pow`/`mod_`/`div_int`, `group`, `set_metadata`, `cross_context_add`, `commit`, `hibernate` | `calculation.rs:52-330` |
| `Committed` | `result`, `signature`, `payload`, `committed`, `output`, `to_json`, `verify` | `calculation.rs:332-393` |
| `Verified` | os mesmos acessores, sem promoção adicional | `calculation.rs:363-393` |

`commit` é o único caminho para `Committed` e chama `verify_custody` antes de
avaliar (`calculation.rs:267-282`). `hibernate` serializa a AST em JSON antes
do selo (`calculation.rs:300-306`) e `hydrate` a reconstrói com limites
(`calculation.rs:87-92`), fechando o ciclo de persistência sem aceitar AST
inválida.

Os estados inválidos são impedidos em **compilação**: há testes `trybuild`
(`crates/calc-auy-builder/tests/compile_fail.rs`) com casos
`ui/draft_has_no_result.rs` (chamar `result()` num `Draft` não compila) e
`ui/f64_rejected.rs` (`f64` não entra).

## 6. Limites como invariantes

| Limite | Padrão | Onde é imposto |
|---|---:|---|
| `max_bits` | 1 000 000 | avaliação/commit (`engine/commit.rs:71-72`), parsing (`numeric/parse.rs:161-166`), `IntoRational` (`into_rational.rs:29-33`) |
| `max_depth` | 500 | `AstBuilder` (`ast/builder.rs:126-165`), `verify_custody` (`custody.rs:62-69`) |
| `max_nodes` | 1000 | `verify_custody` (`custody.rs:62-69`), desserialização |
| `max_operands` | 100 | `AstBuilder::operation` (ADR-0006; `limits.rs:132-147`) |
| `max_metadata_bytes` | 16 384 | `insert_metadata` (`ast/builder.rs:349`) |

Fonte única: `Limits::default()` (`crates/calc-auy-core/src/limits.rs:34-45`),
com `Limits::new` recusando zero (`limits.rs:47-80`) e um `check_*` por limite
(`limits.rs:82-192`).

O ponto mais forte é a **desserialização com orçamento durante a leitura**:
`FlatAst::from_json_str` usa um `DeserializeSeed` (`AstSeed`,
`ast/wire.rs:172-183`, `:217`) que impõe profundidade, nós, operandos e bytes
de metadata *enquanto* o `serde` lê, abortando payloads hostis cedo
(`ast/wire.rs:1-7`). Cada nó aceita apenas campos conhecidos
(`LITERAL_FIELDS`/`OPERATION_FIELDS`/…, `ast/wire.rs:22-33`) e campo
desconhecido é erro (`ast/wire.rs:342`).

## 7. Erros e diagnóstico

### 7.1 Modelo de erro

`CalcAuyError` é um enum `thiserror` com variantes de domínio:
`DivisionByZero`, `IrrationalResult`, `MathOverflow`, `LimitExceeded{kind}`,
`IntegrityViolation{expected,received}`, `CustodyViolation{reason}`,
`InvalidInput`, `ParseError`, `SerializationError`
(`crates/calc-auy-core/src/error.rs:34-103`). A conversão para **RFC 7807** é
centralizada em `to_problem_details(&dyn Clock)`
(`error.rs:109-160`), com `type` URI estável (`https://calc-auy.dev/errors/...`),
`title`, `status` HTTP, `detail`, `instance: Uuid` e `context: BTreeMap`
(`error.rs:109-146`; ADR-015).

### 7.2 Códigos de saída (CLI)

| Classe | Código | Evidência |
|---|---:|---|
| parse | 3 | `crates/calc-auy/src/error.rs:29-37` |
| limite/overflow | 4 | idem |
| divisão por zero | 5 | idem; testado em `tests/cli.rs:154-159` |
| integridade/custódia | 6 | idem |
| irracional | 7 | idem |
| erro de contexto/IO/uso | 1 ou 2 | `main.rs:33-44` (2), `:55-63` (2 e 1) |

`--params` inválido e `--batch` + `--params` devolvem 2 com mensagem em
`stderr` (`main.rs:33-44`, `:46-72`; testes `tests/cli.rs:191-204`). Com
`error_format: json`, o `ProblemDetails` vai para **stdout** e o código de
saída continua o de domínio (`crates/calc-auy/src/error.rs:41-59`;
`tests/cli.rs:161-169`).

### 7.3 Diagnóstico transversal

O módulo `diag` define: log estruturado como **id estável + campos tipados**
(nunca texto interpolado), custo zero sem a feature `instrument`, e ligação em
runtime (`CALC_AUY_INSTRUMENT`) (`crates/calc-auy-core/src/diag/mod.rs:1-10`).
`Span` é no-op sem a feature (`diag/mod.rs:126-133`). Há também
`fingerprint` e `redact` (`diag/mod.rs:11-12`); o `redact` de `diag` trata
segredos de log e **não** é usado pelo Mermaid, que tem política própria de
PII (`crates/calc-auy-output/src/mermaid.rs:17-31`;
`docs/book/src/renderizacao.md:55-58`).

**Limite honesto**: `SystemClock::now_rfc3339` devolve um timestamp **fixo**
com um `TODO` explícito (`crates/calc-auy-core/src/clock/mod.rs:25-34`).
Portanto "tempo real" em produção **não** está implementado; só o eixo
determinístico (`MockClock`) está.

## 8. Formatos e emissão

Todos os formatos derivam de `Committed` (`output.rs:62-70`) e são memoizados
por instância via `OnceLock` (`output.rs:74-95`): LaTeX, Unicode, Mermaid,
monetário, rastro de auditoria, decimal efetivo, inteiro escalado e SRL.

| Saída | Garantia observável | Evidência |
|---|---|---|
| decimal | casas fixas, arredondamento aplicado antes de truncar | `decimal.rs:63-75`; teste `:111-123` |
| escalado | inteiro `round(x·10^d)` | `decimal.rs:79-90` |
| monetário | separador de milhar e posição de símbolo vêm de `LocaleData` | `monetary.rs:66-84` |
| LaTeX/Unicode | raízes fracionárias viram `\sqrt`/`√` | `output.rs:13-46`; `docs/book/src/renderizacao.md:55-59` |
| verbal | SRL + realizer por idioma | `output.rs:219-238` |
| JSON multi-formato | subconjunto de `OutputKey`, `signature` sempre presente, chaves em `BTreeMap` | `json.rs:112-140` |
| HTML | fragmento auto-contido com CSS embutido e `aria-label` | `html.rs:138-181` |
| SVG/PNG | documento/SVG via `latex-rust`, determinístico | `svg.rs:88-99`; teste `crates/calc-auy-render/tests/render.rs:41-45` |

Determinismo do HTML: o CSS é `include_str!` de um asset fixo
(`html.rs:15-18`), o contexto KaTeX é `OnceLock`
(`html.rs:81-85`), e a ordem dos atributos é literal no `format!`
(`html.rs:152-181`).

## 9. Acessibilidade

O que existe:

- **HTML**: `aria-label` recebe a **descrição verbal** do cálculo, gerada pela
  camada NLG (`crates/calc-auy-output/src/render.rs:20-45`; `html.rs:138-158`), escapada
  para atributo (`html.rs:87-101`) e testada
  (`crates/calc-auy-output/tests/render.rs:13-22`).
- **Verbal**: SRL independente de idioma + 9 realizers; a mesma descrição
  serve para leitores de tela e para o campo `verbal` do JSON
  (`crates/calc-auy-i18n/src/lib.rs:73-93`; `json.rs:28-40`).
- **Classes**: `container_class` só aceita `[A-Za-z0-9_-]`; tentativa de
  injeção é descartada (`html.rs:103-118`; teste `:210-214`).

O que **não** existe: `role`, `<title>`/`<desc>` no SVG, `alt` em PNG, ou
qualquer árvore semântica para o SVG. `to_svg` devolve o que `latex-rust`
produz (`svg.rs:88-96`) e nenhum dos testes de SVG verifica rótulo
(`crates/calc-auy-render/tests/render.rs:24-28`). A afirmação do livro de que
a saída HTML é "acessível" (`docs/book/src/renderizacao.md:8-11`) vale só para
o modo HTML.

## 10. Superfícies de integração

### 10.1 CLI

Convenção: **um único posicional é o `BODY`** (padrão stdin, aceita heredoc);
toda configuração em `--params='{...}'`
(`crates/calc-auy/src/cli.rs:1-40`). Subcomandos: `verify`, `hydrate`, `repl`,
`schema`, `man --completions <shell>` (`cli.rs:42-67`). Precedência de
configuração: `--params` > ambiente > arquivo `chave = valor` apontado por
`CALC_AUY_CONFIG` > padrões (`crates/calc-auy/src/config.rs:1-5`,
`:112-159`, parser do arquivo em `:221-238`).

Campo desconhecido em `--params` é **recusado** (`deny_unknown_fields`,
`config.rs:58`; teste `tests/cli.rs:198-204`) — nada de configuração
silenciosamente ignorada.

Lote: NDJSON `{"id"?, "body", "params"?}` na entrada, mesma ordem na saída,
`rayon` entre linhas, canal limitado (`CALC_AUY_BATCH_BUFFER`, padrão 256)
(`crates/calc-auy/src/batch.rs:1-81`; teste `tests/cli.rs:205-225`).

### 10.1.1 Introspecção parcial e drift de documentação

O subcomando `schema` devolve uma lista **fixa em código**
(`crates/calc-auy/src/commands.rs:316-331`) cujo campo `modes` tem 7 itens e
**omite `json`, `html` e `svg`**, embora `FormatMode` os aceite
(`config.rs:32-54`) e o binário os execute (saída real em
`exemplos/README.md` §3.8). Quem gerar UI a partir de `schema` perde
três modos. É o tipo de divergência que um schema gerado (como o
`gen_spec --check` do `html-auy`) impediria.

Segundo drift real: o livro mostra a saída verbal japonesa com
`丸め：nbr5891` (`docs/book/src/i18n.md:12-14`), mas o código usa o rótulo de
exibição (`as_display`) e imprime `丸め：NBR-5891`
(`crates/calc-auy-core/src/numeric/rounding.rs:43-55`; saída observada em
`exemplos/README.md` §3).

### 10.2 Biblioteca

`calc_auy_api` reexporta tudo e adiciona `CalcAuy`, que guarda
`InstanceConfig` e cria cálculos que a tomam emprestada
(`crates/calc-auy-api/src/lib.rs:13-22`, `:38-95`). É a "fachada única"
(ADR-0008).

### 10.3 WASM

`WasmCalc` expõe `parse/from/hydrate/commit/verify/format/monetary`, com
`HostVerifier` de elos registrados pelo host e erros convertidos em
`ProblemDetails` JS (`crates/calc-auy-wasm/src/lib.rs:1-9`, `:36-60`,
`:120-175`, `:385-388`). Números grandes trafegam como string (ADR-012).

### 10.4 Processadores binários

Os quatro codecs compartilham **um** modelo wire recursivo
(`WireNode`, `crates/calc-auy-processors/src/wire.rs:10-66`) e reconstroem a
arena via `AstBuilder` reaplicando limites (`crates/calc-auy-processors/src/wire.rs:86-105`). Schema
versionado em `schema/calc_auy.cddl`, `.proto` e `.schema.json`, com política
"adicionar campos é OK; renomear/remover exige major" (ADR-013;
`schema/calc_auy.cddl:1-3`).

O `.proto` é compilado pelo *build script*
(`crates/calc-auy-processors/build.rs:11-22`) e o código prost gerado é
incluído em `crates/calc-auy-processors/src/lib.rs:31-32`; `.cddl` e
`.schema.json` saem das constantes de
`crates/calc-auy-processors/src/schema.rs` por `xtask schema`
(`xtask/src/main.rs:435-447`). O teste de sincronia
(`crates/calc-auy-processors/tests/schema.rs:4-9`) compara só esses dois
arquivos com as constantes: o `.proto` **não** tem teste de paridade com
`schema.rs`, então as duas descrições podem divergir sem o CI acusar.

## 11. Testes

- **311** testes em 30 alvos de unidade/integração (soma dos `test result`
  não-doc da execução de `cargo test --workspace`) + **11** doctests =
  **322**; a contagem estática de `#[test]` também resulta em 311
  (`grep -rhoE '^\s*#\[test\]' --include=*.rs crates xtask | wc -l`).
- **Corpus de conformidade** com 32 vetores em 5 seções
  (`crates/calc-auy-api/tests/corpus/vectors.json`: `arithmetic` 9,
  `rounding` 14, `pow` 4, `canonical` 3, `limits` 2), executado por
  `crates/calc-auy-api/tests/corpus.rs` com digest BLAKE3 do trace.
- **Diferencial** contra um oráculo independente `BigInt` (`engine/differential.rs:1-7`,
  `:186-200`), 512 casos por `proptest`, mais um soak de 20 000 reavaliações
  verificando determinismo (`:202-225`).
- **Compilação negativa** (`trybuild`): `builder/tests/ui/draft_has_no_result.rs`,
  `ui/f64_rejected.rs`, `core/tests/ui/canonical_rejects_f64.rs`.
- **CLI**: 21 testes de integração cobrindo stdin, modos, exit codes, batch,
  schema, man e determinismo (`crates/calc-auy/tests/cli.rs`).
- **Fuzz**: 3 alvos (`parse`, `canonicalize`, `deserialize`) em
  `fuzz/fuzz_targets/`; `cargo-fuzz` **não** está disponível no ambiente, e o
  `Makefile:45-48` registra isso com fallback para proptests.
- **CI**: 8 jobs — fmt, clippy `-D warnings`, test (incluindo corpus), docs
  (`RUSTDOCFLAGS=-D warnings` + doctests), build WASM, `cargo-deny`,
  `cargo-audit`, Miri **não bloqueante** (`.github/workflows/ci.yml:11-95`).

O resultado da execução de `cargo test --workspace` neste dossiê está em
`exemplos/README.md` §1 — inclusive a observação de que a toolchain 1.98
fixada em `rust-toolchain.toml:2` não tinha `cargo` no ambiente local e o
build foi feito com 1.99.

## 12. Performance

Desenho, não medição:

- **Arena** em vez de `Arc<Node>` por nó; `NodeId` de 4 bytes
  (`ast/types.rs:12-45`; `ast/flat.rs:74-90`).
- **Avaliação iterativa** com pilha explícita (`eval.rs:91-198`), sem
  recursão; memoização de DAG só quando há subárvore compartilhada
  (`eval.rs:74-90`).
- **Cache de avaliação** opcional (`engine/cache.rs:13`; `eval_memoized`,
  `eval.rs:288`).
- **Saída memoizada** por `OnceLock` (`output.rs:75-95`).
- **Lote SoA** com `Vec<u64>` paralelos e tiling L1/L2
  (`batch/soa.rs:1-8`, `:33-45`; `DEFAULT_TILE` reexportado em
  `crates/calc-auy-core/src/lib.rs:21`; ADR-010).
- **Paralelismo determinístico**: `eval_batch` indexa o resultado pela posição
  de entrada, nunca pela ordem de conclusão (`parallel.rs:1-25`).
- **Cache quente de parsing** limitado a 256 entradas ≤ 64 bytes, com
  revalidação de limites no *hit* (`numeric/parse.rs:19-66`; teste
  `:206-220`).

Medição declarada pelo projeto (não reproduzida por mim): speedup geométrico
~6,9× (depois ~7,8×, segundo o changelog) contra o rascunho TypeScript em 62
workloads, com metodologia e hardware em `bench/README.md:1-60` e resultados
em `plan/evidence/bench/compare.md`. É evidência do projeto, não deste dossiê.

## 13. Limites, riscos e o que não existe

| Fato | Consequência | Evidência |
|---|---|---|
| `rust-toolchain.toml` fixa 1.98 | o ambiente local não tinha `cargo` nessa toolchain; build exigiu 1.99 | `rust-toolchain.toml:1-3`; observado em `exemplos/README.md` |
| `SystemClock` devolve tempo fixo | auditoria com timestamp de produção não funciona | `clock/mod.rs:22-34` |
| `publish = false` em todos os pacotes | não há release publicada; consumo é por caminho/git | `Cargo.toml:25` (workspace.package) |
| Sem servidor/HTTP/MCP | não há superfície de protocolo; `ProblemDetails` é modelo, não endpoint | busca: nenhum `axum`/`hyper`/`mcp` em `crates/` |
| SVG/PNG sem rótulo acessível | a11y só no caminho HTML | `svg.rs:88-96` |
| Sem importação de expressão estruturada de terceiros | a entrada é sempre texto; não há parser de JSON→AST público (só `hydrate` do próprio wire) | `ast/wire.rs:172-183` |
| Assinatura simétrica (`BLAKE3 keyed`) | quem tem o sal pode forjar; não há chave pública | `integrity/sign.rs:16-21` |
| `f64` proibido por tipo | interoperar com fontes que só têm `float` exige string/racional na borda | `into_rational.rs:7-14` |

## 14. O que isso ensina ao `rich-editor-auy`

O encaixe conceitual (não de código de diagrama) está em `ideias.md`. Em uma
frase: o projeto demonstra que **um pipeline com tipos de estado, bytes
canônicos e limites como invariantes** pode ser pequeno, testável e
determinístico — exatamente a tese de `plan/00_panorama.md`, aplicada a um
domínio mais simples.
