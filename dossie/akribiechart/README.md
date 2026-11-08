# akribie (`akribiechart`) — dossiê

> Estudo de caso verificável do projeto em [`ref/akribiechart`](../../ref/akribiechart).
> Commit local **`7275fedfe81f2d4ac1df4e92a580c05b90a81049`** (2026-11-03), versão de workspace
> **0.1.3**, licença **MIT OR Apache-2.0** (`ref/akribiechart/Cargo.toml:19`).
> Os comandos marcados como executados foram rodados neste checkout; saídas literais estão em
> [`exemplos/README.md`](exemplos/README.md) e [`avaliacao.md`](avaliacao.md) §1.
> Nada em `ref/` foi alterado (builds com `CARGO_TARGET_DIR=/tmp/dossie-akribie-target`).

---

## 1. O que é

**akribie não é um editor de texto rico.** É um **compilador de diagramas**: um binário Rust que lê
um arquivo de texto (`.akrc`) declarando *topologia e rótulos* e emite artefatos derivados — SVG por
padrão, e sob demanda PNG/JPG/AVIF, `scene` JSON e `.excalidraw`.

```text
.akrc  ──lex→parse→resolve→layout→paint→emit──▶  SVG · PNG/JPG/AVIF · scene JSON · .excalidraw
```

Duas superfícies de uso, um núcleo só:

| superfície | binário/crate | transporte |
|---|---|---|
| CLI | `akr` (`crates/akribie-cli`) | argv, `stdin`/`stdout`, exit codes |
| MCP | `akribie-mcp` | JSON-RPC 2.0 por linha em `stdin`/`stdout` |
| biblioteca | `akribie-core` | funções puras Rust, sem I/O |

O alvo declarado é o **LLM/agente**, não o humano desenhando
(`ref/akribiechart/AGENTS.md:5-10`; `docs/plan/00-panorama.md:1-6`): o autor escreve *o que existe e
como se liga*; geometria, cor, texto alternativo e bytes são calculados.

Metáfora que se sustenta no código: `.akrc` é **fonte** versionada, e todo o resto é **artefato de
build** descartável (ADR-0002, `docs/adr/ADR-0002-fonte-texto-canonica.md:24-27`).

## 2. Que problema resolve

Três dores concretas, todas com um custo nomeado:

1. **Diagrama binário não tem diff útil.** Um `.excalidraw` ou um `.svg` versionado carrega
   coordenadas, `versionNonce`, ordem de atributos. O diff mostra ruído, não mudança de intenção
   (`docs/adr/ADR-0002-fonte-texto-canonica.md:11-17`). A resposta: um formato textual fechado,
   com **forma canônica única** e `akr diff` que compara semântica, nunca geometria
   (`crates/akribie-core/src/diff/mod.rs:1-7`).
2. **Regeneração muda bytes sem mudar o desenho.** Git mostra ruído a cada build. A resposta:
   determinismo como **contrato de bytes** — mesma entrada + mesma versão ⇒ mesmos bytes de SVG,
   `scene` e `.excalidraw` (`docs/specs/DETERMINISM.md:5-14`; verificado §3.2).
3. **Acessibilidade vira pós-processamento manual.** A resposta: o SVG já sai com
   `role="img"`, `<title>`, `<desc>` derivado do **modelo** (não da geometria), `aria-label` por
   figura, `lang`, e contraste WCAG auditado por `W007`
   (`crates/akribie-core/src/emit/svg.rs:20-83`, `crates/akribie-core/src/a11y.rs:16-136`).

## 3. Resultado real (medido, não prometido)

### 3.1 Suíte de testes

```sh
$ CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo test --workspace --locked
```

`230` testes, `0` falhas, `0` doc-tests (contagem por `test result:` em cada binário):

| alvo | testes |
|---|---|
| `akribie-core` (unit, `src/**/tests.rs`) | 117 |
| `akribie-core` — `tests/lex.rs` | 7 |
| `akribie-core` — `tests/parse.rs` | 13 |
| `akribie-cli` (unit, `src/main.rs`) | 7 |
| `akribie-cli` — `check.rs` / `diff.rs` / `docs.rs` | 22 / 10 / 4 |
| `akribie-cli` — `fmt.rs` / `import.rs` / `init.rs` / `prime.rs` | 11 / 6 / 2 / 2 |
| `akribie-cli` — `render.rs` | 23 |
| `akribie-mcp` (unit + `tests/mcp.rs`) | 1 + 5 |
| `akribie-adapters` | 0 |

