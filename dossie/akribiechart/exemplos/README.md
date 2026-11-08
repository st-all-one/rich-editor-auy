# akribie — exemplos executados

> Tudo aqui foi **executado** neste checkout (`ref/akribiechart`, commit `7275fed`), com o binário
> `akr` construído em `/tmp/dossie-akribie-target`. As saídas estão coladas literalmente.
> O que não rodou está em §6, com o motivo.
>
> Legenda: `$ AKR=/tmp/dossie-akribie-target/release/akr` em todos os blocos.

---

## 0. Arquivos deste diretório

| arquivo | origem | o que é |
|---|---|---|
| `retry.akrc` | cópia verbatim de `ref/akribiechart/fixtures/retry.akrc` (sha256 `f726221d…` confere) | exemplo mínimo real usado pelo projeto |
| `broken.akrc` | escrito aqui | `.akrc` com três defeitos distintos, para exercitar o diagnóstico |
| `messy.akrc` | escrito aqui | `.akrc` válido mas fora da forma canônica, para exercitar `fmt` |
| `novo.akrc` | escrito aqui | variação de `retry.akrc` com mudança de forma e uma aresta removida, para `diff` |
| `harness/` | escrito aqui | crate Rust mínimo que usa `akribie-core` **como biblioteca**, sem CLI |

`retry.akrc`:

```akrc
chart v1 "Política de retry" direction=lr layout=flow theme=paper

node api "API Gateway" fill=blue shape=rounded
node retry "Retry ×3" fill=amber shape=cloud
node dlq "DLQ" fill=red shape=cylinder

edge api -> retry "5xx"
edge retry -> api "backoff" curve=arc style=dashed
edge retry -> dlq "esgotado" style=dashed
```

## 1. Build

```sh
$ cd ref/akribiechart
$ CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo build --locked -p akribie-cli
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s

$ CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo build --locked --release -p akribie-cli
   Compiling akribie-cli v0.1.3 (…/crates/akribie-cli)
    Finished `release` profile [optimized] target(s) in 1m 03s

$ ls -la /tmp/dossie-akribie-target/release/akr
-rwxr-xr-x 8384056 … /tmp/dossie-akribie-target/release/akr
```

**Nota honesta sobre o build de release**: falhou duas vezes com
`Disk quota exceeded (os error 122)` (tmpfs de `/tmp` com 8,7 GB compartilhado com outros builds do
repositório), ao compilar `read-fonts v0.46.0` e depois ao linkar com `lto = "fat"`. Só concluiu
depois de remover os artefatos de `debug` (mantendo o binário). **Não é falha do projeto**; é
restrição de ambiente. Todos os números de §5 são do binário **release**.

## 2. `check` — validação

### 2.1 Documento válido: silêncio e `exit 0`

```sh
$ $AKR check retry.akrc
$ echo "exit=$?"
exit=0
```

Nada em `stdout`. É o contrato "silencioso quando está tudo certo" (`ref/akribiechart/README.md:57`).

### 2.2 Documento inválido: 3 diagnósticos, `exit 8`

`broken.akrc`:

```akrc
chart v1 "Quebrado"
node a "A" shape=trapezio
node a "Duplicado"
edge a -> fantasma
```

```sh
$ $AKR check broken.akrc
broken.akrc:2:18: E003 invalid-value: `shape=trapezio` não é um valor válido
  hint: valores de `shape`: rect, rounded, circle, diamond, cloud, cylinder, hexagon, note, text
broken.akrc:3:1: E004 duplicate-id: id `a` já declarado
  hint: ids são únicos entre `group` e `node`; renomeie um dos dois
broken.akrc:4:1: W001 implicit-node: `fantasma` não foi declarado; criado como nó implícito
  hint: declare o nó para controlar forma, cor e rótulo
$ echo "exit=$?"
exit=8
```

Repare: **três erros independentes na mesma execução** (o parser recupera por linha), coluna
`1-based` em caracteres, e cada mensagem traz o conjunto válido — que é o que permite a um agente
corrigir sem adivinhar.

