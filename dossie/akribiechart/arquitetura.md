# akribie — arquitetura

> Referências no formato `caminho:linha`, relativas a `ref/akribiechart/`.
> O que foi verificado por execução está marcado **[E]**; por leitura de código **[C]**; por teste
> **[T]**; por documentação do próprio repo **[D]**. Números de linha conferem com o commit
> `7275fedfe81f2d4ac1df4e92a580c05b90a81049`.
>
> Correção de propósito: `docs/plan/00-panorama.md:39-40` e `docs/specs/DETERMINISM.md:32` falam de
> portas `Env`/`Logger` que **não existem mais** — foram removidas em 0.1.3 e o próprio código diz
> isso (`crates/akribie-core/src/ports/mod.rs:5-6`). Este documento segue o código.

---

## 1. Identidade, escopo e forma

O akribie é um **compilador** com uma linguagem de entrada própria (`.akrc`, versão `v1`) e cinco
saídas derivadas. Não há estado global, servidor, banco, daemon, thread no domínio nem rede em tempo
de execução (`docs/adr/ADR-0001-simplicidade-determinismo-di.md:26-39`).

O desenho central em uma frase: **a fonte é texto; tudo o mais é artefato de build**
(`docs/adr/ADR-0002-fonte-texto-canonica.md:24-27`).

| artefato | contrato de bytes | onde |
|---|---|---|
| `.akrc` (fonte) | forma canônica única, idempotente | `crates/akribie-core/src/fmt.rs:19-47` |
| SVG | **byte a byte** entre execuções e versões | `crates/akribie-core/src/emit/svg.rs:20-83` |
| `scene` JSON | **byte a byte** | `crates/akribie-core/src/emit/scene_json.rs:18-36` |
| `.excalidraw` | **byte a byte** | `crates/akribie-core/src/emit/excalidraw.rs:22` |
| PNG/JPG/AVIF | determinístico **na versão**, não entre versões | `crates/akribie-core/src/emit/raster.rs:50-60` |
| `akr prime` | golden por versão | `crates/akribie-cli/tests/prime.rs:8-30` |

## 2. Workspace e grafo de dependências

Quatro crates membro + uma ferramenta de build fora do lockfile (`Cargo.toml:3-12`).

| crate | papel | I/O | linhas em `src/` |
|---|---|---|---|
| `akribie-core` | domínio puro: `lex → parse → resolve → layout → paint → emit` | **nenhum** | 10 355 |
| `akribie-cli` | binário `akr`: envelope JSON, comandos, apresentação, exit codes | arquivo, `git`, `stdout` | 2 982 |
| `akribie-mcp` | servidor MCP JSON-RPC 2.0 sobre `stdin`/`stdout` | `stdin`/`stdout`, arquivo | 468 |
| `akribie-adapters` | implementações reais das portas: `StdFs`, `StdGit` | arquivo, `git` | 93 |
| `xtask` | `woff2 → ttf`; lock próprio, **excluído** do workspace de propósito | rede (uma vez) | fora |

Dependências diretas do domínio (`crates/akribie-core/Cargo.toml:18-28`):

| dep | para quê | nota |
|---|---|---|
| `serde`/`serde_json` | `scene`, `.excalidraw`, envelope | o core **não** é `no_std`; `serde_json` está no caminho |
| `indexmap` | ordem de declaração preservada | substitui `HashMap` por determinismo |
| `unicode-segmentation` | quebra de rótulo por grafema | `text::Gauge::wrap` |
| `harfrust` + `dugong` | shaping de texto e layout em camadas (dagre) | os dois maiores custos |
| `resvg`/`usvg`/`tiny-skia`/`image` | **opcionais** (`feature = "raster"`) | ver §13.1: a feature não funciona |

`cargo tree -p akribie-core --no-default-features` não contém `tokio`, `async-std` nem `rayon`
(`Makefile:32-38`) — o que é verdade por construção, mas veja §13.1: esse alvo não compila.

## 3. Portas (arquitetura hexagonal)

O domínio não toca o sistema operacional. Duas traits, só:

```rust
// crates/akribie-core/src/ports/mod.rs:12-43
pub trait Fs  { fn read(..); fn read_bytes(..); fn write(..); fn exists(..); }
pub trait Git { fn show(&self, rev: &str, path: &Path) -> Result<String>; }
```

- Adaptadores reais em `crates/akribie-adapters` (`StdFs`, `StdGit`); apenas `akribie-cli` e
  `akribie-mcp` os montam (ADR-0006 §3, `docs/specs/ARCHITECTURE.md:10-21`).