O número bate com `grep -rn '#\[test\]' crates --include=*.rs | wc -l` → `230`. O README do projeto
não anuncia contagem de testes; não há número a contestar.

### 3.2 Determinismo (verificado byte a byte)

```sh
$ akr render retry.akrc | sha256sum        # duas execuções
0f98809a1107557b914fc8d4fdb4cbb224a0e929ffcb5614f3829900b757eddb  -
0f98809a1107557b914fc8d4fdb4cbb224a0e929ffcb5614f3829900b757eddb  -
$ akr render retry.akrc --format scene     | sha256sum   # idem: ca83980b…
$ akr render retry.akrc --format excalidraw| sha256sum   # idem: a02bf55e…
$ akr render retry.akrc --format png --scale 2 --out a.png
$ akr render retry.akrc --format png --scale 2 --out b.png
99a56460ae4cdee9969aa93db70bea8b4637eaf7c68e4f420729f7ca3cdd8dfd  a.png
99a56460ae4cdee9969aa93db70bea8b4637eaf7c68e4f420729f7ca3cdd8dfd  b.png
```

Os mecanismos que sustentam isso são mecânicos, não disciplina:
`num::format` em vez de `f32::to_string` (`crates/akribie-core/src/num.rs:1-25`), `IndexMap` em vez
de `HashMap` (`Cargo.toml:28`, `crates/akribie-core/src/layout/finish.rs:40-45`), fontes
`include_bytes!` em vez de fontes do sistema (`crates/akribie-core/src/text.rs:29-31`), raster com
`num_threads(1)` (`crates/akribie-core/src/emit/raster.rs:89`), e `updated`/`versionNonce` fixos em
`0` no `.excalidraw` (`crates/akribie-core/src/emit/excalidraw.rs:3`).

### 3.3 Performance (binário release, mediana de 7 execuções)

| entrada | linhas | `check` | `render` → SVG | `render` → `scene` |
|---|---|---|---|---|
| `retry.akrc` (3 nós, 3 arestas) | 9 | 2,5 ms | 3,1 ms | 2,7 ms |
| `big200.akrc` (200 nós, 199 arestas) | 401 | 3,6 ms | 16,6 ms | 11,5 ms |
| `star120.akrc` (120 nós em estrela) | 241 | 3,3 ms | 10,0 ms | 7,2 ms |
| `big1000.akrc` (1000 nós, 999 arestas, 45 KB) | 2001 | 14,4 ms | 80,8 ms | 53,1 ms |

(Coluna `linhas` = `wc -l` da fixture; `retry.akrc` tem 9 linhas físicas. Reexecutado na revisão,
`big1000` deu 16,6 ms `check` / 85,2 ms SVG / 62,3 ms scene — mesma ordem de grandeza, razão 5,1×.)

Raster é outro patamar: `scripts/bench.sh` nas quatro fixtures reais dá **SVG 3,7–4,6 ms**,
**scene 3,1–3,9 ms** e **PNG 101–137 ms** (o encoder domina). O custo de `render` está em `layout`
(dagre via `dugong`, `crates/akribie-core/src/layout/flow.rs:1-8`); `check` é ~5,6× mais barato
porque não faz layout, pintura nem emissão.

O build de release só concluiu depois de liberar espaço em `/tmp` (quota de disco, ver §6.10); o
binário tem **8,4 MB** e produz **os mesmos sha256** que o build debug — determinismo não depende de
otimização.

### 3.4 Tamanho

`10 355` linhas em `crates/akribie-core/src`, `2 982` em `akribie-cli`, `468` em `akribie-mcp`, `93`
em `akribie-adapters`, mais `2 100` em testes de integração. **Nenhum arquivo de `src/` passa de 300
linhas** (`find crates/*/src -name '*.rs' | xargs wc -l | awk '$1>300'` → vazio), o que é um invariante
declarado e verificado por script (`scripts/check_file_length.sh`, `Makefile:26-27`).

## 4. Como usar

### 4.1 Verificar e renderizar

```sh
cargo build --locked -p akribie-cli          # produz target/debug/akr
akr check chart.akrc                         # silencioso quando está tudo certo
akr render chart.akrc > chart.svg
akr render chart.akrc --format png --scale 2 --out chart.png
akr render chart.akrc --format scene | jq .
akr fmt chart.akrc --write                   # forma canônica
akr fmt chart.akrc --check                   # exit 1 quando diverge
akr diff antigo.akrc novo.akrc               # exit 1 quando há mudanças
akr schema                                   # gramática + tokens + códigos, em JSON
akr prime                                    # gramática em texto para colar num agente
```

