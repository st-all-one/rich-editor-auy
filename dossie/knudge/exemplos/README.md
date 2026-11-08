# `exemplos/` — o ciclo real do `kd`, executado

## O que há aqui

| Arquivo | O que é | Origem |
|---|---|---|
| `ciclo-kd.sh` | script executável que funda um corpus temporário e exercita init → write → dedup → ask → task → rewind → doctor → sync → `--json` → exit codes | escrito para este dossiê, usa apenas o binário `kd` público |
| `nota-gerada.md` | **bytes reais** de uma nota produzida por `kd write` (com corpo, duas âncoras e tag) | gerada localmente por `kd 0.6.4`, não editada à mão |

## Como rodar

Pré-requisito: `kd` no PATH (aqui: `/home/one/.local/bin/kd`, versão `0.6.4`). Se não houver,
a instalação oficial é a da quickstart do projeto (`ref/knudge/wiki/usage/00_quickstart.md:61-70`).

```sh
cd /home/one/projects/one/rich-editor-auy/dossie/knudge/exemplos
bash ciclo-kd.sh
```

O script cria `mktemp -d`, roda tudo lá dentro e apaga no `EXIT` — **não toca neste
repositório**. `git` é inicializado no temporário com `commit.gpgsign false` para que `kd sync`
não dependa da config global da máquina.

## Saída observada (execução real, 2026-11-07)

```
== projeto temporário: /tmp/knudge-exemplo-wEGYUp ==
== kd init ==
projeto knudge-exemplo-wEGYUp fundado em /tmp/knudge-exemplo-wEGYUp/.knudge
estrutura: notas/ (novo), eventos/ (novo), .idx/ (novo), cache/ (novo)
config.toml (novo)
.git/info/exclude (atualizado)
== kd write (primeira nota) ==
created|fact_01swd4xp|r1
== kd write (mesmo statement: idempotente) ==
unchanged|fact_01swd4xp|r1
== kd write (quase-duplicata: merge) ==
merged|fact_01swd4xp|r2
== kd ask ==
fact_01swd4xp|O editor precisa de um gate único de qualidade|0.51|semantic
== kd write --dry-run (não persiste) ==
dry-run: decision_01lenugg | slots ausentes: alternativas, por quê, consequência
== kd task ==
epic=epic_018zo6w7
task_00f7hqtk
epic_018zo6w7|epic|active|Fundação do editor
task_00f7hqtk|task|active|Escrever o gate de CI
== kd rewind (handoff) ==
notes=3 ready=1 blocked=0 containers=1 dirty
recent: task_00f7hqtk epic_018zo6w7 fact_01swd4xp
next: task_00f7hqtk|Escrever o gate de CI
fresh: stale=0 expiring=0 pending=0
== kd doctor --fix ==
próximos:
  - kd doctor --explain (body)
  - kd doctor --explain (program-anchor)
== kd sync (commit de notas/ + eventos/) ==
sync main: 2 arquivo(s) commitados (3567299)
== contrato de máquina: --json é JSON válido em stdout ==
json ok: ask hits= 1
== exit codes observados ==
id ausente -> 0
verbo inválido -> 2
statement vazio -> 2
EPIPE -> 0
== arquivos do corpus (verdade = notas/, resto derivado) ==
.knudge/notas/epic/epic_018zo6w7.md
.knudge/notas/fact/fact_01swd4xp.md
.knudge/notas/task/task_00f7hqtk.md
== ok ==
SCRIPT EXIT=0
```

Nada foi inventado: a saída acima é a transcrição da execução (o sufixo do diretório temporário varia a cada rodada). Os ids são derivados de hash e
por isso mudam a cada projeto/corpus; o hash do commit do `kd sync` também muda a cada execução.

## `nota-gerada.md` — proveniência exata

Gerada com:

```sh
kd init
kd write --summary "A seleção sobrevive a um round-trip de serialização" \
    --type fact --tag selecao \
    --anchor src/selection.rs --anchor src/serializer.rs <<'BODY'
Por quê: offsets de seleção podem apontar para o meio de um grafo de nós se a serialização
reordenar ou colapsar nós vazios.
Evidência: proptest de round-trip com seleção não vazia.
Consequência: todo nó precisa de chave estável para a seleção ser endereçável.
BODY
```

Resultado (`created|fact_01e7uvl1|r1`) e bytes preservados em `nota-gerada.md`.

Observações que o arquivo prova sem precisar de código:

- a ordem das chaves é canônica (`id, type, statement, created_at, body_hash, schema_version,
  tags, anchors`), e as chaves opcionais ausentes são **omitidas**, nunca `null`;
- `body_hash` é 8 caracteres hex, derivado de `normalize(statement) + LF + normalize(body)`;
- `created_at` é UTC com milissegundos e `Z`;
- o corpo começa **sem** título nem heading: o `statement` é a afirmação; o corpo é só o que a
  afirmação não diz.

O contrato completo está em `ref/knudge/wiki/specs/TOON.md:76-93` e as 31 chaves em
`ref/knudge/crates/knudge-core/src/schema/keys.rs:7`.

## O que **não** foi executado (e por quê)

| Item | Estado | Motivo / como verificar |
|---|---|---|
| `cargo build` / `cargo test` do `ref/knudge` | **não executado** | É um workspace de 3 crates; a tarefa proíbe build de monorepo. Verificação correta: `cd ref/knudge && make check` (exige rede para baixar deps e ~minutos de compilação). |
| `make ci` (miri, deny, audit, machete, typos) | **não executado** | `cargo-miri`, `cargo-deny`, `cargo-audit`, `cargo-machete` e `typos` não estão instalados nesta máquina; os alvos pulariam com aviso (`Makefile:64-128`). |
| `make bench` | **não executado** | Compila `bench/` em release. Os números citados no dossiê vêm do artefato versionado `ref/knudge/bench/ULTIMO.md`, não de reexecução. |
| `cargo fuzz` | **não executado** | Exige nightly + `cargo-fuzz`; alvos em `ref/knudge/fuzz/fuzz_targets/`. |
| Gate de tamanho | **executado** | `bash ref/knudge/scripts/check_file_length.sh` → `file-length: ok (limite 300)`, exit 0. É bash puro, sem deps. |
| Ciclo `kd` (init/write/ask/task/rewind/doctor/json/exit) | **executado** | Seções acima; binário `kd 0.6.4` já instalado. |
| Concorrência (4 writes paralelos) | **executado** | 4 notas distintas, sem lost update; ver [`../avaliacao.md`](../avaliacao.md). |
| `kd sync` | **executado** | O script roda `kd sync` com `commit.gpgsign false` no temporário: `sync main: 2 arquivo(s) commitados`, exit 0. A falha por GPG da config global foi reproduzida à parte (exit 70), com o stderr do git preservado. |