- Fakes em `crates/akribie-core/src/ports/fakes.rs` (`MemFs`, `FakeGit`); "todo teste de domínio usa
  fakes e nenhum toca o disco" (`docs/specs/ARCHITECTURE.md:23-24`).
- **Porta sem consumidor não existe**: `Env` e `Logger` foram removidas
  (`crates/akribie-core/src/ports/mod.rs:5-6`). Regra boa de se copiar.

Observação para o nosso caso: a porta `Git` existe para um único verbo (`--at REV`), que lê
`git show REV:FILE`. É a única dependência de processo no produto.

## 4. Pipeline

```
bytes ──lex──▶ tokens ──parse──▶ Document ──resolve──▶ Chart ──layout──▶ Placed ──paint──▶ Scene
                                                                                    │
                                            emit/svg · scene_json · excalidraw · raster
```

(`docs/specs/ARCHITECTURE.md:27-43`; `crates/akribie-core/src/lib.rs:9-15`.)

A composição de `check` é literalmente seis chamadas puras numa função de 14 linhas
(`crates/akribie-core/src/pipeline.rs:27-40`):

```rust
pub fn check(source: &str, config: &Config, font: &Font) -> CheckReport {
    let (document, mut diagnostics) = parse::parse(source);
    let (chart, resolved)  = resolve(&document, config);
    diagnostics.extend(resolved);
    diagnostics.extend(isolated(&chart));        // W002
    diagnostics.extend(missing_glyphs(&chart, font)); // W003
    diagnostics.extend(a11y::check_contrast(&chart)); // W007
    diag::sort(&mut diagnostics);
    CheckReport { document, chart, diagnostics }
}
```

Duas decisões que valem imitar:

1. **A validação semântica é uma lista de funções independentes que devolvem diagnósticos**, não um
   passe monolítico. Cada regra (`W002`, `W003`, `W007`) é testável sozinha.
2. **Ordenação determinística por `(linha, coluna, código)`** no fim
   (`crates/akribie-core/src/diag.rs:167-172`). Modelo para o nosso `Diagnostics`.

### 4.1 `lex` — tokens com posição

`crates/akribie-core/src/lex.rs:41-57`. Uma linha por vez; sete espécies de token
(`Ident`, `Version`, `Str`, `Key`, `Bare`, `Arrow`, `Pair`, `lex.rs:10-27`).

Decisão central: **`chave=valor` é um token indivisível** — o `=` não é separador
(`lex.rs:1-5`, `lex.rs:59-95`). Consequência prática: `layout = flow` é erro; `layout=flow` é válido.
E um token que **abre** com aspas é uma string inteira, então `=` dentro dela é conteúdo —
`node a "user=admin"` funciona (bug corrigido em 0.1.3, `CHANGELOG.md:46-48`).

Posição é `Span { line, col }`, ambos **1-based**, com **coluna em caracteres**, não em bytes — para
que a mensagem aponte o caractere que o autor vê (`crates/akribie-core/src/span.rs:5-13`).

Recuperação: erro por linha. Uma linha inválida gera um diagnóstico e as demais continuam sendo
lidas (`docs/specs/GRAMMAR.md:3-5`).

### 4.2 `parse` — `Document` sintático

`crates/akribie-core/src/parse/mod.rs:33` (função), `:17` (`Document`). O `Document` guarda os
statements **como escritos**: `chart`, `groups`, `nodes`, `edges`, `notes`, comentários atrelados a
cada um, e `trailing_comments`. É o que `fmt` consome — o parser preserva o que o formatador precisa
para não perder comentários.

`parse::stmt` valida por tabela: as chaves, tipos, defaults, faixas e conjuntos fechados vêm **de uma
única fonte** (`crates/akribie-core/src/syntax/specs.rs`), e o parser não conhece nenhuma chave
inline (`crates/akribie-core/src/syntax/mod.rs:1-5`). É o inverso do "schema espalhado".

### 4.3 `resolve` — `Document → Chart`

`crates/akribie-core/src/resolve/mod.rs:34`. Resolve id → objeto, aplica `akribie.toml` onde o
`.akrc` não declara, materializa nós implícitos (`W001`) e emite `E004` (id duplicado), `E005`
(referência ausente), `E006` (`layout=free` sem `at`).

O resultado, `Chart`, é **tipado e fechado**: nenhuma estrutura guarda opções cruas. Cor é `Token`
(enum), forma é `Shape` (enum), tamanho é `f32` já validado (`crates/akribie-core/src/model/mod.rs:1-47`).
O `Chart` carrega também o `Span` de cada elemento — é por isso que um erro de contraste (`W007`) sabe
apontar a linha do nó.