### 4.2 Como biblioteca (headless, sem CLI)

Este é o caminho que importa para nós. `akribie-core` não conhece `std::fs`, relógio, ambiente nem
RNG global (`crates/akribie-core/src/lib.rs:1-20`; `docs/specs/ARCHITECTURE.md:1-25`). O harness em
[`exemplos/harness`](exemplos/harness) usa só cinco chamadas:

```rust
let font = Font::load()?;                                   // faces OFL embutidas
let report = pipeline::check(SOURCE, &Config::default(), &font);  // lex→parse→resolve→check
let placed = layout::layout(&report.chart, &font);           // geometria
let scene  = paint::paint(&report.chart, &placed);           // cores + paths
let svg    = emit::svg(&scene, report.chart.theme);          // bytes determinísticos
```

Saída literal do harness (`cargo run --offline --quiet`, colada inteira em
[`exemplos/README.md`](exemplos/README.md)):

```
== 3. diff semântico ==
~ node api label: API -> API Gateway
~ node retry fill: amber -> none
~ node retry label: Retry -> retry
~ node retry shape: cloud -> rounded

== 4. emissão determinística ==
canvas: 439.72 x 122.5
svg bytes: 2636
svg(idempotente): true
```

### 4.3 Exit codes

Fonte única: `crates/akribie-core/src/error.rs:37-46`. Verificado executando:

| exit | significado | verificado |
|---|---|---|
| `0` | sucesso | `akr check retry.akrc` |
| `1` | `fmt --check` divergiu, ou `diff` achou mudanças | `akr fmt messy2.akrc --check`; `akr diff old new` |
| `2` | uso inválido (flag, formato, `--params` + posicional) | `akr render --format pdf`; `--params` com arquivo posicional |
| `3` | arquivo/revisão/diretório inexistente | `akr check nao-existe.akrc`; `--out-dir` inexistente |
| `5` | erro de I/O | não exercitado |
| `8` | `.akrc` inválido (≥1 erro) | `akr check broken.akrc` |
| `70` | invariante interna | não exercitado |

## 5. O que funciona

- **Pipeline de seis estágios, puro e testável em isolamento.** Cada estágio é uma função sobre
  dados; `pipeline::check` só compõe (`crates/akribie-core/src/pipeline.rs:27-40`).
- **Diagnóstico é dado, não log.** 16 códigos estáveis (`E001`–`E009`, `W001`–`W007`), cada um com
  linha, coluna, mensagem e `hint:` de correção; a mesma tabela alimenta `akr schema` e `akr prime`
  (`crates/akribie-core/src/diag.rs:55-91`). Saída real:

  ```
  broken.akrc:2:18: E003 invalid-value: `shape=trapezio` não é um valor válido
    hint: valores de `shape`: rect, rounded, circle, diamond, cloud, cylinder, hexagon, note, text
  broken.akrc:4:1: W001 implicit-node: `fantasma` não foi declarado; criado como nó implícito
    hint: declare o nó para controlar forma, cor e rótulo
  ```

- **Forma canônica idempotente.** `fmt(fmt(x)) == fmt(x)` medido, e `--check` sai `1` em CI.
  As regras estão em `crates/akribie-core/src/fmt.rs:3-13` e `docs/specs/GRAMMAR.md:54-66`.
- **Diff semântico legível**, ordenado e estável: `- edge retry->api "backoff"` e
  `~ node retry shape: cloud -> rounded` — campos, não coordenadas
  (`crates/akribie-core/src/diff/mod.rs:84-132`).
- **Acessibilidade como contrato de bytes.** `role="img"`, `aria-labelledby`, `<title>`, `<desc>`
  derivado do modelo, `aria-label` por figura, `lang`, `focusable="false"`; contraste auditado por
  `W007` (`crates/akribie-core/src/a11y.rs:58-136`). `xmllint --noout` aceita a saída.
- **MCP estável sob entrada ruim.** Erro de `.akrc` volta como `isError: true` dentro do resultado;
  JSON malformado é erro de protocolo sem derrubar o processo (`crates/akribie-mcp/src/tools.rs:1-5`;
  teste `tests/mcp.rs::malformed_json_is_a_protocol_error_without_killing_the_server`).