### 2.3 O mesmo, em JSON

```sh
$ $AKR check broken.akrc --json; echo "exit=$?"
{"success":false,"command":"check","data":{"diagnostics":[
 {"code":"E003","col":18,"hint":"valores de `shape`: rect, rounded, circle, diamond, cloud, cylinder, hexagon, note, text","line":2,"message":"`shape=trapezio` não é um valor válido","severity":"error"},
 {"code":"E004","col":1,"hint":"ids são únicos entre `group` e `node`; renomeie um dos dois","line":3,"message":"id `a` já declarado","severity":"error"},
 {"code":"W001","col":1,"hint":"declare o nó para controlar forma, cor e rótulo","line":4,"message":"`fantasma` não foi declarado; criado como nó implícito","severity":"warn"}]},
 "error":{"code":"schema","message":"2 erro(s), 1 aviso(s) (ver diagnósticos)"},"warnings":[]}
exit=8
```

*(quebras de linha adicionadas só para leitura; a saída real é uma linha só.)*

### 2.4 `--strict` promove warning a erro

```sh
$ cat warn.akrc
chart v1 "Warn" layout=flow
node a "A"
node solto "Solto"

$ $AKR check warn.akrc; echo "exit=$?"
warn.akrc:2:1: W002 isolated-node: nó `a` sem arestas em `layout=flow`
  hint: conecte o nó ou use `layout=grid`, `layout=stack` ou `layout=free`
warn.akrc:3:1: W002 isolated-node: nó `solto` sem arestas em `layout=flow`
  hint: conecte o nó ou use `layout=grid`, `layout=stack` ou `layout=free`
exit=0

$ $AKR check warn.akrc --strict; echo "exit=$?"
(…) mesmas duas linhas …
exit=8
```

`--strict` muda **só o exit code**, não o texto: os diagnósticos continuam `warn`
(`severity:"warn"` no JSON) mas o processo falha. Verificado também que `layout=grid` **não** gera
`W002` — a regra é condicionada a `chart.layout.expects_graph()` (`crates/akribie-core/src/pipeline.rs:43-45`).

### 2.5 Faixa numérica

```sh
$ printf 'chart v1 "X" fontsize=200\nnode a\n' > fs.akrc && $AKR check fs.akrc; echo "exit=$?"
fs.akrc:1:23: E003 invalid-value: `200` está fora da faixa de `fontsize`
  hint: `fontsize` aceita de 14 a 96
fs.akrc:2:1: W002 isolated-node: nó `a` sem arestas em `layout=flow`
(…)
exit=8
```



## 3. `render`

### 3.1 SVG em `stdout`

```sh
$ $AKR render retry.akrc | head -6
<svg xmlns="http://www.w3.org/2000/svg" width="835.85" height="171.89" viewBox="0 0 835.85 171.89" role="img" aria-labelledby="akr-t akr-d" lang="pt-BR" focusable="false">
<title id="akr-t">Política de retry</title>
<desc id="akr-d">3 nós, 3 arestas. Nós: API Gateway; Retry ×3; DLQ.
Arestas: API Gateway → Retry ×3 (5xx); Retry ×3 → API Gateway (backoff); Retry ×3 → DLQ (esgotado).</desc>
<defs><clipPath id="c-api"><path d="M44 70.43 L160.6 70.43 A12 12 0 0 1 172.6 82.43 L172.6 109.43 A12 12 0 0 1 160.6 121.43 L44 121.43 A12 12 0 0 1 32 109.43 L32 82.43 A12 12 0 0 1 44 70.43 Z"/></clipPath>…
<rect id="canvas" width="835.85" height="171.89" fill="#ffffff"/>
```

Figuras emitidas: **5 `<g>`** — `edges`, `nodes` (seções) e uma por elemento. Contagem por
`data-kind`: `1 edge`, `2 node` no exemplo mínimo de §4.