### 4.4 `layout` — `Chart → Placed`

`crates/akribie-core/src/layout/mod.rs:150-179`. Seis perfis, escolhidos por `chart.layout`:

| perfil | algoritmo | arquivo |
|---|---|---|
| `flow` | dagre (crate `dugong`), rota de aresta própria | `layout/flow.rs:19-74` |
| `tree` | hierarquia | `layout/tree.rs` |
| `mindmap` | radial | `layout/radial.rs` |
| `grid` / `stack` | grade / coluna | `layout/grid.rs` |
| `free` | posições declaradas (`at=`) | `layout/free.rs` |

Três detalhes que importam:

- **Uma única função mede a caixa segura do texto por forma** (`layout/inset.rs`), e é a mesma que
  dimensiona o nó (`docs/specs/ARCHITECTURE.md:35-38`). Um `diamond` cresce até o texto caber dentro
  do contorno, não do retângulo envolvente (`layout/mod.rs:160-172`).
- **`at` (pino) vale em qualquer perfil**, mas é local: fixar um nó não congela os vizinhos
  (`layout/mod.rs:158-165`).
- **O rótulo de aresta entra no dagre como `EdgeLabel` com largura/altura**, senão nasce em cima do nó
  da camada seguinte (`layout/flow.rs:40-54`).

`Placed` é geometria pura sem cor (`layout/mod.rs:127-146`): `rect`, `route: Vec<Point>`,
`label_lines`, `label_rect`. Nenhuma decisão de cor acontece aqui.

### 4.5 `paint` — `Placed + tema + fonte → Scene`

`crates/akribie-core/src/paint/mod.rs:168`. Resolve a matriz **forma × fillstyle** (hachure,
cross-hatch, zigzag, solid) com `clipPath`, escolhe o acabamento (`sketch::Ink::Rough` para traço à
mão, `::Exact` para técnico) e resolve tokens de cor para hex.

`Scene` (`paint/mod.rs:144-165`) é a **última estrutura antes dos bytes**: formas já em atributo `d`,
texto já quebrado, cores já em `&'static str`. Um emissor não precisa saber o que é `Token`, `Shape`
ou `Curve` — só desenhar. É a mesma separação que um editor precisa entre "modelo" e "view data".

Detalhe de tipografia/legibilidade: o fundo do rótulo de aresta é desenhado **depois** do traço, para
interromper a linha, e com folga `LABEL_PAD = 6.0` (`paint/mod.rs:18-19`, `emit/svg.rs:156-170`).

### 4.6 `emit` — bytes

Quatro emissores, uma `Scene` ou um `Placed`:

| emissor | entrada | saída | determinismo |
|---|---|---|---|
| `svg::svg` | `&Scene, &Theme` | `String` | byte a byte |
| `scene_json::scene_json` | `&Placed, &Chart` | `String` | byte a byte |
| `excalidraw::excalidraw` | `&Chart, &Placed` | `String` | byte a byte |
| `raster::encode` | `&str (svg), RasterFormat, f32` | `Vec<u8>` | na versão |

O emissor SVG mostra o contrato inteiro em ~60 linhas (`crates/akribie-core/src/emit/svg.rs:20-83`):

- `<title>` e `<desc>` são os **primeiros** filhos; `aria-labelledby` aponta para os dois;
- seções `groups`, `edges`, `nodes`, `notes`, nesta ordem, cada uma só se não vazia;
- todo número passa por `num::format`; todo texto por `escape` (5 entidades);
- acumula em `Vec<String>` e fecha com `join("\n")` — e o doc-comment registra que a alternativa
  (escrever num `String` pré-dimensionado) foi **medida e empatou**
  (`emit/svg.rs:10-13`). Esse tipo de nota é ouro: evita "otimização" futura sem ganho.

O `scene` JSON deliberadamente **não carrega o `d`** dos paths: caminho é derivável e ruidoso; o que
interessa para depurar é caixa, rota em pontos, cor e rótulo (`emit/scene_json.rs:1-6`).

## 5. Modelo de dados em quatro camadas

O akribie tem quatro estruturas, cada uma com uma responsabilidade e nada além:

| camada | tipo | o que guarda | quem consome |
|---|---|---|---|
| sintaxe | `Document` | statements como escritos + comentários | `fmt`, relatório de `check` |
| semântica | `Chart` | nós/arestas/grupos/notas resolvidos, tipados, com `Span` | `layout`, `diff`, `a11y`, `excalidraw` |
| geometria | `Placed` | `rect`, `route`, `label_lines` — sem cor | `paint`, `scene_json` |
| pintura | `Scene` | `d`, hex, `TextRun` — pronto para bytes | `emit/svg`, `raster` |

Ganho: cada estágio é testável com um valor e não precisa reconstruir o anterior. Custo: quatro
estruturas para manter em sincronia; compensado porque cada transição é uma função pura e testada.

Detalhe relevante para nós: o `Chart` mantém **a ordem de declaração** (`Vec`, não mapa), e busca por
id é linear (`crates/akribie-core/src/model/mod.rs:151-174`). Para 1000 nós isso é O(n) por lookup;
aceitável porque o lookup acontece em `layout`, uma vez por elemento, sobre listas pequenas
(`finish.rs:40-45` constrói um `IndexMap` de id → índice para as rotas). **Isto não escala como
"banco de dados de conteúdo"** — e é a diferença mais importante em relação ao que precisamos.

## 6. Determinismo

Contrato: mesma entrada + mesma versão ⇒ mesmos bytes de SVG, `scene` e `.excalidraw`
(`docs/specs/DETERMINISM.md:5-14`). Verificado em três formatos e no raster em execução própria
([`README.md`](README.md) §3.2).

Os mecanismos, um por um, com o código:

| ameaça | proibição | substituto | onde |
|---|---|---|---|
| relógio | `SystemTime::now`, `Instant::now` | nada — `updated`/`created`/`versionNonce` = `0` | `emit/excalidraw.rs:3` |
| RNG | `rand`, `thread_rng` | FNV-1a 32 bits do **id** do elemento | `sketch/rng.rs:58` |
| ordem de mapa | `HashMap`/`HashSet` | `IndexMap`, `Vec`, `BTreeMap` | `layout/finish.rs:40-45` |
| float impresso | `f32::to_string` | `num::format` (2 casas, sem zeros, sem `-0`) | `num.rs:1-25` |
| fontes do sistema | `fontdb` do SO | `include_bytes!` das duas OFL | `text.rs:29-31` |
| paralelismo no encoder | pool padrão | `num_threads(1)` | `emit/raster.rs:89` |

`num::format` merece destaque: `(value * 100.0).round() / 100.0` e depois `{:.2}` com poda de zeros,
mais o caso `-0` → `0` (`num.rs:8-25`). É três linhas e resolve o problema de "o golden quebra porque
a std mudou a formatação".

**O que não é determinístico entre versões**: geometria de layout e escolha de rota podem melhorar,
então o SVG muda *com a versão*; codecs de raster mudam bytes entre versões. Está declarado
(`docs/specs/DETERMINISM.md:32-39`) — isto é honestidade de contrato, não bug.

Testes que prendem isso (`[T]`, lidos):
`emit/svg/tests.rs:22`, `emit/scene_json/tests.rs:21`, `emit/excalidraw/tests.rs:23`,
`emit/raster.rs:151`, `layout/tests.rs:194`, `sketch/tests.rs:15`,
`akribie-cli/tests/render.rs:26` (processo real, dois `render`), `tests/render.rs:473` (goldens dos
quatro diagramas reais, `crates/akribie-cli/tests/golden/reais/`).

## 7. Acessibilidade

Adr: `docs/adr/ADR-0005-acessibilidade-svg.md`. Spec: `docs/specs/A11Y.md`.
Regra de ouro: **a descrição nasce do modelo, nunca da geometria**
(`crates/akribie-core/src/a11y.rs:1-4`).

Estrutura emitida (`emit/svg.rs:20-83`, verificada com `xmllint --noout` **[E]**):

```xml
<svg xmlns="…" width="…" height="…" viewBox="…" role="img"
     aria-labelledby="akr-t akr-d" lang="pt-BR" focusable="false">
<title id="akr-t">Política de retry</title>
<desc id="akr-d">3 nós, 3 arestas. Nós: API Gateway; Retry ×3; DLQ.
Arestas: API Gateway → Retry ×3 (5xx); Retry ×3 → API Gateway (backoff); Retry ×3 → DLQ (esgotado).</desc>
…
<g data-id="api-&gt;retry#0" data-kind="edge" aria-label="5xx">…</g>
```

Decisões transferíveis:

1. **`<desc>` é derivado do modelo** (`a11y.rs:16-42`): contagem + lista de rótulos + lista de
   relações `from → to (label)`. Um agente responde "quais são as arestas?" lendo o `<desc>`, sem
   parsear geometria. O ADR rejeita explicitamente "`<desc>` gerado da geometria"
   (`docs/adr/ADR-0005-acessibilidade-svg.md:82-85`).
2. **`aria-label` por figura, igual ao rótulo, omitido quando não há rótulo**
   (`emit/svg.rs:174-183`).
3. **Ids internos com prefixo `akr-`** para sobreviver a SVG embutido em HTML (`docs/specs/A11Y.md:26`).
4. **Contraste é auditado por código, não por disciplina**: `check_contrast` percorre nós, grupos e
   arestas e emite `W007` (`a11y.rs:58-136`). Mínimos: 4,5:1 texto, 3:1 traço
   (`crates/akribie-core/src/theme/mod.rs:32-35`). A paleta é fechada e o teste percorre **todas** as
   combinações (`theme/tests.rs`, `theme::tests::text_meets_aa_on_every_fill` — passou **[E]**).
5. **Cor nunca é o único canal**: `W007` avisa, mas o desenho continua legível porque toda relação tem
   rótulo textual e o `<desc>` lista tudo (`docs/specs/A11Y.md:33-36`).
6. **Diagnóstico auto-suficiente e independente de terminal**: sem cor, sem tabela alinhada, funciona
   em pipe (ADR-0005 "Diagnóstico é auto-suficiente").

**Lacunas reais de a11y** (verificadas, não inferidas):

- **`<desc>` é português fixo.** `a11y::describe` tem "nós"/"arestas" hardcoded
  (`a11y.rs:30-41`). Com `chart … lang=en`, o SVG declara `lang="en"` e descreve em português **[E]**.
  Um leitor de tela vai pronunciar português com fonética inglesa.
- **O SVG não é focável nem navegável.** `focusable="false"` é o oposto de navegável; não há
  `tabindex`, nem `role="list"` em `#nodes` apesar de `docs/specs/A11Y.md:24` afirmar que existe —
  `emit/svg.rs:85-93` emite `<g id="nodes">` **sem** `role="list"`, e `node_element` não emite
  `role="listitem"`. **O spec e o emissor divergem.** Para um diagrama estático isso é aceitável; para
  um documento editável seria inaceitável.
- **Sem `aria-describedby` para o detalhe**, sem `aria-live`, sem estado.

Nota para o nosso caso: o akribie prova o **mecanismo** (descrição derivada do modelo + auditoria de
contraste como diagnóstico), não o **nível** de a11y de uma superfície editável. Não há `contenteditable`,
não há foco, não há teclado — logo não há o que copiar de "editor acessível" aqui.

## 8. Formatos de emissão, um por um

### 8.1 SVG (padrão)

`emit/svg.rs:20-83`. Ordem fixa de seções; `<defs>` com `clipPath` só quando há hachura; `id="canvas"`
como retângulo de fundo; números por `num`; texto por `escape`. `id_of` sanitiza ids para `clipPath`
(troca tudo que não é `[A-Za-z0-9_-]` por `-`, `emit/svg.rs:215-226`).

### 8.2 `scene` JSON

`emit/scene_json.rs:18-36`. JSON "bonito", `\n` final, chaves ordenadas (`serde_json::json!` com
`BTreeMap`, `preserve_order` desligado). Carrega `canvas{width,height}`, `desc`, `lang`, `layout`,
`theme`, `title`, `groups/edges/nodes/notes`, `version`. É **depuração estável**, não contrato de
interop.

### 8.3 `.excalidraw`

`emit/excalidraw.rs:22`. Derivado determinístico: `updated`/`created`/`versionNonce` = `0`, `seed` do
id, `index` posicional, `"source": "https://github.com/st-all-one/akribiechart"`
(`emit/excalidraw.rs:51`). É interop de uma via na prática — o round-trip de volta perde informação
(§13.5).

### 8.4 Raster (PNG/JPG/AVIF)

`emit/raster.rs:55-...`. SVG → `usvg` (com as **duas faces injetadas** a partir dos mesmos bytes que
mediram o layout) → `tiny-skia` → codec. `--scale` default 2; exige `--out` porque "binário não vai
para o terminal". AVIF com uma thread. Verificado: `retry.png` = 1672×344 RGBA, sha256 idêntico em duas
execuções **[E]**.

## 9. CLI (`akr`)

`crates/akribie-cli/src/cli.rs:8-42` — oito verbos com `clap`:

| verbo | função | exit não-zero |
|---|---|---|
| `check` | valida e reporta | `8` com erro; `--strict` promove warning a `8` |
| `fmt` | forma canônica | `1` com `--check` divergente; `8` se o fonte é inválido |
| `render` | SVG/scene/excalidraw para stdout; raster exige `--out` | `2` (formato/`--out`), `8` |
| `diff` | mudanças semânticas | `1` quando há mudanças |
| `schema` | gramática + tokens + códigos em JSON | `0` |
| `import` | `.excalidraw → .akrc` | `8`, `2` |
| `init` | escreve `akribie.toml` + bloco em `AGENTS.md` + skill | — |
| `prime` | gramática em texto para agente (`--long` = BNF + tabelas) | — |

Invariantes de borda que valem copiar:

- **`stdout` é dado, `stderr` é log.** Nenhum comando depende de cor ou largura de terminal
  (`cli.rs:1-4`).
- **`--json` troca a saída por um envelope de uma linha**:
  `{"success","command","data"?,"error"?,"warnings"}` (`ref/akribiechart/README.md:89-93`; saída real em
  [`exemplos/README.md`](exemplos/README.md) §2.3).
- **`--params JSON` substitui posicionais e flags**, e combinar os dois é `exit 2` — verificado **[E]**.
- **`--batch` JSONL**: uma linha por item, `best-effort`; `check --batch` sai `0` quando ao menos um
  item passa (`8` só se todos falharem); `render --batch --out-dir` exige o diretório existente
  (**[E]**: `exit 3`).
- **Tabela de exit codes em um único lugar**: `crates/akribie-core/src/error.rs:37-46`, e
  `akr schema` publica a mesma tabela (`error.rs:1-4`). Ótimo exemplo de "uma fonte de verdade".

## 10. MCP (`akribie-mcp`)

JSON-RPC 2.0 por linha em `stdin`/`stdout` (`crates/akribie-mcp/src/protocol.rs:38-62`).
`initialize` anuncia `protocolVersion: "2024-11-05"` e `serverInfo{name, version}`
(`protocol.rs:10`, verificado **[E]**). Três ferramentas (`crates/akribie-mcp/src/tools.rs:26-74`):

| tool | entrada | saída |
|---|---|---|
| `chart_check` | `source` **ou** `path`, `strict?` | texto com diagnósticos |
| `chart_render` | `source`/`path`, `format` (6 valores), `scale?` | texto, ou bloco de **imagem** em base64 |
| `chart_diff` | `old_source`, `new_source` (ambos obrigatórios) | texto `+ / - / ~` |

Regra de robustez que vale copiar: **falha de execução é resultado, não exceção do protocolo.** Um
`.akrc` ruim devolve `isError: true` com a primeira linha do diagnóstico; só `tools/call` com nome
desconhecido é erro de protocolo (`tools.rs:1-5`). O servidor nunca cai. Verificado **[E]**: quatro
chamadas reais numa sessão; e o teste
`tests/mcp.rs::malformed_json_is_a_protocol_error_without_killing_the_server`.

O envelope MCP **não** é o mesmo da CLI: a CLI tem `--json` com `{success,command,data,error,warnings}`;
o MCP usa `content[]`. São dois adaptadores da mesma função de domínio — o que é o desenho certo
(adaptador traduz, domínio não sabe).

## 11. Testes

230 testes, 0 falhas **[E]**. Estratégia em três camadas (`docs/specs/ARCHITECTURE.md:52-56`):

| camada | local | o que cobre |
|---|---|---|
| unidade | `#[cfg(test)]` em `crates/*/src/` | cada estágio com dados puros (**125**: 117 core + 7 cli + 1 mcp) |
| integração | `crates/*/tests/` | o **binário real** por pipe, fixtures temporárias (**105**) |
| goldens | `crates/akribie-cli/tests/golden/` | forma canônica, SVG dos 4 diagramas reais, `prime` |

Observações que importam:

- **Os testes de integração executam o binário**, não a biblioteca (`tests/common/mod.rs` tem `run()`
  sobre `Command`). É o motivo de `render.rs:26` provar determinismo de processo, não de função.
- **Goldens de SVG por diagrama real** (`tests/golden/reais/*.svg`, 68 KB no total) prendem os bytes
  de quatro `.akrc` de um projeto real (`fixtures/reais/`). Qualquer mudança de layout quebra o golden
  — que é o objetivo.
- **Um teste de propriedade sobre colisão de rótulo** percorre os quatro fixtures e afirma que nenhum
  rótulo encosta em nó, em outro rótulo ou na borda do canvas
  (`crates/akribie-core/src/layout/overlap_tests.rs`, 133 linhas; os três testes passaram **[E]**).
  Esse é um padrão que devemos copiar na forma: "propriedade geométrica afirmada sobre conteúdo real".

