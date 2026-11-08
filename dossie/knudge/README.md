# knudge — estudo de caso de **processo** (não de editor)

> `ref/knudge/` é uma CLI Rust (`kd`, 16 subcomandos — 15 verbos + `help`) de memória durável por projeto para agentes de
> IA. Ela **não edita texto rico** e não tem DOM, seleção, IME, CSS ou renderer. O motivo de
> estar neste dossiê é outro, e é explícito: **o knudge é o exemplar mais disciplinado do `ref/`
> em planejamento, gates, teste de contrato e documentação de decisão** — exatamente as partes
> que o `rich-editor-auy` precisa acertar antes de escrever o primeiro `Node`.

A comparação "como editor" está fora de escopo e é tratada em
[`avaliacao.md` §0](avaliacao.md#0-escopo-e-ressalva-a-comparação-como-editor-não-se-aplica).

---

## 1. Ficha

| Item | Valor | Evidência |
|---|---|---|
| Versão observada | `0.6.4` (binário instalado e fonte) | `kd self version` → `kd 0.6.4`; `ref/knudge/Cargo.toml:9` |
| Toolchain | Rust 1.99, Edition 2024, MSRV **obrigatória** | `ref/knudge/rust-toolchain.toml:4`, `ref/knudge/Cargo.toml:10-11` |
| Licença | `MIT OR Apache-2.0` (cópia integral permitida com aviso) | `ref/knudge/Cargo.toml:12`, `ref/knudge/LICENSE-MIT`, `ref/knudge/LICENSE-APACHE` |
| Crates | 3 (`knudge-core` puro, `knudge-cli`, `knudge-mcp`) + `fuzz/` e `bench/` fora do workspace | `ref/knudge/Cargo.toml:4-6` |
| Tamanho medido | 424 arquivos `.rs` em `crates/`, 59.782 linhas | `find crates -name '*.rs' \| wc -l`; `find crates -name '*.rs' -print0 \| xargs -0 cat \| wc -l` |
| Testes | 963 atributos `#[test]` (funções), 20 blocos `proptest!` | `grep -rho '#\[test\]' crates \| wc -l`; idem `proptest!`. Não é o nº de casos executados: cada bloco `proptest!` gera casos em runtime |
| Gate de 300 linhas | **verde**; maior arquivo de produção = **300 linhas** (`adapters/http.rs`), exatamente no limite; o segundo é `health/validator/mod.rs` com 294 | `bash scripts/check_file_length.sh` → `file-length: ok (limite 300)`, exit 0 |
| Decisões registradas | 225 ids `Dxx` distintos citados em `plan/`+`wiki/`; o arquivo de decisões vai até **D224** (213 ids) | `grep -rhoE '\bD[0-9]{1,3}\b' plan wiki \| sort -u \| wc -l` |
| Políticas de engenharia | `R01–R45` | `ref/knudge/plan/implementation/14_revisao_tecnica.md:241` (`R45`) |
| Bordas catalogadas | **120 bordas numeradas** (1–121; o nº 116 não existe) em 145 linhas | `grep -cE '^\| [0-9]+ ' ref/knudge/wiki/specs/DIVERGENCES.md` |

Nada disso é "projeto grande" por vaidade: 60 mil linhas para uma ferramenta de memória só se
sustentam porque **cada linha tem um gate, uma decisão e um teste apontando para ela**. É esse
o objeto do dossiê.

---

## 2. Por que está no dossiê

O `rich-editor-auy` é um editor headless em Rust com front-end TS/PHP. Ele vai acumular
exatamente os artefatos que o knudge já resolveu:

| Problema do nosso projeto | Como o knudge resolve | Onde ver |
|---|---|---|
| "O modelo de documento é byte-contrato?" | `TOON.md` + goldens + proptest de round-trip byte-exato | [`arquitetura.md` §7](arquitetura.md#7-camada-de-testes-golden-proptest-stress-crash) |
| "Onde registro a decisão de seleção/IME?" | `Dxx` fechadas + `Rn` de engenharia, separadas por natureza | [`arquitetura.md` §2](arquitetura.md#2-duas-espécies-de-registro-dxx-e-rnn) |
| "Como impedir arquivo de 4.000 linhas?" | gate de 300 linhas no `make check` e no CI | [`arquitetura.md` §5](arquitetura.md#5-disciplina-de-código) |
| "Como não apodrecer em silêncio?" | `doctor` + `DIVERGENCES.md` + matriz de aceite por verbo | [`arquitetura.md` §8](arquitetura.md#8-matriz-de-aceite-e-documentação-em-camadas) |
| "Como testar crash e concorrência?" | `FaultyFs`, double-buffer, stress de lock | [`arquitetura.md` §7](arquitetura.md#7-camada-de-testes-golden-proptest-stress-crash) |

O que **não** transferir (BM25, RRF, embeddings, dedup por similaridade, MCP) está isolado em
[`arquitetura.md` §10](arquitetura.md#10-o-que-é-específico-de-memóriaia-e-não-transfere).

---

## 3. O que o produto faz — e o que ele *não* faz

O knudge externaliza **conhecimento de projeto** (decisões, fatos, erros, riscos) em notas
Markdown com frontmatter TOON, versionadas no git, e oferece um ciclo:

```
kd ask (busca) → kd write (grava) → kd task (trabalho) → kd sync (commit)
```

- `ref/knudge/wiki/usage/00_quickstart.md:160-163` descreve o ciclo; `ref/knudge/README.md` é a
  porta de marketing.
- `notas/` é a verdade; `eventos/` é auditoria; `.idx/` é **derivado e descartável**
  (`ref/knudge/wiki/specs/ARCHITECTURE.md:90-112`).
- O "usuário" declarado é o **LLM**: `ref/knudge/plan/00_panorama.md:9-21` diz explicitamente
  que legibilidade humana é irrelevante e custo é contexto, não disco. Isso explica por que
  quase nenhuma decisão de UX de editor aparece aqui.

**Não é:** editor, wiki multiusuário, indexador de PDFs, RAG genérico, servidor. A própria
quickstart lista quando **não** adotar (`wiki/usage/00_quickstart.md:11-46`).

---

## 4. O que o usuário vê (execução real, não README)

Tudo abaixo foi executado nesta máquina com `kd 0.6.4` num projeto temporário (`mktemp -d`),
fora deste repositório.

### 4.1 Init

```
$ kd init
projeto kd-dossie-tzIm8Z fundado em /tmp/kd-dossie-tzIm8Z/.knudge
estrutura: notas/ (novo), eventos/ (novo), .idx/ (novo), cache/ (novo)
config.toml (novo)
.git/info/exclude (atualizado)
.gitattributes (atualizado)
AGENTS.md (novo)
.agents/skills/kd/SKILL.md (novo)
```

(Recorte: depois dessas 7 linhas de status, `kd init` ainda imprime `próximos:` e o protocolo
`prime` completo — ~40 linhas no total. O script de `exemplos/` corta em 4 linhas por brevidade.)

Cria 11 diretórios em `notas/` (10 tipos de conhecimento — `fact`, `decision`, `question`,
`task`, `def`, `error`, `snippet`, `link`, `meta`, `risk` — mais `epic/` para containers de
tarefa), escreve `config.toml` com 69 chaves efetivas
(`kd config list` → `project (69)`), instala o bloco gerenciado no `AGENTS.md` e uma skill do
agente. Reexecutar é **idempotente** (`inalterado` em tudo, exit 0); `--force` só reescreve a
config.

### 4.2 Primeira gravação

```
$ kd write --summary "O gateway limita 100 rps por chave" --type fact \
    --tag gateway --anchor src/gateway.rs
created|fact_0094if9p|r1
```

O arquivo produzido (`exemplos/nota-gerada.md` é um exemplo real, gerado igual):

```
---
id: fact_0094if9p
type: fact
statement: O gateway limita 100 rps por chave
created_at: 2026-11-08T01:16:59.123Z
body_hash: bb042070
schema_version: 3
tags: [gateway]
anchors: [src/gateway.rs]
---
```

Ordem de chaves canônica, opcionais omitidos, `id` derivado do conteúdo. Isso é contrato, não
estilo: `ref/knudge/wiki/specs/TOON.md:76-88` e `crates/knudge-core/src/schema/keys.rs:7`
(`CANONICAL_KEYS` com 31 chaves).

### 4.3 Dedup em três respostas (observado)

| Comando | Saída | Significado |
|---|---|---|
| mesmo `statement` | `unchanged\|fact_0094if9p\|r1` | idempotente por conteúdo |
| quase-duplicata | `merged\|fact_0094if9p\|r2` | 0,75–0,92 → funde e incrementa `revision` |
| duplicata exata com outro `type` | `rejected\|fact_0094if9p` | ≥0,92 → rejeita |

Limiares vêm de `[dedup]` em `config.toml` (`create_below = 0.75`, `merge_below = 0.92`) e são
**configuração, não runtime** — coerente com "fixo vs. oscilante"
(`ref/knudge/plan/00_panorama.md:434-447`).

### 4.4 Busca, tarefas e retomada

```
$ kd ask "qual o limite do gateway"
fact_0094if9p|O gateway limita 100 rps por chave|0.51|semantic

$ kd task new --summary "Migrar para schema V2" --scope epic --anchor plan/v2.md
epic_000yogwp
$ kd task list --universe
epic_000yogwp|epic|active|Migrar para schema V2
task_01t4o48u|task|active|Escrever migrador

$ kd rewind
notes=3 ready=1 blocked=0 containers=1 dirty
recent: task_01t4o48u epic_000yogwp fact_0094if9p
next: task_01t4o48u|Escrever migrador
fresh: stale=0 expiring=0 pending=0
```

`rewind` é o handoff de sessão: manifest com contadores + recentes + próximos, dentro de um
orçamento de tokens sem tokenizer (`ceil(chars/4)`).

### 4.5 Saúde do corpus

`kd doctor` roda 13 checks e sai **0 mesmo com `fail`**; `--fix` corrige o derivado
(idempotente — a segunda execução não muda nada). Observado:

```
fail derived índice derivado ausente/divergente; `--fix` reconstrói
warn body 1 sem corpo; 1 sem lastro (corpo+outcome+âncora); 0 com slots ausentes
auditoria: 1 problema(s)
```

Ou seja: diagnóstico é **informativo por padrão**; quem decide o exit é a taxonomia de erro, não
a saúde do corpus.

### 4.6 Contrato de máquina e erros

| Caso | Observado |
|---|---|
| `kd ask --json 2>/dev/null` | JSON válido; `warnings` só aparece quando existe |
| `kd --json` sem verbo | `{"success":false,...}` + exit **2** |
| verbo inválido | exit **2** |
| `kd write --summary ""` | `entrada inválida: statement vazio` + exit **2** |
| `kd write --type task` | recusa com D93 + exit **2** |
| `kd prime \| head -1` | EPIPE → exit **0** |
| `kd ask --id <inexistente>` | aviso em stderr, exit **0** (degradação graciosa) |
| `kd task close` sem evidência | `sem validators; use --outcome` + exit **2** |
| `kd write --batch` com chave inválida | `aviso: linha 1: ... chave de rascunho desconhecida: summary` + exit **0** (erro de schema em lote vira aviso, não falha) |
| `kd sync` com git exigindo assinatura | exit **70** (`internal`) com o stderr do git colado |

O último caso é achado real: falha de `git commit` é classificada como `Internal (70)`, não
`Io (5)` nem `Config (7)`. Funciona para diagnosticar (a mensagem do git aparece), mas
"internal" normalmente significa "bug nosso" — um `git` mal configurado não é.

### 4.7 Performance observada

- `kd ask` (corpus de 2–5 notas): 0,04–0,05 s a quente, até 0,14 s em execuções isoladas
  (mediana ~0,05 s, `/usr/bin/time`, 10 execuções).
- Piso de startup: 3,62 ms de mediana (`ref/knudge/bench/ULTIMO.md:76`).
- `ask` (query comum, corpus de 200 notas): 59,6 ms de mediana (`bench/ULTIMO.md:83`).
- `Index::build` a 1.000 notas: 10,2 ms; `recall limit 5`: 2,33 ms (`bench/ULTIMO.md:52`, `:57`).

A bancada é **observação, não gate** (`ref/knudge/plan/implementation/13_testes_qualidade.md:97`).

### 4.8 Concorrência

Quatro `kd write` em paralelo no mesmo projeto produziram **4 notas distintas**, sem lost update
(`ls .knudge/notas/fact | wc -l` → 4), e `doctor` não acusou lock stale. O lock é advisory por
arquivo-alvo com reclaim por `rename` (`DIVERGENCES.md:22`).

---

## 5. Prós

1. **Gate único, barato e obrigatório**: `make check` = fmt + clippy + test + tamanho
   (`ref/knudge/Makefile:11`). Uma linha, sem ambiguidade.
2. **Rigor com motivo, não com moda**: cada lint proibido tem `reason` em `clippy.toml`, e o
   porquê está documentado (376 linhas de análise em `15_clippy_config.md`).
3. **Decisão ≠ política**: `Dxx` (produto) e `Rn` (engenharia) separados; a matriz de bloqueio
   diz **em que ordem** as decisões precisam existir (`plan/02_decisoes.md:172-198`).
4. **Contrato de bytes testado de verdade**: round-trip byte-exato, goldens versionados,
   `proptest` e crash-injection.
5. **Bordas catalogadas com o teste que as trava**: `DIVERGENCES.md` tem coluna "teste";
   nenhuma borda é "conhecida" sem prova.
6. **Matriz de aceite por verbo**: pipe, `--json`, erro/exit, estado em disco e teste
   (`17_matriz_aceitacao.md:14-76`).
7. **Documentação em camadas com dono**: `MODULE.md` por crate, `wiki/specs` (técnica),
   `wiki/usage` (tarefa), `llms.txt` (índice para modelo), `AGENTS.md` (contribuição).
8. **Release sem improviso**: workflow verifica `tag == Cargo.toml`, matriz nativa, checksums, e
   `install.sh` confere SHA-256 antes de instalar (`install.sh:250-262`).
9. **Erros como valores**, com mapa código→exit congelado e 101 reservado a panic
   (`crates/knudge-core/src/error.rs:1-8`).

## 6. Contras

1. **Custo de processo alto para o tamanho do produto**: 225 ids de decisão citados (213 entradas
   no arquivo de decisões, até `D224`) e 60 mil linhas para uma CLI de memória. Parte disso é
   aprendizado genuíno; parte é cerimônia.
2. **Deriva documental real e observada**: a matriz afirma `config (7)` ao reexecutar `kd init`
   sem `--force` (`17_matriz_aceitacao.md:23`), mas o código diz "idempotente"
   (`knudge-cli/src/commands/init.rs:58`) e a execução dá **exit 0**. A matriz não é gerada a
   partir do código, então envelhece.
3. **Números de teste envelhecem nos índices**: `plan/implementation/README.md:85` fala em
   "458 testes"; o comentário do alvo `miri` (`Makefile:100`) fala em "600+"; a contagem atual é
   **963 funções `#[test]`**. São métricas diferentes (funções vs casos executados), mas nenhuma
   é validada por gate. A mesma deriva aparece nos limites de decisão: `AGENTS.md:211-212`
   ainda diz `D01–D216` e `R01–R44`, quando o arquivo de decisões vai até `D224` e a revisão
   técnica até `R45`.
4. **`doctor` sai 0 com `fail`**: bom para script que só quer inspecionar, ruim para CI que quer
   "corpus saudável" — exige `grep`/`--json`.
5. **Classificação de erro imprecisa em integrações externas** (`sync` → `internal 70`).
6. **`tokio`/`reqwest` banidos por `deny.toml:35`** casam com o domínio local, mas seriam um
   empecilho se o nosso editor precisar de async de rede no core.
7. **Não é biblioteca de UI**: zero evidência de renderer, seleção, IME, a11y de tela. O
   dossiê só pode falar de processo.

---

## 7. Lista curta de práticas adotáveis (detalhe em `avaliacao.md` §5)

1. Um `make check` que ninguém precisa lembrar: `fmt` + `clippy -D warnings` + `test` + gate de
   tamanho.
2. Gate de **300 linhas por arquivo de produção** — barato, objetivo, força fatiar por
   responsabilidade antes de virar bola de neve.
3. `clippy.toml` + `[workspace.lints]` em **duas camadas**, com `reason` obrigatório em todo
   `#[allow]`.
4. `DIVERGENCES.md`: cada borda (Unicode, ordem, seleção, IME, serialização) com **o teste que a
   trava**.
5. Serialização com **golden + proptest de round-trip byte-exato** antes de qualquer feature.
6. Matriz de aceite por comando/API: saída de texto, JSON, erro, exit code e efeito em disco.
7. Crash-injection e stress de concorrência com FS falso que injeta falha.
8. Registro `Dxx` (decisão) separado de `Rn` (política), com matriz de bloqueio e teste que
   trava cada decisão.
9. Release com `tag == versão` verificado no CI e checksum no instalador.
10. `AGENTS.md` + `MODULE.md`: mapa "quero X → vá para Y", mantido em sincronia com os módulos.

---

## 8. Mapa do dossiê

| Arquivo | Conteúdo |
|---|---|
| [`arquitetura.md`](arquitetura.md) | arquitetura **de processo**: camadas de artefato, gates, lints, testes, documentação, release, o que é específico de IA |
| [`avaliacao.md`](avaliacao.md) | matriz de usabilidade adaptada, notas 0–5 com evidência, falhas encontradas, práticas adotáveis |
| [`codigo/INDEX.md`](codigo/INDEX.md) | 6 arquivos reais copiados: `Makefile`, `ci.yml`, `check_file_length.sh`, `clippy.toml`, `Cargo.toml`, `error.rs` |
| [`exemplos/README.md`](exemplos/README.md) | exemplo executável (`ciclo-kd.sh`) com saída observada e nota real gerada |