### 3.2 XML válido

```sh
$ $AKR render retry.akrc | xmllint --noout - && echo "xmllint OK"
xmllint OK
```

### 3.3 Determinismo — bytes idênticos em quatro formatos

```sh
$ $AKR render retry.akrc | sha256sum
0f98809a1107557b914fc8d4fdb4cbb224a0e929ffcb5614f3829900b757eddb  -
$ $AKR render retry.akrc | sha256sum
0f98809a1107557b914fc8d4fdb4cbb224a0e929ffcb5614f3829900b757eddb  -

$ $AKR render retry.akrc --format scene      | sha256sum   # 2×
ca83980b45dfd52242343eba6a20d60ea2a6a6c431f780953984b0612639f8ad  -
ca83980b45dfd52242343eba6a20d60ea2a6a6c431f780953984b0612639f8ad  -

$ $AKR render retry.akrc --format excalidraw | sha256sum   # 2×
a02bf55eca1cf994bde1701413a2db9874b5a0ab0ba6072171e4bd629b07a917  -
a02bf55eca1cf994bde1701413a2db9874b5a0ab0ba6072171e4bd629b07a917  -

$ $AKR render retry.akrc --format png --scale 2 --out a.png
$ $AKR render retry.akrc --format png --scale 2 --out b.png
99a56460ae4cdee9969aa93db70bea8b4637eaf7c68e4f420729f7ca3cdd8dfd  a.png
99a56460ae4cdee9969aa93db70bea8b4637eaf7c68e4f420729f7ca3cdd8dfd  b.png
$ file a.png
a.png: PNG image data, 1672 x 344, 8-bit/color RGBA, non-interlaced
```

**Detalhe que vale registrar**: o binário **debug** e o **release** produzem o *mesmo* sha256
(`0f98809a…` para o SVG e `99a56460…` para o PNG). Determinismo não depende de otimização.

### 3.4 `scene` JSON — geometria para depuração

```sh
$ $AKR render retry.akrc --format scene | head -20
{
  "canvas": {
    "height": 171.89,
    "width": 835.85
  },
  "desc": "3 nós, 3 arestas. Nós: API Gateway; Retry ×3; DLQ.\nArestas: API Gateway → Retry ×3 (5xx); Retry ×3 → API Gateway (backoff); Retry ×3 → DLQ (esgotado).",
  "edges": [
    {
      "from": "api",
      "head": true,
      "id": "api->retry#0",
      "label": "5xx",
      "ordinal": 0,
      "route": [
        [
          172.6,
          95.93
```

### 3.5 Erros de borda de `render` (todos verificados)

```sh
$ $AKR render retry.akrc --format pdf
erro: formato `pdf` desconhecido; use svg, scene, excalidraw, png, jpg ou avif
$ echo "exit=$?"; # 2

$ $AKR render retry.akrc --format png > /dev/null
erro: `--format png|jpg|avif` exige `--out PATH`: binário não vai para o terminal
$ echo "exit=$?"; # 2

$ $AKR check nao-existe.akrc
erro: arquivo não encontrado: nao-existe.akrc
$ echo "exit=$?"; # 3

$ $AKR check retry.akrc --params '{"file":"retry.akrc"}'
erro: `--params` substitui posicionais e flags do verbo; use um ou outro (só `--json` soma)
$ echo "exit=$?"; # 2
```

## 4. O harness de biblioteca (`harness/`)

Mostra o uso **headless**: `akribie-core` como biblioteca, sem processo filho, sem arquivo.

```sh
$ cd dossie/akribiechart/exemplos/harness
$ CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo run --offline --quiet
```

Saída literal (exit 0):