## 12. Performance

Medido com o binário **release** (`/tmp/dossie-akribie-target/release/akr`, 8,4 MB), mediana de 7
execuções de wall time de processo. Comandos e saídas em [`exemplos/README.md`](exemplos/README.md) §5.

| entrada | linhas | `check` | `render` SVG | `render` scene |
|---|---|---|---|---|
| 3 nós / 3 arestas | 9 | 2,5 ms | 3,1 ms | 2,7 ms |
| 200 nós / 199 arestas | 401 | 3,6 ms | 16,6 ms | 11,5 ms |
| 120 nós em estrela | 241 | 3,3 ms | 10,0 ms | 7,2 ms |
| 1000 nós / 999 arestas, 45 KB | 2001 | 14,4 ms | 80,8 ms | 53,1 ms |

`scripts/bench.sh` sobre as quatro fixtures reais (`fixtures/reais/`, 10 execuções cada):

```text
svg   anatomia-modulo 4.3 | boot-descoberta 3.7 | pipeline-cms-rbac 4.6 | topologia-runtime 4.5
scene anatomia-modulo 3.9 | boot-descoberta 3.8 | pipeline-cms-rbac 3.3 | topologia-runtime 3.1
png   anatomia-modulo 137.1 | boot-descoberta 129.1 | pipeline-cms-rbac 125.3 | topologia-runtime 101.4
```

