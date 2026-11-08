# Revisão independente — `dossie/knudge/`

> Revisor distinto do explorador. Método: ler `_GUIA.md`, `_RUBRICA.md`, `_REVISÃO-CHECKS.md`,
> o dossiê inteiro e a fonte em `ref/knudge/`; **reexecutar** o que é barato; corrigir o dossiê;
> registrar o que não foi possível verificar. Nada aqui foi inferido sem dizer que é inferência.

---

## 0. Veredito

**O knudge não é um editor.** Não há DOM, `contenteditable`, renderer, CSS, seleção, IME,
clipboard, undo de edição, colaboração ou ARIA; os três crates são CLI/domínio/MCP
(`ref/knudge/Cargo.toml:4`). A comparação "como editor" **não se aplica** e o dossiê diz isso na
primeira linha, na §0 da avaliação e na §10 da arquitetura — corretamente.

O valor do caso é de **processo**: gate único, limite de 300 linhas, lints em duas camadas com
`reason`, catálogo de bordas com teste, contrato de bytes com golden/proptest, matriz de aceite
por verbo e release com versão verificada. Isso se sustenta na fonte lida e no binário executado.

**Confiança final: 88%** (§5). Abaixo de 90 porque a suíte de testes **não foi executada**
(proibição de build do workspace), então "963 testes" é contagem estática, não "verde".

---

## 1. Achados (e o que foi corrigido)

### 1.1 Números errados — corrigidos

| # | Onde | Estava | Real (comando) |
|---:|---|---|---|
| A1 | `README` ficha, `arquitetura` §5.3 | maior arquivo de produção = **294** linhas | **300** linhas — `crates/knudge-core/src/adapters/http.rs`, exatamente no limite; 294 é o segundo (`health/validator/mod.rs`). O script compara `> 300`, então 300 passa |
| A2 | `README` ficha, `arquitetura` §7.1, `avaliacao` §2/§3/§5 | **121** bordas / "121 linhas de tabela" | **120** bordas numeradas; a numeração vai de 1 a **121** mas o nº **116 não existe** (`awk` por linha numerada; `grep -c` = 120; arquivo = 145 linhas) |
| A3 | `README`, `arquitetura` §1/§2 | decisões **D01–D216** | arquivo vai até **D224**; **213** ids distintos dentro dele; **225** ids citados em `plan/`+`wiki/` (`sort -u`). O `D01–D216` do dossiê vinha de `AGENTS.md:211`, que está **desatualizado** |
| A4 | `arquitetura` §3 | `plan/proposals` = **39** arquivos | **35** arquivos `.md` |
| A5 | `arquitetura` §8.2, §11.1 | wiki/specs **24 documentos**; "~90 documentos de plano" | `wiki/specs`: **22 documentos + README** (e 210 ADRs em `adrs/`); `plan/`: **74** `.md` |
| A6 | `avaliacao` §3 API | core com **20 módulos** (`lib.rs:32-53`) | **23** `pub mod` (`lib.rs:33-55`) |
| A7 | `codigo/INDEX.md` | `clippy.toml` **252** linhas, `Cargo.toml` **143**, script **22** | **194**, **142**, **21** (byte-idênticos à origem) |

### 1.2 Referências de linha erradas — corrigidas

| Onde | Estava | Correto |
|---|---|---|
| `arquitetura` §8.2, `avaliacao` §3 docs | `AGENTS.md:158` como "checklist de conclusão" | `AGENTS.md:196-212` (o `:158` é sobre `unwrap` em testes) |
| `arquitetura` §5.4 | `deny.toml:35-40`, licenças `:12-25`, deps `Cargo.toml:14-30` | `deny.toml:33-38`, `:11-26`, `Cargo.toml:17-30` |
| `arquitetura` §5.5 | perfis `Cargo.toml:132-142` | `:133-142` (`[profile.release]` na 133) |
| `arquitetura` §4.3 | job `supply-chain` em `:50-53` | job começa em `:40`; `:50-53` são os `run` |
| `avaliacao` §4.5 | `ARCHITECTURE.md:234` para "14 verbos" | `:236` |
| `avaliacao` §1 (Unicode) | `ARCHITECTURE.md:150-163` como fonte de NFC | `:153` cobre o **fold** de diacríticos no índice; NFC/`normalize` está em `DIVERGENCES.md:12` e `schema/` |
| `codigo/INDEX.md` | `allow-unwrap-in-tests` em `:16-18` | `clippy.toml:18` |

### 1.3 Contexto que faltava (não era número errado, era leitura enganosa)

- **"963 testes"** é **963 funções anotadas com `#[test]`** (0 `#[tokio::test]`, 0 `#[rstest]`),
  mais **20 blocos `proptest!`** que geram casos em runtime. O número não é "casos executados".
  A fonte se contradiz: `plan/implementation/README.md:85` diz "458 testes" e `Makefile:100` diz
  "600+". O dossiê agora rotula a métrica e cita as três versões.