```text
== 1. diagnóstico é dado ==
documento válido: true
diagnósticos: 0
segundo documento, diagnósticos: 1

== 2. forma canônica (idempotente) ==
chart v1 "Retry" direction=lr lang=pt-BR layout=flow

node api "API Gateway" fill=blue

edge api -> retry "5xx"
fmt(fmt(x)) == fmt(x): true

== 3. diff semântico ==
~ node api label: API -> API Gateway
~ node retry fill: amber -> none
~ node retry label: Retry -> retry
~ node retry shape: cloud -> rounded

== 4. emissão determinística ==
canvas: 439.72 x 122.5
svg bytes: 2636
svg(idempotente): true
primeira linha: <svg xmlns="http://www.w3.org/2000/svg" width="439.72" height="122.5" viewBox="0 0 439.72 122.5" role="img" aria-labelledby="akr-t akr-d" lang="pt-BR" focusable="false">
scene json bytes: 1295

== 5. acessibilidade derivada do modelo ==
desc: 2 nós, 1 aresta. Nós: API Gateway; retry.
```

Leitura do resultado:

- `segundo documento, diagnósticos: 1` — remover a declaração de `retry` faz a aresta criá-lo
  **implicitamente** (`W001`), e o rótulo passa a ser o próprio id (`Retry` → `retry`). Exatamente o
  que o diff em §3 mostra como `~ node retry label: Retry -> retry`. O modelo é honesto sobre a
  degradação.
- `fmt(fmt(x)) == fmt(x): true` — idempotência provada, não afirmada.
- `svg(idempotente): true` e `2636` bytes — a emissão é uma função pura da `Scene`.

### 4.1 Custo de compilar o harness

O harness depende de `akribie-core` por `path` e leva ~20 s na primeira compilação (deps + crate).
**Aviso verificado**: `default-features = false` **não compila** (ver §6.1); o `Cargo.toml` do harness
usa as features padrão por causa disso, com o motivo comentado no arquivo.

## 5. Performance (binário release)

### 5.1 Script do próprio projeto

```sh
$ ./scripts/bench.sh --bin /tmp/dossie-akribie-target/release/akr --runs 10
png anatomia-modulo 137.1
png boot-descoberta 129.1
png pipeline-cms-rbac 125.3
png topologia-runtime 101.4
scene anatomia-modulo 3.9
scene boot-descoberta 3.8
scene pipeline-cms-rbac 3.3
scene topologia-runtime 3.1
svg anatomia-modulo 4.3
svg boot-descoberta 3.7
svg pipeline-cms-rbac 4.6
svg topologia-runtime 4.5
```

(`ms` por execução, com as quatro fixtures reais de `fixtures/reais/`.)

### 5.2 Escala (mediana de 7 execuções, wall time de processo)

| entrada | linhas | `check` | `render` SVG | `render` scene |
|---|---|---|---|---|
| `retry.akrc` | 9 | 2,5 ms | 3,1 ms | 2,7 ms |
| `big200.akrc` (200 nós, 199 arestas) | 401 | 3,6 ms | 16,6 ms | 11,5 ms |
| `star120.akrc` (120 nós em estrela) | 241 | 3,3 ms | 10,0 ms | 7,2 ms |
| `big1000.akrc` (1000 nós, 999 arestas, 45 KB) | 2001 | 14,4 ms | 80,8 ms | 53,1 ms |

Geração dos fixtures grandes (script descartável, não versionado):

```python
lines = ['chart v1 "Big1000" direction=lr layout=flow font=sans lang=pt-BR', '']
for i in range(1000):  lines.append(f'node n{i} "N{i}" fill=paper')
for i in range(999):   lines.append(f'edge n{i} -> n{i+1}')
```

Leitura: `check` é ~5,5× mais barato que `render` em 1000 nós (14 vs 81 ms), porque não faz layout,
pintura nem emissão. Raster é ~30× o custo do SVG (o encoder domina).

## 6. O que **não** foi executado, e por quê

### 6.1 Feature `raster` desligada — **falha**, e a falha é do projeto

```sh
$ CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo build --locked -p akribie-core --no-default-features
error[E0433]: cannot find module or crate `image` in this scope
 --> crates/akribie-core/src/emit/raster.rs:7:5
error[E0432]: unresolved import `image`
error[E0433]: cannot find module or crate `tiny_skia` in this scope
error[E0433]: cannot find module or crate `usvg` in this scope
EXIT=101
```