Leitura: **`check` é ~5,6× mais barato que `render`** em 1000 nós (14,4 vs 80,8 ms), porque não faz
layout, pintura nem emissão. É a separação que permite um loop de validação rápido ("valide, não
desenhe") — e é o argumento empírico da ideia 2 em [`ideias.md`](ideias.md). O custo de `render` está
em `layout` (dagre) e na pintura de hachura; a emissão de SVG é "dezenas de microssegundos dentro de
uma execução de ~3 ms" segundo o próprio código (`emit/svg.rs:10-13`). Raster é ~30× o custo do SVG.

Rigor de performance do projeto: existe `scripts/bench.sh` que mede `svg`/`scene`/`png` por fixture
real e imprime `<modo> <fixture> <ms>` em ordem fixa, "para comparar versões sem opinião". **Não há
benchmark automatizado no CI** e **não há orçamento de alocação** — o projeto mede, mas não impõe teto.

Nota para nós: o kernel do layout é `dugong` (dagre portado), que aloca grafo/labels em `f64`. Isso
*é* um custo externo aceito em nome da simplicidade (ADR-0001 §1) — e explica por que `render` paga
10× mais que `check` mesmo com poucos nós.

## 13. Limites e riscos arquiteturais

### 13.1 A feature `raster` não compila (defeito real)

`crates/akribie-core/Cargo.toml:12-16` declara:

```toml
default = ["raster"]
raster = ["dep:resvg", "dep:usvg", "dep:tiny-skia", "dep:image"]
```

com o comentário "Desligar `raster` remove … e deixa só SVG/scene/excalidraw. Medição e layout
continuam funcionando."

**Verificado [E]**: `cargo build -p akribie-core --no-default-features` →

```
error[E0433]: cannot find module or crate `image` in this scope
 --> crates/akribie-core/src/emit/raster.rs:7:5
error[E0433]: cannot find module or crate `tiny_skia` in this scope
error[E0433]: cannot find module or crate `usvg` in this scope
```

`grep -rn 'feature = "raster"' crates/` → **nenhum resultado**. O módulo `raster` é declarado
incondicionalmente (`crates/akribie-core/src/emit/mod.rs:4`) e `emit::encode`/`RasterFormat` são
reexportados sem `cfg`. Consequências: a "matriz de features" não existe; o `make tree` que checa a
árvore sem `raster` (`Makefile:41-43`) verifica uma configuração que **nunca compila** — logo o gate é
teatro. O harness deste dossiê foi obrigado a usar as features padrão.

É exatamente o tipo de coisa que o `AGENTS.md` do `rich-editor-auy` §2.10 ("sem camada advisory: ou é
tipo, ou é teste que falha, ou gate de CI") manda não fazer.

### 13.2 Gramática frágil em um ponto específico

`edge api->retry` é `E007`, porque a seta precisa ser um token isolado (`lex.rs:97-123`). Um gerador de
texto (LLM ou humano) escreve a forma compacta com naturalidade. O diagnóstico é bom (`token inválido
'api->retry'`), mas a gramática poderia aceitar `id->id` como caso especial. **Verificado [E]**.

### 13.3 Escala: busca por id é linear

`Chart::node`/`label_of` fazem `iter().find` (`model/mod.rs:151-174`). Para `describe()` de um grafo com
E arestas e V nós, isso é O(E·V). Com 1000 nós a descrição custa 10⁶ comparações de `&str` — invisível
aqui (§12) porque 1000 é pouco, mas é exatamente o padrão que não podemos herdar num documento com
dezenas de milhares de nós. O `finish.rs:40-45` já resolve o caso quente com um `IndexMap`; a
inconsistência é o cheiro.

### 13.4 A11y abaixo do que o próprio spec promete

`role="list"` em `#nodes` e `role="listitem"` por nó estão em `docs/specs/A11Y.md:16,25` mas **não**
são emitidos (`emit/svg.rs:71-75`, `emit/svg.rs:95-122`). O SVG também não é navegável por teclado.
Documentação e código divergem — e é a documentação que um revisor confiaria se não lesse o emissor.

### 13.5 `import` de `.excalidraw` é lossy e verboso

Verificado **[E]**: `akr render retry.akrc --format excalidraw > x.excalidraw; akr import x.excalidraw`
produz `layout=free`, todo elemento com `at=`/`size=` explícitos, `shape=rounded` para todos e sem
`curve=arc`. Funciona como ponto de partida, não como round-trip. `W006 element-dropped` existe para
avisar sobre o que não é representável (`diag.rs:84`).

### 13.6 `check --batch` só falha quando todos os itens falham

Com uma mistura de itens válidos e inválidos o processo sai `0`; `exit 8` só quando `ok == 0`
(`crates/akribie-cli/src/app/emit/batch.rs:21-24`, **[E]**). É coerente com "best-effort", mas
empurra a decisão para o consumidor. Um `--fail-fast` ou um exit agregado seria mais útil em CI.

### 13.7 Build de release: lento no ambiente, não no código

O build de release falhou duas vezes por `Disk quota exceeded (os error 122)` — primeiro compilando
`read-fonts v0.46.0`, depois linkando `akr`. Só concluiu (`Finished release profile in 1m 03s`)
depois de remover os artefatos de `debug` do `CARGO_TARGET_DIR`. `profile.release` usa
`lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks = true`
(`Cargo.toml:43-47`): excelente para distribuir um binário de 8,4 MB, caro para compilar em tmpfs
pequeno. O determinismo sobrevive à otimização — release e debug dão o **mesmo** sha256 de SVG e PNG
(**verificado [E]**).

### 13.8 Docs desatualizados em três pontos

1. `docs/plan/00-panorama.md:39-40` — "quatro portas (`Fs`, `Git`, `Env`, `Logger`)"; são duas.
2. `docs/specs/DETERMINISM.md:32` — "a CLI usa a porta `Env`"; removida.
3. `docs/specs/A11Y.md:16,25` — `role="list"`/`listitem` que o emissor não produz (§13.4).

### 13.9 Não é `no_std`, não é incremental, não tem estado

O akribie compila **do zero a cada chamada**. Isso é ótimo para um compilador e inútil para um editor:
não há modelo persistente, não há step/rebase, não há seleção. Qualquer reaproveitamento é conceitual
(ver [`ideias.md`](ideias.md)).

## 14. Mapa de arquivos que este dossiê copiou

| cópia | origem | por quê |
|---|---|---|
| [`codigo/pipeline.rs`](codigo/pipeline.rs) | `crates/akribie-core/src/pipeline.rs` | composição dos estágios e do relatório de `check` |
| [`codigo/diag.rs`](codigo/diag.rs) | `crates/akribie-core/src/diag.rs` | catálogo de códigos + `render()` + `to_json()` + `sort()` |
| [`codigo/fmt.rs`](codigo/fmt.rs) | `crates/akribie-core/src/fmt.rs` | forma canônica idempotente |
| [`codigo/a11y.rs`](codigo/a11y.rs) | `crates/akribie-core/src/a11y.rs` | `<desc>` derivado do modelo + auditoria de contraste |
| [`codigo/emit-svg.rs`](codigo/emit-svg.rs) | `crates/akribie-core/src/emit/svg.rs` | emissão determinística e acessível |
| [`codigo/diff-mod.rs`](codigo/diff-mod.rs) | `crates/akribie-core/src/diff/mod.rs` | diff semântico com identidade por elemento |
