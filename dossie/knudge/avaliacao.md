# Avaliação — rigor, manutenção e o que se aproveita

> **Leia primeiro a §0.** O knudge não é editor; compará-lo como editor produziria uma tabela
> de zeros inúteis. A avaliação abaixo mede o que o projeto **é**: um processo de engenharia
> aplicado a um domínio pequeno, com execução real e citações.

---

## 0. Escopo e ressalva: a comparação como editor **não se aplica**

O knudge:

- não tem DOM, `contenteditable`, renderer, CSS, layout, seleção, cursor, IME, clipboard,
  undo/redo de edição, colaboração em tempo real ou ARIA (`ref/knudge/Cargo.toml:4-6` lista três
  crates de CLI; nenhum contém UI);
- não serializa documento rico — serializa **nota** (frontmatter TOON + corpo Markdown);
- não é biblioteca de front-end: `knudge-core` é biblioteca de domínio
  (`ref/knudge/wiki/specs/ARCHITECTURE.md:16-36`).

Portanto, nas dimensões do guia específicas de editor (edição de texto, seleção/IME, CSS,
a11y de tela), a resposta honesta é **"não se aplica"** — e onde existe um análogo estrutural
(persistência, schema, contrato de saída, concorrência), o análogo foi avaliado com evidência.
A nota de "adequação ao nosso caso" (§3) reflete exatamente isso: alto em **processo**, zero em
**produto de editor**.