- **Disciplina de qualidade com guarda mecânica**: `forbid(unsafe_code)` no core, `unwrap_used`/
  `expect_used`/`panic`/`indexing_slicing` negados no workspace (`Cargo.toml:77-120`), `cargo tree`
  proibindo `tokio`/`async-std`/`rayon` (`Makefile:32-38`).

## 6. O que **não** funciona / não existe

Verificado, não inferido. Detalhe e caminho:linha em [`avaliacao.md`](avaliacao.md) §4.

1. **A feature `raster` é decorativa.** `crates/akribie-core/Cargo.toml:12-16` promete que desligá-la
   "deixa só SVG/scene/excalidraw". **Não compila**:
   `cargo build -p akribie-core --no-default-features` → `error[E0433]: cannot find module or crate
   'image'`. Não existe nenhum `#[cfg(feature = "raster")]` no crate. O harness deste dossiê foi
   forçado a usar as features padrão por causa disso.
2. **`<desc>` é sempre em português.** `a11y::describe` tem as palavras "nós", "arestas" e "Arestas:"
   **hardcoded** (`crates/akribie-core/src/a11y.rs:16-42`). Com `lang=en`:

   ```
   <svg … lang="en" …>
   <desc id="akr-d">2 nós, 1 aresta. Nós: API; DB.
   ```

   O `lang` anuncia inglês e um leitor de tela pronuncia português. O ADR-0005 §"Negativas"
   (`docs/adr/ADR-0005-acessibilidade-svg.md:71-78`) reconhece "não traduzido", mas o problema é mais
   estreito e mais grave: o texto *gerado* é de um idioma fixo.
3. **`->` exige espaços.** `edge api->retry "5xx"` é `E007 malformed-line: token inválido
   'api->retry'`. Só `edge api -> retry` parseia (`crates/akribie-core/src/lex.rs:97-123` — a seta
   precisa ser um token isolado). Fricção real para quem escreve à mão e para geração de LLM.
4. **Sem edição interativa, sem seleção, sem undo, sem IME.** Fora de escopo declarado
   (`docs/plan/00-panorama.md:59-60`: "Fora da v1: … edição interativa, … colaboração").
5. **Gramática fechada e sem plugins.** Adicionar um `shape`, uma chave ou um statement exige editar
   o crate: os conjuntos são `const &[&str]` (`crates/akribie-core/src/syntax/specs.rs:63-82`) e os
   tipos são `enum` fechados (`crates/akribie-core/src/model/enums.rs`). Não é defeito para o akribie;
   é um teto duro para nós.
6. **`import` de `.excalidraw` é lossy.** Round-trip `akrc → excalidraw → akrc` troca o layout para
   `free`, colapsa `cloud`/`cylinder` em `rounded` e perde `curve=arc` — e materializa todo `at=` e
   `size=` como coordenada explícita. Saída colada em [`exemplos/README.md`](exemplos/README.md).
7. **`--out-dir` não cria o diretório.** `akr render --batch lote.jsonl --out-dir saida` →
   ``erro: diretório de saída não encontrado: `saida` `` (exit 3). É preciso `mkdir -p` antes.
8. **`check --batch` só falha o processo quando *todos* os itens falham.** Com mistura de itens
   válidos e inválidos o exit é `0`; só `ok == 0` produz `exit 8`
   (`crates/akribie-cli/src/app/emit/batch.rs:21-24`). É "best-effort" por desenho
   (`ref/akribiechart/README.md:92-93`), mas significa que um CI precisa inspecionar a saída, não o exit code, para
   pegar erros parciais.
9. **Docs citam portas que foram removidas.** `docs/plan/00-panorama.md:39-40` e
   `docs/specs/DETERMINISM.md:32` falam em `Env`/`Logger`; `crates/akribie-core/src/ports/mod.rs:5-6`
   e `docs/specs/ARCHITECTURE.md:20-21` dizem que foram **removidas em 0.1.3**. O panorama não foi
   atualizado.
10. **O build de release é frágil neste ambiente** — não por bug de código: `Disk quota exceeded (os
    error 122)` ao compilar `read-fonts v0.46.0` e, na segunda tentativa, ao linkar com
    `lto = "fat"`. Só concluiu depois de remover os artefatos de `debug`. Com `lto = "fat"` +
    `codegen-units = 1` (`Cargo.toml:43-47`), o pico de disco é alto para um projeto deste tamanho.