- **`kd init`** imprime 7 linhas de status **e depois** `próximos:` + o protocolo `prime` inteiro
  (~40 linhas). O recorte do dossiê era legítimo, mas não estava marcado como recorte. Agora está.
- **`kd ask` local**: 0,04–0,05 s **a quente**; execuções isoladas chegaram a 0,13–0,14 s
  (mediana ~0,05 s, 10 execuções). A faixa antiga era boa demais para ser uma faixa.
- **Tipos de nota**: `notas/` tem 10 tipos de conhecimento + `epic/` para containers — não
  "10 tipos armazenáveis + epic derivado". Corrigido para a taxonomia real do `prime`.

### 1.4 Achado novo (não estava no dossiê)

**`kd write --batch` com erro de schema sai 0.** Com `{"summary":"..."}` (chave inválida):

```
aviso: linha 1: erro de schema: chave de rascunho desconhecida: summary
exit=0
```

Ou seja, o lote reporta o problema como **aviso** e não falha o processo. Isso reforça o ponto
de "diagnóstico informativo × exit code" que o dossiê já faz para `doctor`, mas em um caminho
onde o usuário pode perder linhas silenciosamente. Adicionado ao quadro de exit codes do README.

### 1.5 O que foi confirmado (não mudou)

Reexecutado nesta máquina com `kd 0.6.4`, em diretórios temporários:

- `kd init` → 7 linhas de status, 11 diretórios, `config list` → `project (69)` (69 chaves +
  cabeçalho);
- `kd init` de novo → tudo `inalterado`, **exit 0**; `--force` muda **só** `config.toml`;
- `write` → `created|fact_0094if9p|r1`; mesmo statement → `unchanged|...|r1`; quase-duplicata →
  `merged|...|r2`; duplicata com outro tipo → `rejected|...` + aviso, exit 0;
- `ask` textual → `id|statement|0.51|semantic`; `--json 2>/dev/null` → JSON válido;
- `task new/list`, `rewind` (manifest com `notes/ready/blocked/containers/dirty`, `recent`,
  `next`, `fresh`), `doctor` (**13 checks**, `fail derived` com **exit 0**), `doctor --fix`
  idempotente (mesmo `md5` do `.idx/` em duas execuções), `map` sem escopo → exit 2;
- `kd prime` byte-idêntico entre execuções (mesmo SHA-256), `prime | head -1` → EPIPE exit 0;
- `kd sync` **caminho feliz** (repo com `commit.gpgsign false`) → `sync main: 2 arquivo(s)
  commitados`, exit 0; falha por GPG do host → **exit 70** com stderr do git preservado;
- exit codes: verbo inválido 2, `--json` sem verbo 2, statement vazio 2, `write --type task` 2
  (D93), `task close` sem `--outcome` 2, `ask --id` inexistente → aviso + exit 0;
- 4 `write` paralelos com statements **distintos** → 4 notas, `doctor` sem lock stale nem
  duplicata. (Com statements quase iguais, o dedup colapsa em `unchanged`; isso é dedup, não
  lost update — vale notar para quem repetir o teste.)

Também: `bash scripts/check_file_length.sh` → `file-length: ok (limite 300)`, exit 0; as 6 cópias
de `codigo/` são **byte-idênticas** (`cmp`, 6/6); `kd --help` lista **16 subcomandos** (15 verbos
+ `help`).

### 1.6 Correção de exemplo

O script `exemplos/ciclo-kd.sh` configurava `commit.gpgsign false` **para o `kd sync`** mas nunca
chamava `sync`. Adicionei o passo, reexecutei o script inteiro e colei a saída nova
(`sync main: 2 arquivo(s) commitados`) no `exemplos/README.md`; a linha "não executado" do `sync`
virou "executado".

---

## 2. Correções aplicadas (arquivos)

| Arquivo | Mudanças |
|---|---|
| `README.md` | ficha (963 `#[test]` com contexto, 300 linhas, D224, 120 bordas, evidência do `sort -u`); `init` marcado como recorte e taxonomia de tipos; linha de exit code do `batch`; faixa de tempo do `ask`; "225 decisões" qualificado; deriva `D216`/`R44` incluída nos contras |
| `arquitetura.md` | D01–D224 (2×); proposals 35; job `supply-chain :40-53`; maior arquivo 300; `deny.toml`/`Cargo.toml` (linhas); `wiki/specs` 22+README; `AGENTS.md:196-212`; 74 docs de plano; 120 bordas; números de teste com as três versões |
| `avaliacao.md` | contagem de `init`; tempo do `ask`; `ARCHITECTURE.md:153`; `batch` sai 0; 23 módulos; 22 documentos; 120 bordas; 963 funções; "600+" atribuído só ao `Makefile`; `ARCHITECTURE.md:236`; `sync` agora observado; **cobertura reescrita** (§3 abaixo) |
| `codigo/INDEX.md` | 194/142/21 linhas; `clippy.toml:18` |
| `exemplos/ciclo-kd.sh` | passo `kd sync` adicionado |
| `exemplos/README.md` | saída observada com `sync`; tabela de execução atualizada; descrição do script |
| `revisao.md` | este arquivo |