O que foi executado nesta avaliação: `kd 0.6.4` (binário instalado), em projeto temporário;
`bash scripts/check_file_length.sh` no `ref/`. O que não foi: build/test/lint do workspace
(proibido pela tarefa) — ver [`exemplos/README.md`](exemplos/README.md#o-que-não-foi-executado-e-por-quê).

---

## 1. Matriz de usabilidade adaptada

Legenda de status conforme o guia: `código`, `teste`, `exemplo`, `doc`, `não verificado`,
`N/A`. "N/A" só aparece quando a dimensão não tem análogo estrutural; quando tem, o análogo foi
marcado.

| # | Dimensão do guia | Equivalente neste projeto | Status | Evidência |
|---:|---|---|---|---|
| 1 | Instalação e pré-requisitos | binário único, `curl \| bash` com checksum, Rust 1.99 só para source | **doc + execução** | `wiki/usage/00_quickstart.md:61-99`; `install.sh:250-262` (SHA-256 antes de instalar); binário `kd 0.6.4` já presente e executado |
| 2 | Inicialização mínima | `kd init` cria `.knudge/`, config, bloco no `AGENTS.md`, skill | **exemplo** | execução real: 7 linhas de status + `próximos:` + bloco `prime`, exit 0; `exemplos/README.md` §"Saída observada" |
| 3 | Edição básica (texto, negrito, link) | **N/A como editor**; análogo = criar unidade de conteúdo (`kd write`) | **exemplo** | `kd write` → `created\|fact_0094if9p\|r1`, nota de 8 chaves em disco (§4.2 do README) |
| 4 | Seleção, teclado, undo/redo, clipboard, IME | **N/A**; análogo = idempotência, `revision`, `--update`, supersessão | **exemplo + código** | `unchanged`/`merged`/`rejected` observados; `knudge-core/src/write/` descrito em `MODULE.md:228-244`; sem supersessão reexecutada |
| 5 | Schema e restrições de conteúdo | contrato de bytes TOON: 31 chaves canônicas, enums fechados, opcionais omitidos | **código + exemplo** | `schema/keys.rs:7` (`CANONICAL_KEYS: [&str; 31]`), `schema/mod.rs:28` (`SCHEMA_VERSION = 3`), nota real em `exemplos/nota-gerada.md`; `wiki/specs/TOON.md:76-93` |
| 6 | Custom node/mark/plugin | **N/A**; análogo = hooks, `validators.toml`, `Embedder` plugável, config | **doc + código** | `ARCHITECTURE.md:232-244` (hooks `pre-record`/`post-record`/`pre-prune`/`pre-compact`, `ProcessHookRunner` sem shell); `MODULE.md:27-43` (tabela portas/fakes/adaptadores) |
| 7 | Persistência e serialização | `notas/` verdade, `eventos/` auditoria, `.idx/` derivado; escrita atômica | **exemplo + doc + teste citado** | disco real: `.knudge/notas/{fact,epic,task}/*.md`; `ARCHITECTURE.md:90-112`; `DIVERGENCES.md:20-22` (tmp+rename, nota-antes-evento, double-buffer) |
| 8 | Renderização e não interferir no CSS | **N/A**; análogo = contrato de saída (pipe, `--json`, stdout≠stderr) | **exemplo** | `kd ask` emite `id\|statement\|score\|why`; `--json 2>/dev/null` validado com `json.load`; EPIPE → 0; `DIVERGENCES.md:28-30` |
| 9 | Acessibilidade (ARIA, foco, leitor de tela) | **N/A** (sem tela); análogo = acessibilidade de máquina | **exemplo + doc** | exit codes estáveis (`wiki/specs/erros.md`), `--json` para agente, `kd` == `kd help`, help por verbo; nada de ARIA porque não há UI |
| 10 | Colaboração | multi-agente por **shards** `.knudge/ai/<area>/<agent_id>.jsonl`, `sync` git, lock advisory | **doc + execução parcial** | `wiki/specs/fontes.md` (citado em `specs/README.md:14`); `CHANGELOG.md:26-32` (memória v2 em shards); 4 writes concorrentes sem lost update; **sem** colaboração em tempo real |
| 11 | Performance e limites | startup ~3,6 ms; `ask` 59,6 ms @200 notas; `recall` 2,33 ms @1000 | **doc (artefato) + execução** | `bench/ULTIMO.md:76` (p95 3,96 ms), `:83` (mediana 59,574 ms), `:52`, `:57`; `kd ask` local 0,04–0,05 s a quente (até 0,14 s isolado); bancada é observação, não gate (`13_testes_qualidade.md:97`) |
| 12 | Erros, migração e diagnóstico | taxonomia `ErrorKind` → exit; `doctor` 13 checks; leitura tolerante; `--fix` idempotente | **exemplo + código** | `error.rs:14-70`; execuções: verbo inválido 2, statement vazio 2, EPIPE 0, id ausente 0 com aviso; `doctor --fix` duas vezes sem mudança; `DIVERGENCES.md:121` (chave aditiva não quebra rebuild) |
| 13 | Licença e maturidade | `MIT OR Apache-2.0`; 0.6.4; changelog por versão; 225 decisões | **código + doc** | `Cargo.toml:9,12`; `wiki/CHANGELOG.md:1-30`; contagem de `Dxx` observada |
| 14 | *(extra)* Concorrência | lock por arquivo-alvo, reclaim por `rename`, stress com FS real | **exemplo + doc/teste citado** | 4 writes paralelos → 4 notas; `DIVERGENCES.md:22`; `crates/knudge-core/tests/stress.rs` (`13_testes_qualidade.md:106-110`) |
| 15 | *(extra)* Unicode | NFC/trim/colapso alimentam `id` e `body_hash`; fold de diacríticos só no índice | **código + teste citado** | `DIVERGENCES.md:12`; `ARCHITECTURE.md:153` (fold no índice); **não** há IME — logo, metade desta dimensão é N/A |

**Cobertura (contagem honesta):** das 15 linhas, **10 são dimensões plenamente aplicáveis** ao
projeto e todas as 10 têm evidência (código, teste citado, exemplo executado ou doc com
`caminho:linha`) — **10/10 = 100% das aplicáveis**, acima da meta de 85%. As outras **5 são
dimensões de editor** (edição rica, seleção/IME/undo, plugin de nó/mark, CSS, ARIA): o knudge
**não as implementa** e elas aparecem na tabela apenas com um **análogo estrutural** (edição →
`write`; seleção/IME → idempotência/`revision`; plugin → hooks/validators; CSS → contrato de
saída; ARIA → acessibilidade de máquina). Elas **não entram no numerador da meta** — não são
cobertura de usabilidade de editor, são o que o projeto tem no lugar.

> A meta de 85% do guia pressupõe um editor. Aplicada literalmente a um projeto sem UI, ela é
> inatingível por definição; por isso a contagem acima separa "aplicável com evidência" (10/10)
> de "N/A como editor" (5). Nenhuma linha foi marcada como "verificada" sem arquivo, teste ou
> execução correspondente, e nenhuma nota de editor foi inflada por ausência de UI.

---

## 2. Profundidade da inspeção

| Área | O que foi lido/executado |
|---|---|
| Processo | `AGENTS.md` (14,8 KB), `plan/implementation/README.md`, `01_fundacao_workspace.md`, `13_testes_qualidade.md`, `14_revisao_tecnica.md`, `15_clippy_config.md` (§0–§2), `17_matriz_aceitacao.md`, `24_plano_mestre_0.5.0.md`, `02_decisoes.md` (§matriz e §10 decisões), `03_decisoes-fechadas.md` (A–G) |
| Gates | `Makefile` integral, `ci.yml`, `release.yml`, `check_file_length.sh`, `bump-version.sh`, `package.sh` |
| Config | `Cargo.toml`, `clippy.toml` integral, `deny.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `_typos.toml` |
| Código | `error.rs` integral, `store/commit.rs`, `lib.rs`, `schema/keys.rs`, `schema/mod.rs`, `write/draft.rs` (trechos), `write/batch.rs` (topo), `tests/invariants.rs`, árvore de `crates/*/src` com contagem de linhas |
| Docs | `ARCHITECTURE.md` integral, `DIVERGENCES.md` integral (120 bordas em 145 linhas), `TOON.md` (§1–§5), `erros.md`, `specs/README.md`, `usage/00_quickstart.md`, `usage/19_troubleshooting.md` (estrutura), `CHANGELOG.md` (topo), `llms.txt` (topo), `MODULE.md` do core |
| Execução | `kd init`, `write` (create/unchanged/merge/reject/dry-run/batch), `ask` (texto/JSON/`--id`/limite), `task new/list/close`, `rewind`, `doctor`/`--fix`, `map`, `config list`, `self version`, `forget`, `sync` (sucesso e falha por GPG), EPIPE, 4 writes concorrentes, gate de 300 linhas |

São **~45 inspeções/execuções**, acima do mínimo de 25–40.

---

## 3. Notas 0–5 (com evidência)

| Dimensão | Nota | Evidência | Comentário |
|---|---:|---|---|
| **Usabilidade** | **3** | `kd init` → relatório legível; `write` → `created\|id\|r1`; quickstart de 5 min (`00_quickstart.md:61-163`); `doctor` com `próximos:`; mas `write --batch` rejeitou `summary` (a chave correta é `statement`, `write/draft.rs:228-232`) sem lista de chaves válidas no erro — e sai **0**, com aviso | Excelente para o **usuário declarado** (LLM/pipe). Para humano, a superfície de 15 verbos e a distinção `--summary` vs chave JSON `statement` custam. O erro de batch é claro sobre a linha, não sobre o vocabulário. |
| **API** | **4** | `ErrorKind`/`exit_code` (`error.rs:14-70`); `Knudge::builder()` como fachada de biblioteca (`lib.rs:7-19`); envelope `{success, command, data?, error{code,message,retryable}, warnings?}`; `--json` validado | Contrato de máquina é o ponto mais forte. Perde 1 por: `retryable` só para `Timeout` (útil, mas estreito), e porque a API pública do core é grande (23 módulos, `lib.rs:33-55`) sem versionamento semântico explícito além de `SCHEMA_VERSION`. |
| **Neutralidade de estilo** | **N/A** *(análogo: 5)* | Não há CSS nem renderer. Análogo: stdout=dados / stderr=logs (`R20`), pipe e `--json` (`DIVERGENCES.md:28`), EPIPE→0 (`:30`) | Como editor, **0 por inexistência**. Como contrato de saída, 5: o knudge nunca mistura log com dado e trata pipe fechado. Não há como "o estilo do host vazar". |
| **Modelo de dados** | **5** | 31 chaves canônicas (`schema/keys.rs:7`), `SCHEMA_VERSION = 3` (`schema/mod.rs:28`), id por conteúdo (`D01`), `body_hash` com NFC (`D06`), opcionais omitidos (`D05`), round-trip byte-exato (proptest citado em `TOON.md:93`) | O melhor artefato do projeto. A nota real em `exemplos/nota-gerada.md` mostra o contrato em 10 linhas. Nada aqui é "mais ou menos": ordem, hash e normalização são testados. |
| **Extensibilidade** | **3** | Portas + adapters (`ARCHITECTURE.md:53-63`), hooks (`:232-244`), `validators.toml` (`:194-206`), `Embedder` plugável (`:219-231`), config em 2 níveis | Extensível **por dentro** (trocar FS/Clock/Git/Embedder, adicionar validator). Não extensível **por fora**: sem plugins de terceiros, enums fechados por decisão (`D14`), sem API de extensão de tipo de nota. Para um editor isso seria insuficiente se quiséssemos nós customizados por usuário. |
| **Performance** | **4** | startup 3,62 ms mediana / 3,96 ms p95 (`bench/ULTIMO.md:76`); `ask` 59,6 ms @200 (`:83`); `recall` 2,33 ms @1000 (`:57`); `Index::build` 10,2 ms @1000 (`:52`); 4 writes concorrentes sem perda | Números existem, são reproduzíveis e têm harness dedicado (`make bench`). Perde 1 porque: benchmark é **observação**, não gate (`13_testes_qualidade.md:97`), e não há budget em CI — uma regressão de 2× só é vista se alguém rodar a bancada. |
| **Acessibilidade** | **N/A** *(análogo: 3)* | Sem UI, ARIA, foco ou leitor de tela. Análogo: exit codes estáveis, `--json`, `kd` == `kd help`, help por verbo, mensagens em pt-BR com código ASCII | Como editor: **0**. Como CLI: 3 — ótimo para script/agente, mas dependente de locale (mensagens em português) e sem `--json` uniforme em todos os verbos (ex.: `map` tem formato próprio). |
| **Documentação** | **4** | `AGENTS.md` com mapa "quero X → Y" (`:74-101`); `wiki/specs` (22 documentos + `README.md`, estrutura comum em `specs/README.md`); `wiki/usage` (21 guias); `MODULE.md` por crate; `llms.txt`; skill embutida | Estrutura exemplar e com dono declarado. Perde 1 por **deriva comprovada**: matriz diz exit 7 no reinit, código e execução dizem 0 (§4.1). Documentação que descreve comportamento sem teste envelhece — e aqui envelheceu. |
| **Manutenção** | **4** | gate único (`Makefile:11`), 300 linhas verdes (0 violações em 424 arquivos), `-D warnings`, `deny.toml`, `DIVERGENCES.md` com 120 bordas e teste, 963 funções `#[test]`, proptest, crash-injection | Os gates são reais e baratos. Perde 1 por **custo de contexto** (225 ids de decisão, 74 docs de plano) e por achados de §4: número de testes desatualizado no índice, classificação de erro imprecisa em `sync`. Manutenção de *código* é forte; manutenção de *prosa* não tem gate. |
| **Adequação ao nosso caso** | **3** | Produto: zero (não é editor). Processo: alto — 8 dos 10 itens de §5 aplicam-se sem adaptação | A nota 3 é deliberada: não é 5 porque nada de editor se aproveita; não é 0 porque o **método** (gate, 300 linhas, contrato de bytes, DIVERGENCES, matriz de aceite) é exatamente o que falta no nosso projeto. |

Média das dimensões aplicáveis (excluindo N/A e "adequação"): **3,86/5**.

---

## 4. Falhas e riscos encontrados na execução (não estavam no README)

### 4.1 Deriva entre matriz de aceite e binário

- A matriz afirma: `kd init` → `config (7) se já existe sem --force`
  (`plan/implementation/17_matriz_aceitacao.md:23`).
- O código diz: `/// Executa kd init (idempotente; --force sobrescreve a config)`
  (`crates/knudge-cli/src/commands/init.rs:58`).
- A execução real: **exit 0**, com `config.toml (inalterado)`; `--force` muda para
  `config.toml (atualizado)`.

Consequência prática: um script que depende do exit 7 para detectar "já inicializado" quebra.
A coluna "Teste" da matriz aponta `golden::init_message_matches_golden` e `init_persistence::*`,
que testam conteúdo, não o exit code alegado.

### 4.2 `sync` classificado como `internal` (70)

`kd sync` num repo com `commit.gpgsign=true` sem chave produziu:

```
erro: erro interno: git commit falhou (status 128): ... gpg: signing failed: Sem chave secreta
exit=70
```

`Internal` significa "bug nosso"; a causa é ambiente. O stderr do git é preservado (bom), mas o
código induz o usuário a abrir issue em vez de corrigir a config. Um `Config`/`Io` seria mais
honesto.

### 4.3 `doctor` sai 0 com `fail`

Observado com `fail derived índice derivado ausente/divergente` e exit 0. Para inspeção
interativa é uma escolha defensável; para CI ("corpus saudável?") obriga a parsear `--json` ou
`grep fail`. Não há flag `--exit-code`.

### 4.4 Vocabulário de batch não descoberto pelo erro

`kd write --batch` com `{"summary": ...}` responde
`linha 1: erro de schema: chave de rascunho desconhecida: summary` — mas não lista
`type/statement/body/tags/anchors` (`write/draft.rs:228-232`). O erro aponta a linha e o
problema, não a solução; a doc de batch teria que ser consultada.

### 4.5 Números de teste envelhecidos

`plan/implementation/README.md:85` cita "458 testes"; o comentário do alvo `miri` fala em "600+"
(`Makefile:100`); a contagem observada de funções `#[test]` é **963**. São métricas diferentes
(funções vs casos executados), mas nenhuma é verificada por gate — é o mesmo padrão de 4.1.
O mesmo vale para a superfície: `llms.txt:3` fala em "15 verbos", `ARCHITECTURE.md:236` em
"14 verbos", e `kd --help` lista **16 subcomandos** (15 verbos + `help`).

### 4.6 O que essas falhas ensinam

Todas são falhas de **prosa que descreve comportamento**, não de código. O knudge tem gate para
bytes, lints, tamanho e supply chain — não tem gate para "a matriz ainda é verdadeira".
A lição transferível é direta: ou a matriz é gerada do código, ou cada linha precisa de um teste
que a execute (não que teste o conteúdo da mensagem).

---

## 5. Lista curta de práticas adotáveis

Ordenada por relação custo/benefício para o `rich-editor-auy`.

| # | Prática | Evidência no knudge | Como aplicar no nosso projeto |
|---:|---|---|---|
| 1 | **Gate único** `make check` = fmt + clippy `-D warnings` + test + tamanho | `Makefile:11`; mesmo alvo no CI (`ci.yml:26`) | Um alvo que roda cargo + os testes de front-end; se o front-end exigir npm, um segundo alvo (`check-web`) chamado pelo mesmo `check`. |
| 2 | **300 linhas por arquivo de produção**, com script, não lint | `check_file_length.sh:6,15`; 0 violações em 424 arquivos | `find crates -name '*.rs' -path '*/src/*'`; adaptar para `packages/*/src/*.ts` com limite próprio (ex.: 250) e excluir testes. |
| 3 | **Lints em duas camadas** + `reason` obrigatório | `Cargo.toml:32-101` + `clippy.toml:114-127`; `allow_attributes_without_reason = deny` (`Cargo.toml:101`) | Copiar a estrutura; adicionar `disallowed-types` para o que fere o modelo (ex.: `Rc` no core, `unsafe` fora do renderer). |
| 4 | **`DIVERGENCES.md` com coluna "teste que trava"** | `DIVERGENCES.md:3-8` e 120 bordas numeradas | Bordas do editor: NFC vs offsets, grapheme clusters, seleção após undo, IME em composição, serialização de nós vazios, colagem de HTML, foco. Cada linha exige teste nomeado. |
| 5 | **Golden + proptest de round-trip byte-exato** antes de features | `TOON.md:93`; `13_testes_qualidade.md:21-35` | Serializar→parsear→serializar idêntico para o documento; golden do JSON de exemplo; proptest sobre sequências de operações. |
| 6 | **Matriz de aceite por comando/API** | `17_matriz_aceitacao.md:1-12,111-114` | Uma linha por comando de edição/API pública: entrada, saída, erro, exit/HTTP, efeito no documento, teste. |
| 7 | **Portas + fakes para tudo que é impuro** | `ARCHITECTURE.md:53-63`; `MODULE.md:27-43` | Portas para DOM, clipboard, IME, FS, relógio; fakes no core; teste transversal "leitura não muta documento". |
| 8 | **Crash-injection e stress de concorrência** | `FaultyFs`, `tests/stress.rs`; `DIVERGENCES.md:20-22` | Persistência: matar entre tmp/rename; carga: dois clientes na mesma sessão; documento: undo concorrente com autosave. |
| 9 | **`Dxx` (produto) ≠ `Rn` (política)**, com matriz de bloqueio | `02_decisoes.md:172-215`; `14_revisao_tecnica.md` | Registrar antes de codar: formato de serialização, modelo de seleção, contrato de eventos, política de `unsafe`/panic, orçamento de dependências. |
| 10 | **Release verificado + bump em script** | `release.yml:34-60`; `bump-version.sh` atualiza Cargo/lock/goldens/docs | Um script de versão que atualiza os goldens junto; CI que falha se `tag != package.json/Cargo.toml`. |

**O que deixaria de fora:** os alvos `miri`/`fuzz` só depois de o núcleo estar estável (não
antes); `llms.txt` só quando houver um doc estável para indexar; skill `.agents/` só quando o
processo estabilizar (antes, ela documentaria ruído).

---

## 6. O que evitar (do knudge)

1. **Copiar o volume de decisões.** 225 `Dxx` para uma CLI. Nosso equivalente deve cobrir só o
   que quebra bytes ou contrato: serialização, modelo de seleção, eventos, extensão.
2. **Texto que descreve comportamento sem teste.** Foi a única fonte real de erro encontrada
   (§4). Cada afirmação numérica/de exit em doc deve vir de teste ou ser gerada.
3. **`#[deny]` global sem exceção local documentada.** O knudge escapa com `allow(reason=...)`;
   sem esse escape, o projeto trava e alguém desliga o lint inteiro.
4. **`HashMap` proibido por gosto.** Aqui a proibição é justificada (ordem é contrato). No nosso
   caso, ordem importa em serialização e seleção, não em todo mapa.
5. **"doctor sai 0"** como padrão se o CI for consumir: definir exit code de saúde desde o
   começo.
6. **Confundir produto com processo.** O knudge não nos ensina a editar texto; ensina a não
   deixar o modelo apodrecer. Não importe BM25, embeddings, dedup ou `rewind`.

---

## 7. Limites deste dossiê

| Lacuna | Impacto | Como fechar |
|---|---|---|
| `make check` / `cargo test` não executados no `ref/` | não sei se a suíte está verde hoje; confio no relatório do projeto (`README` de `plan/implementation` diz verde) | `cd ref/knudge && make check` |
| `make ci` (miri/deny/audit/machete/typos) não executado | não validei supply chain nem UB | instalar as cinco ferramentas e rodar `make ci` |
| `make bench` não reexecutado | números vêm do artefato versionado, não medidos agora | `cd ref/knudge && make bench` |
| `kd sync` caminho feliz | observado nesta revisão (`commit.gpgsign false` → exit 0, commit criado); a falha por GPG também foi reproduzida (exit 70) | nada a fazer |
| Colaboração multi-agente (`kd ai`) não exercitada | a avaliação de colaboração apoia-se em doc + changelog | criar `.knudge/ai/memory/<agent>.jsonl` e rodar `kd ai --memory` |
| Sem inspeção de `knudge-mcp` em execução | MCP avaliado só por doc/arquitetura | falar JSON-RPC com `knudge-mcp` via stdio |
| Profundidade de `retrieval/`, `embeddings/`, `lifecycle/` limitada a docs e contagens | intencional: não é o foco do dossiê | ler `wiki/specs/{busca,embeddings,ciclo-de-vida,matematica}.md` |

Confiança nas afirmações de **processo** (gates, lints, testes, documentação, release): alta —
todas citam arquivo e linha, e as principais foram executadas. Confiança nas afirmações de
**comportamento interno** de retrieval/embeddings: média — apoiadas em documentação do próprio
projeto, não em leitura de código nem execução.