`crates/akribie-core/Cargo.toml:12-16` documenta que desligar `raster` "deixa só
SVG/scene/excalidraw". Não existe nenhum `#[cfg(feature = "raster")]` no crate
(`grep -rn 'feature = "raster"' crates/` → vazio), e `emit/mod.rs:4` declara o módulo
incondicionalmente. Registrado como defeito em [`../arquitetura.md`](../arquitetura.md) §13.1.

### 6.2 `make check` / `make ci`

Exigem `cargo fmt --check`, `clippy -D warnings`, `cargo-deny`, `typos` instalados e `xtask` com lock
próprio (que requer rede). Rodei `cargo test --workspace --locked` (que compila com os lints de
workspace) e não reproduzi o portão completo.

### 6.3 `install.sh` / Release de 6 alvos

Requer rede e não é o objeto do dossiê.

### 6.4 `akr init` e `akr --at REV`

`akr init` escreve `akribie.toml`, um bloco em `AGENTS.md` e uma skill em `.agents/` — **não rodei
para não escrever dentro de `ref/`**. Coberto por `crates/akribie-cli/tests/init.rs` (2 testes, ok).
`--at REV` exige um repositório git com histórico; o clone tem **um commit**.

### 6.5 Teste com leitor de tela real

Não executei NVDA/VoiceOver/Orca. A conclusão de que `<desc>` em português com `lang=en` prejudica a
leitura é dedução direta do byte emitido (§7), não de teste com usuário.

## 7. Acessibilidade: o que o byte mostra

```sh
$ printf 'chart v1 "Retry" lang=en\nnode api "API"\nnode db "DB"\nedge api -> db "read"\n' > en.akrc
$ $AKR render en.akrc | sed -n '1,3p'
<svg xmlns="http://www.w3.org/2000/svg" width="404.02" height="122.5" viewBox="0 0 404.02 122.5" role="img" aria-labelledby="akr-t akr-d" lang="en" focusable="false">
<title id="akr-t">Retry</title>
<desc id="akr-d">2 nós, 1 aresta. Nós: API; DB.
```

`lang="en"` no SVG, `<desc>` em português: `a11y::describe` tem "nós"/"arestas" hardcoded
(`crates/akribie-core/src/a11y.rs:30-41`). Defeito de a11y real, não hipótese.

```sh
$ $AKR render retry.akrc | grep -o 'role="[^"]*"\|aria-label="[^"]*"\|aria-labelledby="[^"]*"'
role="img"
aria-labelledby="akr-t akr-d"
aria-label="5xx"
aria-label="backoff"
aria-label="esgotado"
aria-label="API Gateway"
aria-label="Retry ×3"
aria-label="DLQ"

$ $AKR render retry.akrc | grep -c 'tabindex'
0
```

Sem `role="list"`/`listitem` (prometidos em `docs/specs/A11Y.md:16,25`), sem `tabindex`. Ver
[`../avaliacao.md`](../avaliacao.md) §4.5.

## 8. MCP (JSON-RPC por stdio)

```sh
$ printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"chart_check","arguments":{"source":"chart v1 \"X\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n"}}}' \
  '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"chart_diff","arguments":{"old_source":"chart v1 \"X\"\nnode a\n","new_source":"chart v1 \"X\"\nnode a \"A2\"\n"}}}' \
  | akribie-mcp
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"akribie-mcp","version":"0.1.3"}}}
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"ok (0 erro(s), 0 aviso(s))","type":"text"}]}}
{"id":4,"jsonrpc":"2.0","result":{"content":[{"text":"~ node a label: a -> A2","type":"text"}]}}
```

Executado com o binário **debug** (`/tmp/dossie-akribie-target/debug/akribie-mcp`) — o release do MCP
não foi compilado.

