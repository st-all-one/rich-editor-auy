# Arquitetura **de processo** do knudge

> Este documento não descreve um editor. Descreve **como o knudge se impede de apodrecer**:
> que artefatos existem, qual é a autoridade de cada um, que gates bloqueiam o merge, como uma
> decisão vira código e teste, e como uma borda vira contrato verificável.
>
> Fatos de produto (BM25, RRF, embeddings, MCP) aparecem só para delimitar o que **não**
> transfere. Para a comparação como editor, ver [`avaliacao.md` §0](avaliacao.md#0-escopo-e-ressalva-a-comparação-como-editor-não-se-aplica).

---

## 1. A cadeia de autoridade

O knudge tem sete camadas de artefato, e cada uma tem um papel declarado — não são pastas
soltas:

```
plan/brainstorm/      registro histórico (NUNCA reescrito)     AGENTS.md:188
      ↓
plan/00_panorama.md   visão + "fixo vs. oscilante"             panorâma §1/§11
      ↓
plan/02_decisoes.md   pontos abertos + matriz de bloqueio      plan/02_decisoes.md:172
      ↓
plan/03_decisoes-fechadas.md  D01–D224 (213 ids; contrato de produto)   433 linhas
      ↓
plan/implementation/  políticas Rn + épicos E→T com aceite    14_revisao_tecnica.md, 01..24
      ↓
código                regras vêm dos Rn/Dxx, nunca o contrário
      ↓
wiki/specs + wiki/usage + MODULE.md + llms.txt  documentação derivada, com dono
```

Duas regras sustentam a cadeia:

1. **O brainstorm é registro, não plano.** `AGENTS.md:188` proíbe reescrevê-lo. Isso evita o
   clássico "o documento de ideias vira documento de arquitetura por acidente".
2. **Código e documentação derivam das decisões.** `plan/03_decisoes-fechadas.md:3` diz
   literalmente: "a partir daqui, código e documentação derivam daqui".

A camada mais útil para nós é a **separação entre tipos de artefato**:
visão → ponto aberto → decisão fechada → política de engenharia → épico/tarefa → teste.
Cada salto tem um formato distinto e um dono distinto.

---

## 2. Duas espécies de registro: `Dxx` e `Rnn`

Esta é a ideia mais transferível do projeto.

| | `Dxx` — decisão | `Rnn` — política de engenharia |
|---|---|---|
| Natureza | **Produto/contrato**: o que o sistema é | **Como se escreve** o sistema |
| Exemplo | `D01` ID endereçado por conteúdo; `D20` tmp+rename | `R01` `unsafe` confinado; `R30` `thiserror` no core |
| Vive em | `plan/03_decisoes-fechadas.md` (D01–D224) | `plan/implementation/14_revisao_tecnica.md` (R01–R45) |
| Formato | tabela `#` + decisão final + status ✅/⚠️ | achado com **estado atual → recomendação → onde → aceite** |
| Consequência | mudar exige novo `Dxx` + propagação | violar quebra lint/gate/CI |
| Trava | teste/golden citado na tarefa | `clippy.toml`, `deny.toml`, `[workspace.lints]`, gate de linhas |

Exemplo de disciplina real: `R01` pede `#![forbid(unsafe_code)]` e Miri
(`14_revisao_tecnica.md:37-45`); `R44` fixa o Clippy em duas camadas
(`14_revisao_tecnica.md:236-240`); `R45` fixa invariantes de retenção/reescrita que **não são
sobre uma feature**, e sim sobre "toda feature que toque shelf-life"
(`14_revisao_tecnica.md:241-250`).

Por que funciona: um `Dxx` não pode ser "esquecido" porque a tarefa que o implementa cita o id;
uma `Rn` não pode ser esquecida porque o gate a executa. **Decisão sem teste é intenção;
política sem gate é comentário.**

### 2.1 Matriz de bloqueio

`plan/02_decisoes.md:172-198` ordena as decisões em camadas e proíbe avançar antes do
alicerce: "nada de D52 em diante deve ser implementado antes de D01–D18 estarem fechadas".
Isso é gestão de risco escrita como grafo de dependência de decisões — e não como cronograma.

O mesmo documento lista as "10 decisões que mais previnem dor" (`plan/02_decisoes.md:203-215`),
todas de contrato (bytes, tolerância de leitura, lock, dedup, saída para máquina, worktree).
Nenhuma é sobre feature.

---

## 3. Planejamento: proposta → épico → tarefa

Três formatos, três perguntas:

| Artefato | Pergunta que responde | Exemplo |
|---|---|---|
| `plan/proposals/*.md` (35 arquivos) | **por que** e **com que medição** | `qualidade_busca_depreciacao.md`, `riscos_memoria_duravel.md` |
| `plan/implementation/<NN>_<epico>.md` | **o que** entra, com **aceite verificável** | `01_fundacao_workspace.md` |
| `plan/implementation/24_plano_mestre_0.5.0.md` | **em que ordem** e **com que gate** | ordem integrada E16–E19 |

Convenções declaradas em `plan/implementation/README.md:26-34`:

- tarefa = `E<épico>-T<nn>`, status `☐ / ◐ / ☑`;
- **toda tarefa cita as decisões que implementa** (`Dxx`);
- **aceite é critério verificável** (teste, golden, propriedade, script) — "não prosa";
- disciplina Rust dentro do próprio plano: ≤300 linhas, sem `unwrap/expect/panic`, `fmt` +
  `clippy -D warnings` sempre verdes.

O plano-mestre acrescenta o que planos normalmente omitem: **autoridade única por assunto**
(`24_plano_mestre_0.5.0.md:22-40`) — uma tabela "assunto → documento autoritativo → papel", para
que ninguém precise decidir qual documento está certo.

E acrescenta uma regra de honestidade: "se o número não paga (R43) ou regride sem ganho,
**reverta** e registre 'rejeitado por medição'".
(`24_plano_mestre_0.5.0.md:19-20`).

`17_matriz_aceitacao.md` é o artefato de fecho: uma linha por verbo com **pipe, `--json`, erro,
exit code, estado em disco e teste**. A coluna "Teste" é obrigatória. É a diferença entre
"documentei a CLI" e "a CLI tem contrato".

---

## 4. O gate único e os gates extras

### 4.1 `make check` — a única porta local

```make
check: fmt clippy test file-length     # Makefile:11
fmt:     cargo fmt --all -- --check    # Makefile:14
clippy:  cargo clippy --workspace --all-targets -- -D warnings   # Makefile:18
test:    cargo test --workspace        # Makefile:22
file-length: ./scripts/check_file_length.sh   # Makefile:30
```

Quatro propriedades fazem esse alvo funcionar:

1. **É um comando só.** `AGENTS.md:27` chama de "portão único"; ninguém precisa montar a
   sequência.
2. **`-D warnings` no clippy**, não `warn`. Warning é falha.
3. **`--all-targets`**: testes e benches também são lintados (por isso
   `allow-unwrap-in-tests = false` em `clippy.toml:16-18`).
4. **O gate de tamanho é um script próprio**, não um lint: `check_file_length.sh:6` fixa
   `limit=300`, `:15` varre só `crates/**/src/*.rs` — **teste embutido conta**, então o limite é
   do arquivo, não do "código de produção". Binários de teste e `fuzz/` ficam fora.

### 4.2 `make ci` — extras que **pulam** quando a ferramenta falta

```make
ci: check miri deny audit machete typos      # Makefile:128
```

Cada alvo extra é condicional:

```make
deny:
	@if command -v cargo-deny >/dev/null 2>&1; then $(CARGO) deny check; \
	else echo "cargo-deny ausente; pule (cargo install cargo-deny --locked)"; fi
```

Isso é ergonomia deliberada: o contribuidor sem `cargo-miri`/`cargo-audit` **não fica
bloqueado**, mas o CI tem as ferramentas e não pula. O comentário do alvo `miri`
(`Makefile:97-101`) explica até o escopo: roda `tests/miri_smoke.rs` em segundos, porque o core é
`forbid(unsafe_code)` e a cobertura funcional fica no `make check`.

### 4.3 CI em cinco jobs

`ref/knudge/.github/workflows/ci.yml`:

| Job | O que roda | Linha |
|---|---|---|
| `check` | `make check` (o mesmo comando local) | :26 |
| `windows` | smoke do wrapper de scripts no Windows | :29-36 |
| `supply-chain` | `cargo deny` + `audit` + `machete` + `typos` | :40-53 |
| `miri` | suíte mínima no core puro, `-Zmiri-disable-isolation` | :56-74 |
| `fuzz` | build + **30 s por alvo** (`toon_parse`, `jsonl_decode`) | :77-92 |

Três detalhes de engenharia que valem copiar:

- `env: RUSTFLAGS: "-D warnings"` global (`ci.yml:12`) — protege até `cargo build`;
- `make check` no CI **é o mesmo alvo do dev**, não uma reimplementação em YAML;
- fuzz com tempo fixo (30 s) em vez de "roda até alguém matar": cabe no orçamento do CI.

O `release.yml` adiciona o portão que quase todo projeto esquece: **tag == versão do
`Cargo.toml`**, verificado antes de compilar a matriz (`release.yml:46-60`).

---

## 5. Disciplina de código

### 5.1 Duas camadas de lint, não uma

`15_clippy_config.md:17-30` explica o erro comum: `clippy.toml` só define **valores**
(thresholds, allowlists); os **níveis** (`deny`/`warn`) ficam em `[workspace.lints]`. Sem a
segunda camada, as lints de `restriction` são `allow` por padrão e o `clippy.toml` não faz nada.

No knudge:

| Camada | Onde | Exemplos |
|---|---|---|
| Níveis | `Cargo.toml:32-101` | `unwrap_used/expect_used/panic = deny` (:68-70), `indexing_slicing = deny` (:79), `arithmetic_side_effects = warn` (:80), grupos `all/correctness/suspicious/style/complexity/perf` em `deny` |
| Valores | `clippy.toml` | `cognitive-complexity-threshold = 15` (:43), `too-many-lines-threshold = 60` (:45), `disallowed-types` (:114), `msrv = "1.99"` (:74) |

### 5.2 Proibições com motivo, não com gosto

`clippy.toml:114-127` proíbe tipos e **explica cada um**:

- `Rc`/`Weak`/`RefCell`/`Cell` → não são `Send`/`Sync` (R03);
- `HashMap`/`HashSet` → "iteração não determinística; use BTreeMap/IndexMap";
- `LinkedList` → "cache-hostil";
- `transmute`/`forget`/`zeroed`/`ptr::*` → UB/vazamento;
- `SystemTime::now`/`Instant::now`/`env::var` → "não determinístico; use o port Clock/Env";
- `process::exit` → "propague Result; exit codes no adaptador".

Isso é a diferença entre "odeio `HashMap`" e "a ordem de iteração é contrato de bytes, logo
`HashMap` é um bug". Um revisor novo entende a regra em 3 segundos.

`AGENTS.md:119` fecha o ciclo: **todo `#[allow]` exige `reason`**
(`allow_attributes_without_reason = deny`). Sem isso, o rigor vira teatro: qualquer um silencia
o lint.

### 5.3 O limite de 300 linhas na prática

Resultado observado nesta máquina:

```
$ bash scripts/check_file_length.sh
file-length: ok (limite 300)      # exit 0
$ # maior arquivo de produção:
300 crates/knudge-core/src/adapters/http.rs      # exatamente no limite
294 crates/knudge-core/src/health/validator/mod.rs
```

Zero arquivos acima do limite em 424 arquivos (o limite do script é `> 300`, então 300 passa). O mecanismo de fatiamento é declarado em
`AGENTS.md:123-125`: ao passar de ~280 linhas, separar em `mod.rs` + subarquivos por
responsabilidade (ex.: `toon/{lex,flow,parse,emit}.rs`).

**Custo:** o projeto tem muitos arquivos pequenos e módulos `tests.rs` próprios. **Ganho:** o
diff de review cabe na cabeça; a colisão de merge cai; e o limite é objetivo — não há discussão.

### 5.4 Orçamento de dependências

`deny.toml` proíbe `tokio`, `reqwest`, `hyper`, `axum` (`deny.toml:33-38`), exige licença da
allowlist (`:11-26`), nega registry/git desconhecidos (`:42-44`) e nega wildcard (`:30`).
`[workspace.dependencies]` centraliza versões (`Cargo.toml:17-30`) e `cargo machete` pega
dependência não usada.

O efeito observado é um grafo de dependências pequeno: o core usa só `thiserror`, `indexmap`,
`unicode-normalization`, `sha2` (`crates/knudge-core/MODULE.md:9-11`).

### 5.5 Perfis de build como decisão

`Cargo.toml:133-142`: release com `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`,
`overflow-checks = true`, `panic = "abort"`; dev com `debug-assertions` e `overflow-checks`.
`overflow-checks` no release é raro e deliberado (R02): IDs em base36 e aritmética de orçamento
não podem estourar em silêncio.

---

## 6. Erros como valores

`crates/knudge-core/src/error.rs` (254 linhas, copiado em `codigo/error.rs`) é o modelo:

- `ErrorKind` fechado e `#[non_exhaustive]` (`error.rs:14`), com `code()` (`:38`) — contrato de
  máquina, nunca traduzido — e `exit_code()` (`:56`);
- `enum Error` com `thiserror`, `Send + Sync + 'static`, sem `Box<dyn Error>` na API pública
  (`error.rs:74`);
- `pub type Result<T>` (`:194`);
- `lock_or_recover` resolve poison com `into_inner` (`:199`) — nunca `.expect("poisoned")`.

Mapa observado e documentado (`wiki/specs/erros.md`):

| Kind | code | exit |
|---|---|---|
| `InvalidInput` | `invalid_input` | 2 |
| `NotFound` | `not_found` | 3 |
| `Conflict` | `conflict` | 4 |
| `Io` | `io` | 5 |
| `Timeout` | `timeout` | 6 |
| `Config` | `config` | 7 |
| `Schema` | `schema` | 8 |
| `UnsafeBlocked` | `unsafe_blocked` | 9 |
| `Internal` | `internal` | 70 |

`101` é reservado a panic e `0` a sucesso. `retryable()` só é verdadeiro para `Timeout`.

A parte transferível não é a tabela — é o **contrato duplo**: mensagem humana em português
(golden), código estável em ASCII para a máquina (`--json`), e exit code separado do código.
Isso permite script, MCP e humano sem parsing de prosa.

---

## 7. Camada de testes: golden, proptest, stress, crash

`wiki/specs/ARCHITECTURE.md:264-278` resume a pirâmide, e `13_testes_qualidade.md` a detalha em
tarefas com aceite:

| Camada | Trava | Onde |
|---|---|---|
| **Golden** | bytes de `prime`, `--json`, mensagens de erro, `init`, EPIPE | `crates/knudge-cli/tests/golden/*` (8 arquivos) |
| **Property** | round-trip TOON, RRF (determinismo/união/monotonicidade), decay, IDs, `normalize` idempotente | 20 blocos `proptest!` + `proptest-regressions/` |
| **Stress** | lock sem lost update, escritas concorrentes, leitor × rebuild | `crates/knudge-core/tests/stress.rs` |
| **Crash-injection** | nota-antes-de-evento, tmp+rename, rebuild double-buffer | `FaultyFs` |
| **Bordas** | catálogo com o teste que trava cada uma | `DIVERGENCES.md` |
| **Aceite** | pipe/`--json`/erro/exit/estado por verbo | `17_matriz_aceitacao.md` |

### 7.1 `DIVERGENCES.md` — o artefato mais copiável

`wiki/specs/DIVERGENCES.md:3-8` declara o problema: o projeto não tem "produto de referência"
para harness diferencial, então o papel é ocupado por um **catálogo de bordas**, cada uma com
mitigação e teste. O arquivo tem 145 linhas e **120 bordas numeradas** (a numeração vai de 1 a
121, mas o nº 116 não existe).

Linhas exemplares, e por que importam para um editor:

| # | Borda | Por que é análoga ao nosso caso |
|---|---|---|
| 1 | Unicode/NFC afetando id/hash (`:12`) | texto digitado, acentos, normalização |
| 5 | Ordem de chaves no frontmatter (`DIVERGENCES.md:16`) | ordem de propriedades no documento |
| 9 | Escrita atômica tmp+rename (`:20`) | autosave/persistência |
| 10 | Nota antes do evento (`:21`) | ordem de commit em modelo com histórico |
| 11 | Rebuild double-buffer (`:22`) | índice derivado de documento |
| 17 | Log não pode vazar em stdout (`:28`) | contrato de saída JSON |
| 19 | EPIPE → exit 0 (`:30`) | CLI em pipeline |
| 107 | O(N²) de dedup vira LSH acima de 256 notas (`:118`) | performance com degradação documentada |
| 110 | Chave aditiva não quebra rebuild byte-idêntico (`:121`) | evolução de schema sem migração |

O padrão é sempre o mesmo: **borda → risco → mitigação → teste que trava**. Nenhuma linha diz
"cuidado com Unicode"; todas dizem qual teste falha se você quebrar.

### 7.2 Determinismo por portas

`wiki/specs/ARCHITECTURE.md:7-14`: o domínio não conhece terminal, `argv`, relógio, RNG nem FS.
Tudo atravessa uma trait em `ports` com fake correspondente (`FixedClock`, `SeqRng`, `MemFs`,
`FaultyFs`, `FakeGit`, `RecordingLogger`).

Consequência: os testes do core são reprodutíveis byte a byte, e crash-injection é possível sem
matar processo (`FaultyFs` injeta falha). Para nós, o análogo é: **o núcleo do editor não deve
conhecer DOM, relógio, clipboard nem FS**; essas bordas ficam atrás de portas, com fakes.

### 7.3 Uma decisão pode virar um arquivo de teste

`crates/knudge-cli/tests/invariants.rs` existe para **R45**: não testa feature, testa
propriedades transversais de leitura (não escrever nota nem derivado por default; renovação só
estende; supersessão vence uso). O comentário de topo diz isso explicitamente (`invariants.rs:1-5`).

Este é um padrão raro e valioso: quando a invariante é **negativa** ("leitura não escreve"), ela
não pertence a um módulo, pertence a um teste transversal.

---

## 8. Matriz de aceite e documentação em camadas

### 8.1 A matriz

`17_matriz_aceitacao.md:1-12` define o critério de "pronto" de uma tool: formato pipe, envelope
`--json`, erro + exit code, estado de `.knudge/` e teste. `:111-114` diz como manter: ao
mudar um verbo, atualizar a linha **e** o teste; mudança intencional de saída atualiza o golden.

O documento tem 60+ linhas de verbo e uma seção de invariantes transversais (`:76-86`): JSON
válido sem stderr, `kd == kd --help`, EPIPE → 0, comando desconhecido → 2, `strict` só por
config, `learn/compact/doctor` não escrevem sem aceite.

### 8.2 Documentação com dono

| Camada | Arquivo | Público | Regra de manutenção |
|---|---|---|---|
| Contribuição | `AGENTS.md` (14,8 KB) | quem mexe no código | mapa "quero X → vá para Y" e checklist final (`:196-212`) |
| Módulo | `crates/*/MODULE.md` | quem mexe no crate | "mantenha em sincronia ao criar/remover módulos" (`AGENTS.md:74-75`) |
| Técnica | `wiki/specs/*` (22 documentos + `README.md`; 210 ADRs em `adrs/`) | quem implementa | mesma estrutura: papel, invariantes, fluxo, contratos, onde vive, testes |
| Uso | `wiki/usage/*` (21 guias) | usuário do `kd` | tarefa a tarefa |
| Índice para LLM | `llms.txt` (32 KB) | agente | regras de ouro + links |
| Skill | `.agents/skills/rust/` e `kd/` | agente | carregada por contexto |

Três detalhes que evitam documentação morta:

- `AGENTS.md` diz **onde** mexer por assunto, não repete a arquitetura;
- `wiki/specs/README.md` declara a estrutura comum de cada doc técnico;
- `AGENTS.md:196-212` transforma documentação em **checklist de conclusão**, não em cortesia.

---

## 9. Release e distribuição

- `release.yml` compila matriz nativa (musl estático, macOS nativo + cross, Windows MSVC nativo
  + cross), sem Docker (`release.yml:1-10`), e publica `.tar.gz`/`.zip` + `sha256sums.txt`.
- `install.sh` **verifica SHA-256 antes de instalar** (`install.sh:250-262`) e exige HTTPS por
  padrão (`:49-51`).
- `scripts/bump-version.sh` atualiza **todos** os pontos que citam a versão: `Cargo.toml`,
  `Cargo.lock`, goldens (`json_prime.json`, `json_version.json`), `install.sh`, `README.md` e
  `CHANGELOG.md`. É a resposta correta para "release quebrou golden".

Esse é um padrão transferível direto: **versão é um dado com N cópias; bump é um script, não
um checklist humano**. E o CI verifica a cópia mais crítica (tag vs `Cargo.toml`).

---

## 10. O que é específico de memória/IA e **não** transfere

| Peça | Por que não transfere |
|---|---|
| BM25/RRF/âncoras/PPR (`retrieval/`) | busca textual sobre notas; um editor não ranqueia documentos assim |
| Embeddings, cache vetorial, `drain`, worker (`embeddings/`) | infraestrutura de IA opcional; nosso core não deve falar HTTP |
| Dedup por similaridade (0,75/0,92) e MinHash/LSH | resolvedor de duplicata de conhecimento |
| `shelf-life`, decay, confiança Beta, FSRS, drift KL/JS | ciclo de vida de conhecimento |
| Hierarquia `epic ⊃ issue ⊃ task`, `rewind`, orçamento de tokens | gestão de trabalho e handoff de agente |
| MCP/JSON-RPC, gatilhos proativos (`knudge-mcp`) | protocolo de agente |
| TOON e as 31 chaves | contrato de bytes de **nota**; o nosso contrato é o documento do editor |

O que **transfere** dessas peças é o *método*, não o conteúdo: toda borda acima virou teste
nomeado e linha em `DIVERGENCES.md`. Isso é replicável mesmo que o domínio mude.

---

## 11. Custos, riscos e o que o processo **não** pega

### 11.1 Custo

- 225 ids de decisão e 74 documentos Markdown em `plan/` exigem leitura de contexto para contribuir. Um
  `AGENTS.md` de 14,8 KB é o preço de entrada.
- 60 mil linhas para uma CLI: a razão feature/processo é baixa. Copiar o *formato* sem o
  *volume* é essencial.
- Alvos extras (`miri`, `fuzz`, `deny`) só valem porque **pulam** quando ausentes; se fossem
  obrigatórios localmente, a fricção mataria a adoção.

### 11.2 O que o processo não pega (achados reais)

1. **Deriva entre matriz e código.** `17_matriz_aceitacao.md:23` afirma
   `config (7) se já existe sem --force`; `knudge-cli/src/commands/init.rs:58` diz "idempotente;
   `--force` sobrescreve a config"; execução real: **exit 0**, tudo `inalterado`.
   Nenhum gate compara a matriz com o binário.
2. **Números de teste desatualizados em índices.** `plan/implementation/README.md:85` cita
   "458 testes"; o comentário do alvo `miri` fala em "600+" (`Makefile:100`); hoje há 963
   funções `#[test]`. Também não é verificado — e `AGENTS.md:211-212` ainda diz `D01–D216`/
   `R01–R44` quando o corpus já chegou a `D224`/`R45`.
3. **Classificação de erro imprecisa em integração externa.** `kd sync` com git exigindo
   assinatura deu exit **70 `internal`** — reservado a "bug nosso" — quando a causa é ambiente.
4. **`doctor` sai 0 com `fail`.** Bom para inspeção, ruim para CI; é preciso `--json` +
   `jq`/`grep`.

Lição: **golden e gate pegam bytes; não pegam prosa**. Documentos que descrevem comportamento
precisam ser gerados do código ou verificados por teste, senão envelhecem como qualquer README.

### 11.3 Comparação estrutural com o nosso caso

| Dimensão | knudge | `rich-editor-auy` |
|---|---|---|
| Fonte da verdade | `notas/` (Markdown + TOON) | modelo de documento + serialização |
| Derivado | `.idx/` reconstruível | layout, índices de posição, DOM espelhado |
| Contrato de bytes | TOON + 31 chaves + `body_hash` | formato de serialização + schema de nós/marks |
| Entrada hostil | parser TOON/JSONL (fuzz) | HTML/colagem/IME/Unicode (fuzz + corpus) |
| Concorrência | processos CLI + lock de arquivo | editor único + sessão remota, se houver |
| Porta/fake | `Fs`, `Clock`, `Git`, `Logger` | DOM, clipboard, IME, FS, relógio |
| Teste transversal | `invariants.rs` (leitura não escreve) | "leitura não muta documento", "undo não perde seleção" |
| Superfície | 15 verbos de CLI | comandos de edição + API do front-end |

O mapeamento é quase 1:1 nos eixos de **contrato**; divergem só nas peças de IA. A tradução
concreta está em [`avaliacao.md` §5](avaliacao.md#5-lista-curta-de-práticas-adotáveis).