11. **O campo `warnings` do envelope `--json` é sempre vazio.** `Envelope { …, warnings: Vec<String> }`
    é construído com `Vec::new()` nos três construtores e **nunca** recebe valor
    (`crates/akribie-cli/src/envelope.rs:35, 46, 60, 78`). Os avisos reais vivem em
    `data.diagnostics` — verificado: `akr check warn.akrc --strict --json` mostra os dois `W002` em
    `data.diagnostics[]` e `"warnings":[]` no topo. É superfície pública que não carrega informação.

## 7. Prós e contras como referência para um editor de texto rico

O `rich-editor-auy` é um **engine headless** (modelo canônico, steps, transações, wire). O akribie
não compartilha uma linha de domínio com ele — não há documento editável, não há marks, não há
posições. O que compartilha é o **contrato de rigor**: bytes canônicos, erro como dado, a11y derivada
do modelo, diff semântico. Prosa curta:

**Prós**

- **Um formato textual fechado com forma canônica e testemunho de idempotência.** É exatamente a
  propriedade que queremos no serializador (`fmt` idempotente + `--check` em CI
  → [`codigo/fmt.rs`](codigo/fmt.rs)).
- **Catálogo de diagnósticos como tipo, não como string.** Código estável, `severity` intrínseca,
  `hint` obrigatório na prática, tabela única que alimenta três consumidores
  (→ [`codigo/diag.rs`](codigo/diag.rs)).
- **Determinismo com guardas mecânicas**, não com promessa: `num::format`, `IndexMap`, fontes
  embutidas, uma thread no encoder, teste de bytes a cada PR (→ [`codigo/emit-svg.rs`](codigo/emit-svg.rs)).
- **A11y gerada do modelo, não da geometria.** `<desc>` lista relações; contraste é auditado sobre
  *todas* as combinações de paleta (→ [`codigo/a11y.rs`](codigo/a11y.rs)).
- **Diff semântico com identidade explícita por elemento** (nó por id, aresta por `(from,to,ordinal)`,
  nota por âncora + posição). É o desenho certo para relatar mudanças de documento.
- **Portas + fakes**: o domínio é testável sem ambiente; 117 testes de core rodam offline em 0,2 s.

**Contras**

- **Não é um editor.** Zero sobre seleção, alças de composição (IME), undo, clipboard, marks, schema
  de conteúdo inline. Metade da matriz de usabilidade do `_GUIA.md` é `—` por ausência, não por
  lacuna nossa de leitura.
- **Acoplado a uma linguagem visual fechada.** Paleta fechada, 9 formas, 6 layouts, 2 fontes. Nada
  disso é genérico.
- **`akribie-core` não é `no_std`.** Usa `std::path::Path` nas portas, `String`, `Vec`,
  `serde_json`; o "sem I/O" é sobre não abrir arquivo, não sobre ausência de `std`
  (`crates/akribie-core/src/ports/mod.rs:8-10`).
- **Uma feature que não compila e um ADR desatualizado** — sinal de que o CI não cobre a matriz de
  features. Guarda contra isso: buildar `--no-default-features` em CI.
- **Risco de bus factor**: um único commit no clone, um autor no changelog.

## 8. Licença

**MIT OR Apache-2.0** (`ref/akribiechart/Cargo.toml:19`; textos integrais em `ref/akribiechart/LICENSE`).
Os dois arquivos de fonte commitados em `assets/` são **OFL-1.1** e não entram sob a licença do
código: `assets/EXCALIFONT-OFL.txt`, `assets/LIBERATION-OFL.txt` (`ref/akribiechart/README.md:122-126`).
Isso permite copiar arquivos completos para este dossiê com aviso — feito em
[`codigo/`](codigo/INDEX.md), com SHA-256 conferido contra a origem.

## 9. Mapa deste dossiê

| arquivo | conteúdo |
|---|---|
| [`arquitetura.md`](arquitetura.md) | pipeline, crates, portas, modelo, determinismo, a11y, formatos, CLI/MCP, testes, performance, limites — com `caminho:linha` |
| [`avaliacao.md`](avaliacao.md) | 13 itens de usabilidade (E/C/T/D/X/—), 10 dimensões com nota 0–5, copiar/adaptar/evitar, confiança |
| [`codigo/INDEX.md`](codigo/INDEX.md) | 6 arquivos reais copiados verbatim, com origem, hash e motivo |
| [`exemplos/README.md`](exemplos/README.md) | comandos reais, saída colada, o que não rodou e por quê |
| [`ideias.md`](ideias.md) | 11 ideias priorizadas (5 agora, 3 pós-MVP, 3 rejeitar) para o `rich-editor-auy`, com evidência, encaixe, custo/risco |