## 9. Batch e `import` — os dois pontos de atrito verificados

### 9.1 `check --batch` sai `0` com erro parcial (mas `8` se todos falharem)

O lote abaixo tem dois itens válidos e dois inválidos, logo `exit 0`. Um lote em que **todos** os
itens falham retorna `exit 8` (`crates/akribie-cli/src/app/emit/batch.rs:21-24`).

```sh
$ cat lote.jsonl
{"name":"a","path":"retry.akrc"}
{"name":"b","source":"chart v1 \"X\"\nnode x \"X\"\nnode y \"Y\"\nedge x -> y\n"}
{"name":"c","source":"chart v1 \"Y\"\nnode z shape=nope\n"}
not json

$ $AKR check --batch lote.jsonl; echo "exit=$?"
a  ok  0 aviso(s)
b  ok  0 aviso(s)
c  erro  E003 `shape=nope` não é um valor válido
line-4  erro  invalid_input linha 4: JSONL inválido: expected ident at line 1 column 2
exit=0
```

### 9.2 `render --batch --out-dir` exige o diretório existente

```sh
$ $AKR render --batch lote.jsonl --out-dir saida --format svg
erro: diretório de saída não encontrado: `saida`
$ echo "exit=$?"   # 3

$ mkdir -p saida && $AKR render --batch lote.jsonl --out-dir saida --format svg; echo "exit=$?"
saida/a.svg  ok  8096 B
saida/b.svg  ok  1495 B
c  erro  schema E003 invalid-value: `shape=nope` não é um valor válido
line-4  erro  invalid_input linha 4: JSONL inválido: expected ident at line 1 column 2
exit=0
```

### 9.3 `import` de `.excalidraw` é lossy

```sh
$ $AKR render retry.akrc --format excalidraw > retry.excalidraw
$ $AKR import retry.excalidraw
chart v1 "retry" layout=free

node akr-api "API Gateway" at=32,70.43 fill=blue shape=rounded size=140.6,51 stroke=ink
node akr-retry "Retry ×3" at=345.04,51.96 fill=amber shape=rounded size=172.55,87.93 stroke=ink
node akr-dlq "DLQ" at=707.85,52.54 fill=red shape=rounded size=96,86.78 stroke=ink

edge akr-api -> akr-retry "5xx" style=solid
edge akr-retry -> akr-api "backoff" style=dashed
edge akr-retry -> akr-dlq "esgotado" style=dashed
```

O que mudou em relação ao original (`retry.akrc`):

| original | após import | por quê |
|---|---|---|
| `layout=flow` | `layout=free` | import não reconstrói topologia, fixa coordenadas |
| `shape=cloud`, `shape=cylinder` | todos `rounded` | `.excalidraw` não distingue as formas |
| `curve=arc` na aresta `retry -> api` | ausente | `curve` não sobrevive ao formato |
| zero coordenadas | `at=`/`size=` em todos | a geometria do SVG vira dado |
| ids `api`, `retry`, `dlq` | `akr-api`, `akr-retry`, `akr-dlq` | prefixo defensivo do importador |

## 10. Resumo de reprodutibilidade

Tudo que está colado acima pode ser reproduzido com:

```sh
cd ref/akribiechart
CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo test --workspace --locked     # 230 passam
CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo build --release -p akribie-cli
cd ../../dossie/akribiechart/exemplos
AKR=/tmp/dossie-akribie-target/release/akr
$AKR check retry.akrc && $AKR render retry.akrc | sha256sum      # 0f98809a…
$AKR check broken.akrc ; echo $?                                 # 8
$AKR diff retry.akrc novo.akrc ; echo $?                         # 1
$AKR fmt messy.akrc --check ; echo $?                            # 1
cd harness && CARGO_TARGET_DIR=/tmp/dossie-akribie-target cargo run --offline --quiet
```

Nenhuma saída neste arquivo foi inventada ou editada além de quebras de linha e elipses marcadas
com `…`/`| head`.