Nada fora de `dossie/knudge/` foi tocado. `ref/` permaneceu somente leitura.

---

## 3. Auditoria da matriz de 85% (status honesto)

O dossiê declarava "15/15 = 100%". Isso **contava as dimensões de editor como cobertura**, o que
infla a métrica. Reescrevi para:

- **10/10 dimensões plenamente aplicáveis** com evidência (instalação, inicialização, schema,
  persistência, colaboração por shards, performance, erros/diagnóstico, licença, concorrência,
  Unicode) → **100% das aplicáveis**, acima da meta;
- **5 dimensões de editor** (edição rica, seleção/IME/undo, plugin de nó/mark, CSS, ARIA) →
  **N/A**, com análogo estrutural explicitamente rotulado. Não entram no numerador.

Notas 0–5: mantive as do explorador (usabilidade 3, API 4, modelo 5, extensibilidade 3,
performance 4, documentação 4, manutenção 4, adequação 3), porque a evidência que as sustenta
foi verificada. Ressoalva: a nota "adequação 3" é a que expressa "não é editor; é referência de
processo" — está correta e não deve ser lida como nota de produto.

---

## 4. O que **não** foi executado (e por quê)

| Item | Estado | Motivo |
|---|---|---|
| `cargo build` / `make check` / `cargo test` no `ref/` | **não executado** | a tarefa proíbe build do workspace; sem isso, "963 testes verdes" não foi verificado — só a contagem estática |
| `make ci` (miri, deny, audit, machete, typos) | **não executado** | ferramentas ausentes na máquina; os alvos pulariam |
| `make bench` / `cargo fuzz` | **não executado** | exigem release/nightly; números vêm do artefato `bench/ULTIMO.md` |
| `install.sh` oficial (`curl \| bash`) | **não executado** | rede/instalação; o checksum foi lido no código, não exercitado |
| `knudge-mcp` (JSON-RPC stdio) | **não executado** | fora do foco; avaliado só por código/doc |
| `kd map`, `forget`, `drain`, `ai` | **parcial** | `map` sem escopo e `forget` foram exercitados nesta revisão; `drain`/`ai` não |
| Semântica de cada um dos 13 checks do `doctor` | **não auditada** | contei os checks na saída; não validei a lógica de cada um |

Observação de execução: `kd forget --id` devolve `forget|...|r2` (exit 0) e `kd ask --id` ainda
resolve a nota em seguida. É coerente com *soft-delete*, mas o dossiê não descreve esse detalhe;
não alterei o texto para não afirmar semântica que não li no código.

---

## 5. Confiança (0–100%)

| Eixo | Confiança | Base |
|---|---:|---|
| Processo, gates, CI, release, docs | **93%** | arquivos lidos integralmente; gate e cópias reexecutados; citações corrigidas |
| Contagens e números | **96%** | recontados por comando (`find`, `grep`, `awk`, `wc`), não copiados |
| Comportamento de CLI observado | **92%** | 20+ execuções reais; restam `drain`/`ai`/MCP |
| Matriz de usabilidade / cobertura | **88%** | 10/10 aplicáveis com evidência; 5 N/A rotuladas |
| Internos de retrieval/embeddings/lifecycle | **55%** | só doc + leitura pontual; sem execução |
| **Global** | **88%** | teto da faixa 75–89 do guia, porque a suíte não foi rodada |

---

## 6. Lacunas restantes

1. **Verde da suíte**: ninguém rodou `make check` no `ref/`; a afirmação "gate verde" é do
   projeto, não observada. Fechar com `cd ref/knudge && make check`.
2. **Bancada**: os tempos de `bench/ULTIMO.md` não foram reproduzidos; só os de `kd ask` local.
3. **`knudge-mcp`**: nenhuma execução; a seção de colaboração/MCP apoia-se em doc.
4. **`drain`/worker/embeddings**: não exercitados; o dossiê já os trata como "não transfere".
5. **Deriva da fonte**: a matriz de aceite do knudge afirma `config (7)` no reinit e o binário
   devolve 0; `AGENTS.md:211-212` está preso em `D216`/`R44`. São problemas **do projeto**, não
   do dossiê — o dossiê os registra como achado, que é o comportamento correto.
6. **`batch` exit 0**: documentado como observação; não investiguei se é intencional (D110) ou
   bug. Não afirmei intenção.

---

## 7. Estado final e concorrência de edição

Após o aviso de que o explorador poderia ter tocado a pasta, reli os cinco arquivos editáveis:
os `mtime` finais são todos desta revisão (22:30–22:31) e não há seção duplicada ou conflitante.
As verificações finais (`grep` das correções, `cmp` das cópias, reexecução do exemplo) foram
feitas sobre o estado já corrigido. **Não há trabalho pendente em `dossie/knudge/`.**
